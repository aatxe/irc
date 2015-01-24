//! Thread-safe connections on IrcStreams.
#![stable]
#[cfg(feature = "ssl")] use std::borrow::ToOwned;
#[cfg(feature = "ssl")] use std::error::Error;
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
#[cfg(any(feature = "encode", feature = "ssl"))] use std::io::{IoError, IoErrorKind};
use std::sync::{Mutex, MutexGuard};
#[cfg(feature = "encode")] use encoding::{DecoderTrap, EncoderTrap, Encoding};
#[cfg(feature = "encode")] use encoding::label::encoding_from_whatwg_label;
use client::data::kinds::{IrcReader, IrcWriter};
use client::data::message::ToMessage;
#[cfg(feature = "ssl")] use openssl::ssl::{SslContext, SslMethod, SslStream};
#[cfg(feature = "ssl")] use openssl::ssl::error::SslError;

/// A thread-safe connection.
#[stable]
pub struct Connection<T: IrcReader, U: IrcWriter> {
    reader: Mutex<T>,
    writer: Mutex<U>,
}

/// A Connection over a buffered NetStream.
#[stable]
pub type NetConnection = Connection<BufferedReader<NetStream>, BufferedWriter<NetStream>>;
/// An internal type
type NetReaderWriterPair = (BufferedReader<NetStream>, BufferedWriter<NetStream>);

#[stable]
impl Connection<BufferedReader<NetStream>, BufferedWriter<NetStream>> {
    /// Creates a thread-safe TCP connection to the specified server.
    #[stable]
    pub fn connect(host: &str, port: u16) -> IoResult<NetConnection> {
        let (reader, writer) = try!(Connection::connect_internal(host, port));
        Ok(Connection::new(reader, writer))
    }

    /// connects to the specified server and returns a reader-writer pair.
    fn connect_internal(host: &str, port: u16) -> IoResult<NetReaderWriterPair> {
        let socket = try!(TcpStream::connect(&format!("{}:{}", host, port)[]));
        Ok((BufferedReader::new(NetStream::UnsecuredTcpStream(socket.clone())),
            BufferedWriter::new(NetStream::UnsecuredTcpStream(socket))))
    }

    /// Creates a thread-safe TCP connection to the specified server over SSL.
    /// If the library is compiled without SSL support, this method panics.
    #[stable]
    pub fn connect_ssl(host: &str, port: u16) -> IoResult<NetConnection> {
        let (reader, writer) = try!(Connection::connect_ssl_internal(host, port));
        Ok(Connection::new(reader, writer))
    }

    /// Connects over SSL to the specified server and returns a reader-writer pair.
    #[cfg(feature = "ssl")]
    fn connect_ssl_internal(host: &str, port: u16) -> IoResult<NetReaderWriterPair> {
        let socket = try!(TcpStream::connect(&format!("{}:{}", host, port)[]));
        let ssl = try!(ssl_to_io(SslContext::new(SslMethod::Tlsv1)));
        let ssl_socket = try!(ssl_to_io(SslStream::new(&ssl, socket)));
        Ok((BufferedReader::new(NetStream::SslTcpStream(ssl_socket.clone())),
            BufferedWriter::new(NetStream::SslTcpStream(ssl_socket))))
    }

    /// Panics because SSL support is not compiled in.
    #[cfg(not(feature = "ssl"))]
    fn connect_ssl_internal(host: &str, port: u16) -> IoResult<NetReaderWriterPair> {
        panic!("Cannot connect to {}:{} over SSL without compiling with SSL support.", host, port)
    }

    /// Reconnects to the specified server, dropping the current connection.
    #[unstable = "Feature is relatively new."]
    pub fn reconnect(&self, host: &str, port: u16) -> IoResult<()> {
        let use_ssl = match self.reader.lock().unwrap().get_ref() {
            &NetStream::UnsecuredTcpStream(_) =>  false,
            #[cfg(feature = "ssl")]
            &NetStream::SslTcpStream(_) => true,
        };
        let (reader, writer) = if use_ssl {
            try!(Connection::connect_ssl_internal(host, port))
        } else {
            try!(Connection::connect_internal(host, port))
        };
        *self.reader.lock().unwrap() = reader;
        *self.writer.lock().unwrap() = writer;
        Ok(())
    }

    /// Sets the keepalive for the network stream.
    #[unstable = "Feature is relatively new."]
    pub fn set_keepalive(&self, delay_in_seconds: Option<usize>) -> IoResult<()> {
        self.mod_stream(|tcp| tcp.set_keepalive(delay_in_seconds))
    }

    /// Sets the timeout for the network stream.
    #[unstable = "Feature is relatively new."]
    pub fn set_timeout(&self, timeout_ms: Option<u64>) {
        self.mod_stream(|tcp| Ok(tcp.set_timeout(timeout_ms))).unwrap(); // this cannot fail.
    }

    /// Modifies the internal TcpStream using a function.
    fn mod_stream<F>(&self, f: F) -> IoResult<()> where F: FnOnce(&mut TcpStream) -> IoResult<()> {
        match self.reader.lock().unwrap().get_mut() {
            &mut NetStream::UnsecuredTcpStream(ref mut tcp) => f(tcp),
            #[cfg(feature = "ssl")]
            &mut NetStream::SslTcpStream(ref mut ssl) => f(ssl.get_mut()),
        }
    }
}

#[stable]
impl<T: IrcReader, U: IrcWriter> Connection<T, U> {
    /// Creates a new connection from an IrcReader and an IrcWriter.
    #[stable]
    pub fn new(reader: T, writer: U) -> Connection<T, U> {
        Connection {
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        }
    }

    /// Sends a Message over this connection.
    #[experimental = "Design is very new."]
    #[cfg(feature = "encode")]
    pub fn send<M: ToMessage>(&self, to_msg: M, encoding: &str) -> IoResult<()> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => return Err(IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "Failed to find decoder.",
                detail: Some(format!("Invalid decoder: {}", encoding))
            })
        };
        let msg = to_msg.to_message();
        let data = match encoding.encode(&msg.into_string()[], EncoderTrap::Replace) {
            Ok(data) => data,
            Err(data) => return Err(IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "Failed to decode message.",
                detail: Some(format!("Failed to decode {} as {}.", data, encoding.name())),
            })
        };
        let mut writer = self.writer.lock().unwrap();
        try!(writer.write(&data[]));
        writer.flush()
    }

    /// Sends a message over this connection. 
    #[experimental = "Design is very new."]
    #[cfg(not(feature = "encode"))]
    pub fn send<M: ToMessage>(&self, to_msg: M) -> IoResult<()> {
        let mut writer = self.writer.lock().unwrap();
        try!(writer.write_str(&to_msg.to_message().into_string()[]));
        writer.flush()
    }

    /// Receives a single line from this connection.
    #[stable]
    #[cfg(feature = "encoding")]
    pub fn recv(&self, encoding: &str) -> IoResult<String> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => return Err(IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "Failed to find decoder.",
                detail: Some(format!("Invalid decoder: {}", encoding))
            })
        };
        self.reader.lock().unwrap().read_until(b'\n').and_then(|line|
            match encoding.decode(&line[], DecoderTrap::Replace) {
                Ok(data) => Ok(data),
                Err(data) => Err(IoError {
                    kind: IoErrorKind::InvalidInput,
                    desc: "Failed to decode message.",
                    detail: Some(format!("Failed to decode {} as {}.", data, encoding.name())),
                })
            }
        )
    }

    /// Receives a single line from this connection.
    #[stable]
    #[cfg(not(feature = "encoding"))]
    pub fn recv(&self) -> IoResult<String> {
        self.reader.lock().unwrap().read_line()
    }

    /// Acquires the Reader lock.
    #[stable]
    pub fn reader<'a>(&'a self) -> MutexGuard<'a, T> {
        self.reader.lock().unwrap()
    }

    /// Acquires the Writer lock.
    #[stable]
    pub fn writer<'a>(&'a self) -> MutexGuard<'a, U> {
        self.writer.lock().unwrap()
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
            detail: Some(e.description().to_owned()),
        }),
    }
}

/// An abstraction over different networked streams.
#[stable]
pub enum NetStream {
    /// An unsecured TcpStream.
    #[stable]
    UnsecuredTcpStream(TcpStream),
    /// An SSL-secured TcpStream.
    /// This is only available when compiled with SSL support.
    #[cfg(feature = "ssl")]
    #[stable]
    SslTcpStream(SslStream<TcpStream>),
}

impl Reader for NetStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match self {
            &mut NetStream::UnsecuredTcpStream(ref mut stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            &mut NetStream::SslTcpStream(ref mut stream) => stream.read(buf),
        }
    }
}

impl Writer for NetStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match self {
            &mut NetStream::UnsecuredTcpStream(ref mut stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            &mut NetStream::SslTcpStream(ref mut stream) => stream.write(buf),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Connection;
    use std::io::{MemReader, MemWriter};
    use std::io::util::{NullReader, NullWriter};
    use client::data::message::Message;
    #[cfg(feature = "encode")] use encoding::{DecoderTrap, Encoding};
    #[cfg(feature = "encode")] use encoding::all::{ISO_8859_15, UTF_8};

    #[test]
    #[cfg(not(feature = "encode"))]
    fn send() {
        let conn = Connection::new(NullReader, MemWriter::new());
        assert!(conn.send(
            Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("Testing!"))
        ).is_ok());
        let data = String::from_utf8(conn.writer().get_ref().to_vec()).unwrap();
        assert_eq!(&data[], "PRIVMSG test :Testing!\r\n");
    }
    
    #[test]
    #[cfg(not(feature = "encode"))]
    fn send_str() {
        let exp = "PRIVMSG test :Testing!\r\n";
        let conn = Connection::new(NullReader, MemWriter::new());
        assert!(conn.send(exp).is_ok());
        let data = String::from_utf8(conn.writer().get_ref().to_vec()).unwrap();
        assert_eq!(&data[], exp);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_utf8() {
        let conn = Connection::new(NullReader, MemWriter::new());
        assert!(conn.send(
            Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("€ŠšŽžŒœŸ")), "UTF-8"
        ).is_ok());
        let data = UTF_8.decode(conn.writer().get_ref(), DecoderTrap::Strict).unwrap();
        assert_eq!(&data[], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_utf8_str() {
        let exp = "PRIVMSG test :€ŠšŽžŒœŸ\r\n";
        let conn = Connection::new(NullReader, MemWriter::new());
        assert!(conn.send(exp, "UTF-8").is_ok());
        let data = UTF_8.decode(conn.writer().get_ref(), DecoderTrap::Strict).unwrap();
        assert_eq!(&data[], exp);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_iso885915() {
        let conn = Connection::new(NullReader, MemWriter::new());
        assert!(conn.send(
            Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("€ŠšŽžŒœŸ")), "l9"
        ).is_ok());
        let data = ISO_8859_15.decode(conn.writer().get_ref(), DecoderTrap::Strict).unwrap();
        assert_eq!(&data[], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_iso885915_str() {
        let exp = "PRIVMSG test :€ŠšŽžŒœŸ\r\n";
        let conn = Connection::new(NullReader, MemWriter::new());
        assert!(conn.send(exp, "l9").is_ok());
        let data = ISO_8859_15.decode(conn.writer().get_ref(), DecoderTrap::Strict).unwrap();
        assert_eq!(&data[], exp);
    }

    #[test]
    #[cfg(not(feature = "encode"))]
    fn recv() {
        let conn = Connection::new(
            MemReader::new("PRIVMSG test :Testing!\r\n".as_bytes().to_vec()), NullWriter
        );
        assert_eq!(&conn.recv().unwrap()[], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn recv_utf8() {
        let conn = Connection::new(
            MemReader::new(b"PRIVMSG test :Testing!\r\n".to_vec()), NullWriter
        );
        assert_eq!(&conn.recv("UTF-8").unwrap()[], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn recv_iso885915() {
        let conn = Connection::new(
            MemReader::new({
                let mut vec = Vec::new();
                vec.push_all(b"PRIVMSG test :");
                vec.push_all(&[0xA4, 0xA6, 0xA8, 0xB4, 0xB8, 0xBC, 0xBD, 0xBE]);
                vec.push_all(b"\r\n");
                vec
            }), NullWriter
        );
        assert_eq!(&conn.recv("l9").unwrap()[], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }
}
