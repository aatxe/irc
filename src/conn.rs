//! Thread-safe connections on any IrcWriters and IrcReaders.
#![experimental]
use std::sync::{Mutex, MutexGuard};
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
#[cfg(feature = "ssl")] use std::io::{IoError, OtherIoError};
use data::kinds::{IrcWriter, IrcReader};
use data::message::Message;
#[cfg(feature = "ssl")] use openssl::ssl::{SslContext, SslStream, Tlsv1};
#[cfg(feature = "ssl")] use openssl::ssl::error::SslError;

/// A thread-safe connection.
#[experimental]
pub struct Connection<T, U> where T: IrcWriter, U: IrcReader {
    writer: Mutex<T>,
    reader: Mutex<U>,
}

impl Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    /// Creates a thread-safe TCP connection to the specified server.
    #[experimental]
    pub fn connect(host: &str, port: u16) -> IoResult<Connection<BufferedWriter<NetStream>, BufferedReader<NetStream>>> {
        let socket = try!(TcpStream::connect(format!("{}:{}", host, port)[]));
        Ok(Connection::new(BufferedWriter::new(UnsecuredTcpStream(socket.clone())),
                           BufferedReader::new(UnsecuredTcpStream(socket))))
    }

    /// Creates a thread-safe TCP connection to the specified server over SSL.
    /// If the library is compiled without SSL support, this method panics.
    #[experimental]
    #[cfg(feature = "ssl")]
    pub fn connect_ssl(host: &str, port: u16) -> IoResult<Connection<BufferedWriter<NetStream>, BufferedReader<NetStream>>> {
        let socket = try!(TcpStream::connect(format!("{}:{}", host, port)[]));
        let ssl = try!(ssl_to_io(SslContext::new(Tlsv1)));
        let input = try!(ssl_to_io(SslStream::new(&ssl, socket.clone())));
        let output = try!(ssl_to_io(SslStream::new(&ssl, socket)));
        Ok(Connection::new(BufferedWriter::new(SslTcpStream(input)),
                           BufferedReader::new(SslTcpStream(output))))
    }

    /// Creates a thread-safe TCP connection to the specified server over SSL.
    /// If the library is compiled without SSL support, this method panics.
    #[experimental]
    #[cfg(not(feature = "ssl"))]
    pub fn connect_ssl(host: &str, port: u16) -> IoResult<Connection<BufferedWriter<NetStream>, BufferedReader<NetStream>>> {
        panic!("Cannot connect to {}:{} over SSL without compiling with SSL support.", host, port)
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
            &UnsecuredTcpStream(ref mut stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            &SslTcpStream(ref mut stream) => stream.read(buf),
        }
    }
}

impl Writer for NetStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match self {
            &UnsecuredTcpStream(ref mut stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            &SslTcpStream(ref mut stream) => stream.write(buf),
        }
    }
}

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

impl<T, U> Connection<T, U> where T: IrcWriter, U: IrcReader {
    /// Creates a new connection from any arbitrary IrcWriter and IrcReader.
    #[experimental]
    pub fn new(writer: T, reader: U) -> Connection<T, U> {
        Connection {
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
        }
    }

    /// Sends a Message over this connection.
    #[experimental]
    pub fn send(&self, message: Message) -> IoResult<()> {
        let mut send = self.writer.lock();
        try!(send.write_str(message.into_string()[]));
        send.flush()
    }

    /// Receives a single line from this connection.
    #[experimental]
    pub fn recv(&self) -> IoResult<String> {
        self.reader.lock().read_line()
    }

    /// Acquires the Writer lock.
    #[experimental]
    pub fn writer<'a>(&'a self) -> MutexGuard<'a, T> {
        self.writer.lock()
    }
}

#[cfg(test)]
mod test {
    use super::Connection;
    use std::io::{MemReader, MemWriter};
    use std::io::util::{NullReader, NullWriter};
    use data::message::Message;

    #[test]
    fn send() {
        let conn = Connection::new(MemWriter::new(), NullReader);
        assert!(conn.send(Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("Testing!"))).is_ok());
        let data = String::from_utf8(conn.writer().get_ref().to_vec()).unwrap();
        assert_eq!(data[], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    fn recv() {
        let conn = Connection::new(NullWriter, MemReader::new("PRIVMSG test :Testing!\r\n".as_bytes().to_vec()));
        assert_eq!(conn.recv().unwrap()[], "PRIVMSG test :Testing!\r\n");
    }
}
