//! Thread-safe connections on IrcStreams.
#![experimental]
use std::sync::{Mutex, MutexGuard};
use std::io::{BufferedStream, IoError, IoErrorKind, IoResult, MemWriter, TcpStream};
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use encoding::label::encoding_from_whatwg_label;
use data::kinds::{IrcReader, IrcStream, IrcWriter};
use data::message::Message;
#[cfg(feature = "ssl")] use openssl::ssl::{SslContext, SslStream, Tlsv1};
#[cfg(feature = "ssl")] use openssl::ssl::error::SslError;

/// A thread-safe connection.
#[experimental]
pub struct Connection<T> where T: IrcStream {
    stream: Mutex<T>
}

impl Connection<BufferedStream<TcpStream>> {
    /// Creates a thread-safe TCP connection to the specified server.
    #[experimental]
    pub fn connect(host: &str, port: u16) -> IoResult<Connection<BufferedStream<NetStream>>> {
        Connection::connect_internal(host, port, None)
    }

    /// Creates a thread-safe TCP connection to the specified server with a given timeout in 
    /// milliseconds.
    #[experimental]
    pub fn connect_with_timeout(host: &str, port: u16, timeout_ms: u64) 
        -> IoResult<Connection<BufferedStream<NetStream>>> {   
        Connection::connect_internal(host, port, Some(timeout_ms))
    }

    /// Creates a thread-safe TCP connection with an optional timeout.
    #[experimental]
    fn connect_internal(host: &str, port: u16, timeout_ms: Option<u64>) 
    -> IoResult<Connection<BufferedStream<NetStream>>> {  
        let mut socket = try!(TcpStream::connect(format!("{}:{}", host, port)[]));
        socket.set_timeout(timeout_ms);
        Ok(Connection::new(BufferedStream::new(NetStream::UnsecuredTcpStream(socket))))
    }

    /// Creates a thread-safe TCP connection to the specified server over SSL.
    /// If the library is compiled without SSL support, this method panics.
    #[experimental]
    pub fn connect_ssl(host: &str, port: u16) -> IoResult<Connection<BufferedStream<NetStream>>> {
        Connection::connect_ssl_internal(host, port, None)
    }

    /// Creates a thread-safe TCP connection to the specificed server over SSL with a given timeout
    /// in milliseconds. If the library is compiled without SSL support, this method panics.
    #[experimental]
    pub fn connect_ssl_with_timeout(host: &str, port: u16, timeout_ms: u64)
        -> IoResult<Connection<BufferedStream<NetStream>>> {
        Connection::connect_ssl_internal(host, port, Some(timeout_ms))
    }

    /// Panics because SSL support was not included at compilation.
    #[experimental]
    #[cfg(not(feature = "ssl"))]
    fn connect_ssl_internal(host: &str, port: u16, _: Option<u64>) 
    -> IoResult<Connection<BufferedStream<NetStream>>> {
        panic!("Cannot connect to {}:{} over SSL without compiling with SSL support.", host, port)
    }

    /// Creates a thread-safe TCP connection over SSL with an optional timeout.
    #[experimental]
    #[cfg(feature = "ssl")]
    fn connect_ssl_internal(host: &str, port: u16, timeout_ms: Option<u64>)
    -> IoResult<Connection<BufferedStream<NetStream>>> {
        let mut socket = try!(TcpStream::connect(format!("{}:{}", host, port)[]));
        socket.set_timeout(timeout_ms);
        let ssl = try!(ssl_to_io(SslContext::new(Tlsv1)));
        let ssl_socket = try!(ssl_to_io(SslStream::new(&ssl, socket)));
        Ok(Connection::new(BufferedStream::new(NetStream::SslTcpStream(ssl_socket))))
    }
}

impl<T: IrcStream> Connection<T> {
    /// Creates a new connection from any arbitrary IrcStream.
    #[experimental]
    pub fn new(stream: T) -> Connection<T> {
        Connection {
            stream: Mutex::new(stream),
        }
    }

    /// Sends a Message over this connection.
    #[experimental]
    pub fn send(&self, message: Message, encoding: &str) -> IoResult<()> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => return Err(IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "Failed to find decoder.",
                detail: Some(format!("Invalid decoder: {}", encoding))
            })
        };
        let data = match encoding.encode(message.into_string()[], EncoderTrap::Strict) {
            Ok(data) => data,
            Err(data) => return Err(IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "Failed to decode message.",
                detail: Some(format!("Failed to decode {} as {}.", data, encoding.name())),
            })
        };
        let mut stream = self.stream.lock();
        try!(stream.write(data[]));
        stream.flush()
    }

    /// Receives a single line from this connection.
    #[experimental]
    pub fn recv(&self, encoding: &str) -> IoResult<String> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => return Err(IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "Failed to find decoder.",
                detail: Some(format!("Invalid decoder: {}", encoding))
            })
        };
        self.stream.lock().read_until(b'\n').and_then(|line|
            match encoding.decode(line[], DecoderTrap::Strict) {
                Ok(data) => Ok(data),
                Err(data) => Err(IoError {
                    kind: IoErrorKind::InvalidInput,
                    desc: "Failed to decode message.",
                    detail: Some(format!("Failed to decode {} as {}.", data, encoding.name())),
                })
            }
        )
    }

    /// Acquires the Stream lock.
    #[experimental]
    pub fn stream<'a>(&'a self) -> MutexGuard<'a, T> {
        self.stream.lock()
    }
}

/// Converts a Result<T, SslError> into an IoResult<T>.
#[cfg(feature = "ssl")]
fn ssl_to_io<T>(res: Result<T, SslError>) -> IoResult<T> {
    match res {
        Ok(x) => Ok(x),
        Err(e) => Err(IoError {
            kind: IoErrorKind::OtherIoError,
            desc: "An SSL error occurred.",
            detail: Some(format!("{}", e)),
        }),
    }
}

/// An abstraction over different networked streams.
#[experimental]
pub enum NetStream {
    /// An unsecured TcpStream.
    UnsecuredTcpStream(TcpStream),
    /// An SSL-secured TcpStream.
    /// This is only available when compiled with SSL support.
    #[cfg(feature = "ssl")]
    SslTcpStream(SslStream<TcpStream>),
}

impl Reader for NetStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        match self {
            &NetStream::UnsecuredTcpStream(ref mut stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            &NetStream::SslTcpStream(ref mut stream) => stream.read(buf),
        }
    }
}

impl Writer for NetStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match self {
            &NetStream::UnsecuredTcpStream(ref mut stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            &NetStream::SslTcpStream(ref mut stream) => stream.write(buf),
        }
    }
}

/// An IrcStream built from an IrcWriter and an IrcReader.
#[experimental]
pub struct IoStream<T: IrcWriter, U: IrcReader> {
    writer: T,
    reader: U,
}

impl<T: IrcWriter, U: IrcReader> IoStream<T, U> {
    /// Creates a new IoStream from the given IrcWriter and IrcReader.
    #[experimental]
    pub fn new(writer: T, reader: U) -> IoStream<T, U> {
        IoStream { writer: writer, reader: reader }
    }
}

impl<U: IrcReader> IoStream<MemWriter, U> {
    pub fn value(&self) -> Vec<u8> {
        self.writer.get_ref().to_vec()
    }
}

impl<T: IrcWriter, U: IrcReader> Buffer for IoStream<T, U> {
    fn fill_buf<'a>(&'a mut self) -> IoResult<&'a [u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: uint) {
        self.reader.consume(amt)
    }
}

impl<T: IrcWriter, U: IrcReader> Reader for IoStream<T, U> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.reader.read(buf)
    }
}

impl<T: IrcWriter, U: IrcReader> Writer for IoStream<T, U> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.writer.write(buf)
    }
}

#[cfg(test)]
mod test {
    use super::{Connection, IoStream};
    use std::io::{MemReader, MemWriter};
    use std::io::util::{NullReader, NullWriter};
    use data::message::Message;
    use encoding::{DecoderTrap, Encoding};
    use encoding::all::ISO_8859_15;

    #[test]
    fn send_utf8() {
        let conn = Connection::new(IoStream::new(MemWriter::new(), NullReader));
        assert!(conn.send(
            Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("€ŠšŽžŒœŸ")), "l9"
        ).is_ok());
        let data = ISO_8859_15.decode(conn.stream().value()[], DecoderTrap::Strict).unwrap();
        assert_eq!(data[], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }

    #[test]
    fn send_iso885915() {

    }

    #[test]
    fn recv_utf8() {
        let conn = Connection::new(IoStream::new(
            NullWriter, MemReader::new(b"PRIVMSG test :Testing!\r\n".to_vec())
        ));
        assert_eq!(conn.recv("UTF-8").unwrap()[], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    fn recv_iso885915() {
        let conn = Connection::new(IoStream::new(
            NullWriter, MemReader::new({
                let mut vec = Vec::new();
                vec.push_all(b"PRIVMSG test :");
                vec.push_all(&[0xA4, 0xA6, 0xA8, 0xB4, 0xB8, 0xBC, 0xBD, 0xBE]);
                vec.push_all(b"\r\n");
                vec
            })
        ));
        assert_eq!(conn.recv("l9").unwrap()[], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }
}
