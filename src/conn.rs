//! Thread-safe connections on IrcStreams.
#![experimental]
use std::sync::{Mutex, MutexGuard};
use std::io::{BufferedStream, IoResult, MemWriter, TcpStream};
#[cfg(feature = "ssl")] use std::io::{IoError, OtherIoError};
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
    #[cfg(feature = "ssl")]
    pub fn connect_ssl(host: &str, port: u16) -> IoResult<Connection<BufferedStream<NetStream>>> {
        Connection::connect_ssl_internal(host, port, None)
    }

    #[experimental]
    #[cfg(feature = "ssl")]
    pub fn connect_ssl_with_timeout(host: &str, port: u16, timeout_ms: u64)
        -> IoResult<Connection<BufferedStream<NetStream>>> {
        Connection::connect_ssl_internal(host, port, Some(timeout_ms))
    }

    /// Creates a thread-safe TCP connection to the specified server over SSL.
    /// If the library is compiled without SSL support, this method panics.
    #[experimental]
    #[cfg(not(feature = "ssl"))]
    pub fn connect_ssl(host: &str, port: u16) -> IoResult<Connection<BufferedStream<NetStream>>> {
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

/// Converts a Result<T, SslError> into an IoResult<T>.
#[cfg(feature = "ssl")]
fn ssl_to_io<T>(res: Result<T, SslError>) -> IoResult<T> {
    match res {
        Ok(x) => Ok(x),
        Err(e) => Err(IoError {
            kind: OtherIoError,
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
    pub fn send(&self, message: Message) -> IoResult<()> {
        let mut stream = self.stream.lock();
        try!(stream.write_str(message.into_string()[]));
        stream.flush()
    }

    /// Receives a single line from this connection.
    #[experimental]
    pub fn recv(&self) -> IoResult<String> {
        self.stream.lock().read_line()
    }

    /// Acquires the Stream lock.
    #[experimental]
    pub fn stream<'a>(&'a self) -> MutexGuard<'a, T> {
        self.stream.lock()
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

    #[test]
    fn send() {
        let conn = Connection::new(IoStream::new(MemWriter::new(), NullReader));
        assert!(conn.send(
            Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("Testing!"))
        ).is_ok());
        let data = String::from_utf8(conn.stream().value()).unwrap();
        assert_eq!(data[], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    fn recv() {
        let conn = Connection::new(IoStream::new(
            NullWriter, MemReader::new("PRIVMSG test :Testing!\r\n".as_bytes().to_vec())
        ));
        assert_eq!(conn.recv().unwrap()[], "PRIVMSG test :Testing!\r\n");
    }
}
