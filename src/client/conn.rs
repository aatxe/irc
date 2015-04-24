//! Thread-safe connections on IrcStreams.
#[cfg(feature = "ssl")] use std::error::Error as StdError;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Result};
use std::io::Error;
use std::io::ErrorKind;
use std::net::TcpStream;
#[cfg(feature = "ssl")] use std::result::Result as StdResult;
use std::sync::{Mutex, MutexGuard};
#[cfg(feature = "encode")] use encoding::{DecoderTrap, EncoderTrap, Encoding};
#[cfg(feature = "encode")] use encoding::label::encoding_from_whatwg_label;
use client::data::kinds::{IrcRead, IrcWrite};
use client::data::message::ToMessage;
#[cfg(feature = "ssl")] use openssl::ssl::{SslContext, SslMethod, SslStream};
#[cfg(feature = "ssl")] use openssl::ssl::error::SslError;

/// A thread-safe connection.
pub struct Connection<T: IrcRead, U: IrcWrite> {
    reader: Mutex<T>,
    writer: Mutex<U>,
}

/// A Connection over a buffered NetStream.
pub type NetConnection = Connection<BufReader<NetStream>, BufWriter<NetStream>>;
/// An internal type
type NetReadWritePair = (BufReader<NetStream>, BufWriter<NetStream>);

impl Connection<BufReader<NetStream>, BufWriter<NetStream>> {
    /// Creates a thread-safe TCP connection to the specified server.
    pub fn connect(host: &str, port: u16) -> Result<NetConnection> {
        let (reader, writer) = try!(Connection::connect_internal(host, port));
        Ok(Connection::new(reader, writer))
    }

    /// connects to the specified server and returns a reader-writer pair.
    fn connect_internal(host: &str, port: u16) -> Result<NetReadWritePair> {
        let socket = try!(TcpStream::connect(&format!("{}:{}", host, port)[..]));
        Ok((BufReader::new(NetStream::UnsecuredTcpStream(try!(socket.try_clone()))),
            BufWriter::new(NetStream::UnsecuredTcpStream(socket))))
    }

    /// Creates a thread-safe TCP connection to the specified server over SSL.
    /// If the library is compiled without SSL support, this method panics.
    pub fn connect_ssl(host: &str, port: u16) -> Result<NetConnection> {
        let (reader, writer) = try!(Connection::connect_ssl_internal(host, port));
        Ok(Connection::new(reader, writer))
    }

    /// Connects over SSL to the specified server and returns a reader-writer pair.
    #[cfg(feature = "ssl")]
    fn connect_ssl_internal(host: &str, port: u16) -> Result<NetReadWritePair> {
        let socket = try!(TcpStream::connect(&format!("{}:{}", host, port)[..]));
        let ssl = try!(ssl_to_io(SslContext::new(SslMethod::Tlsv1)));
        let ssl_socket = try!(ssl_to_io(SslStream::new(&ssl, socket)));
        Ok((BufReader::new(NetStream::SslTcpStream(try!(ssl_socket.try_clone()))),
            BufWriter::new(NetStream::SslTcpStream(ssl_socket))))
    }

    /// Panics because SSL support is not compiled in.
    #[cfg(not(feature = "ssl"))]
    fn connect_ssl_internal(host: &str, port: u16) -> Result<NetReadWritePair> {
        panic!("Cannot connect to {}:{} over SSL without compiling with SSL support.", host, port)
    }

    /// Reconnects to the specified server, dropping the current connection.
    pub fn reconnect(&self, host: &str, port: u16) -> Result<()> {
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

    
    /*
    FIXME: removed until set_keepalive is stabilized.
    /// Sets the keepalive for the network stream.
    #[unstable = "Rust IO has not stabilized."]
    pub fn set_keepalive(&self, delay_in_seconds: Option<u32>) -> Result<()> {
        self.mod_stream(|tcp| tcp.set_keepalive(delay_in_seconds))
    }

    /// Modifies the internal TcpStream using a function.
    fn mod_stream<F>(&self, f: F) -> Result<()> where F: FnOnce(&mut TcpStream) -> Result<()> {
        match self.reader.lock().unwrap().get_mut() {
            &mut NetStream::UnsecuredTcpStream(ref mut tcp) => f(tcp),
            #[cfg(feature = "ssl")]
            &mut NetStream::SslTcpStream(ref mut ssl) => f(ssl.get_mut()),
        }
    }
    */
}

impl<T: IrcRead, U: IrcWrite> Connection<T, U> {
    /// Creates a new connection from an IrcReader and an IrcWriter.
    pub fn new(reader: T, writer: U) -> Connection<T, U> {
        Connection {
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        }
    }

    /// Sends a Message over this connection.
    #[cfg(feature = "encode")]
    pub fn send<M: ToMessage>(&self, to_msg: M, encoding: &str) -> Result<()> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => return Err(Error::new(
                ErrorKind::InvalidInput, &format!("Failed to find encoder. ({})", encoding)[..]
            ))
        };
        let msg = to_msg.to_message();
        let data = match encoding.encode(&msg.into_string(), EncoderTrap::Replace) {
            Ok(data) => data,
            Err(data) => return Err(Error::new(ErrorKind::InvalidInput, 
                &format!("Failed to encode {} as {}.", data, encoding.name())[..]
            ))
        };
        let mut writer = self.writer.lock().unwrap();
        try!(writer.write_all(&data));
        writer.flush()
    }

    /// Sends a message over this connection. 
    #[cfg(not(feature = "encode"))]
    pub fn send<M: ToMessage>(&self, to_msg: M) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        try!(writer.write_all(&to_msg.to_message().into_string().as_bytes()));
        writer.flush()
    }

    /// Receives a single line from this connection.
    #[cfg(feature = "encoding")]
    pub fn recv(&self, encoding: &str) -> Result<String> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => return Err(Error::new(
                ErrorKind::InvalidInput, &format!("Failed to find decoder. ({})", encoding)[..]
            ))
        };
        let mut buf = Vec::new();
        self.reader.lock().unwrap().read_until(b'\n', &mut buf).and_then(|_|
            match encoding.decode(&buf, DecoderTrap::Replace) {
                _ if buf.is_empty() => Err(Error::new(ErrorKind::Other, "EOF")),
                Ok(data) => Ok(data),
                Err(data) => return Err(Error::new(ErrorKind::InvalidInput, 
                    &format!("Failed to decode {} as {}.", data, encoding.name())[..]
                ))
            }
        )
    }

    /// Receives a single line from this connection.
    #[cfg(not(feature = "encoding"))]
    pub fn recv(&self) -> Result<String> {
        let mut ret = String::new();
        try!(self.reader.lock().unwrap().read_line(&mut ret));
        if ret.is_empty() {
            Err(Error::new(ErrorKind::Other, "EOF"))
        } else {
            Ok(ret)
        }
    }

    /// Acquires the Reader lock.
    pub fn reader<'a>(&'a self) -> MutexGuard<'a, T> {
        self.reader.lock().unwrap()
    }

    /// Acquires the Writer lock.
    pub fn writer<'a>(&'a self) -> MutexGuard<'a, U> {
        self.writer.lock().unwrap()
    }
}

/// Converts a Result<T, SslError> into an Result<T>.
#[cfg(feature = "ssl")]
fn ssl_to_io<T>(res: StdResult<T, SslError>) -> Result<T> {
    match res {
        Ok(x) => Ok(x),
        Err(e) => Err(Error::new(ErrorKind::Other, 
            &format!("An SSL error occurred. ({})", e.description())[..]
        )),
    }
}

/// An abstraction over different networked streams.
pub enum NetStream {
    /// An unsecured TcpStream.
    UnsecuredTcpStream(TcpStream),
    /// An SSL-secured TcpStream.
    /// This is only available when compiled with SSL support.
    #[cfg(feature = "ssl")]
    SslTcpStream(SslStream<TcpStream>),
}

impl Read for NetStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            &mut NetStream::UnsecuredTcpStream(ref mut stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            &mut NetStream::SslTcpStream(ref mut stream) => stream.read(buf),
        }
    }
}

impl Write for NetStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            &mut NetStream::UnsecuredTcpStream(ref mut stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            &mut NetStream::SslTcpStream(ref mut stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            &mut NetStream::UnsecuredTcpStream(ref mut stream) => stream.flush(),
            #[cfg(feature = "ssl")]
            &mut NetStream::SslTcpStream(ref mut stream) => stream.flush(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Connection;
    use std::io::{Cursor, sink};
    use client::data::message::Message;
    use client::test::buf_empty;
    #[cfg(feature = "encode")] use encoding::{DecoderTrap, Encoding};
    #[cfg(feature = "encode")] use encoding::all::{ISO_8859_15, UTF_8};

    #[test]
    #[cfg(not(feature = "encode"))]
    fn send() {
        let conn = Connection::new(buf_empty(), Vec::new());
        assert!(conn.send(
            Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("Testing!"))
        ).is_ok());
        let data = String::from_utf8(conn.writer().to_vec()).unwrap();
        assert_eq!(&data[..], "PRIVMSG test :Testing!\r\n");
    }
    
    #[test]
    #[cfg(not(feature = "encode"))]
    fn send_str() {
        let exp = "PRIVMSG test :Testing!\r\n";
        let conn = Connection::new(buf_empty(), Vec::new());
        assert!(conn.send(exp).is_ok());
        let data = String::from_utf8(conn.writer().to_vec()).unwrap();
        assert_eq!(&data[..], exp);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_utf8() {
        let conn = Connection::new(buf_empty(), Vec::new());
        assert!(conn.send(
            Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("€ŠšŽžŒœŸ")), "UTF-8"
        ).is_ok());
        let data = UTF_8.decode(&conn.writer(), DecoderTrap::Strict).unwrap();
        assert_eq!(&data[..], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_utf8_str() {
        let exp = "PRIVMSG test :€ŠšŽžŒœŸ\r\n";
        let conn = Connection::new(buf_empty(), Vec::new());
        assert!(conn.send(exp, "UTF-8").is_ok());
        let data = UTF_8.decode(&conn.writer(), DecoderTrap::Strict).unwrap();
        assert_eq!(&data[..], exp);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_iso885915() {
        let conn = Connection::new(buf_empty(), Vec::new());
        assert!(conn.send(
            Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("€ŠšŽžŒœŸ")), "l9"
        ).is_ok());
        let data = ISO_8859_15.decode(&conn.writer(), DecoderTrap::Strict).unwrap();
        assert_eq!(&data[..], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_iso885915_str() {
        let exp = "PRIVMSG test :€ŠšŽžŒœŸ\r\n";
        let conn = Connection::new(buf_empty(), Vec::new());
        assert!(conn.send(exp, "l9").is_ok());
        let data = ISO_8859_15.decode(&conn.writer(), DecoderTrap::Strict).unwrap();
        assert_eq!(&data[..], exp);
    }

    #[test]
    #[cfg(not(feature = "encode"))]
    fn recv() {
        let conn = Connection::new(
            Cursor::new("PRIVMSG test :Testing!\r\n".as_bytes().to_vec()), sink()
        );
        assert_eq!(&conn.recv().unwrap()[..], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn recv_utf8() {
        let conn = Connection::new(
            Cursor::new(b"PRIVMSG test :Testing!\r\n".to_vec()), sink()
        );
        assert_eq!(&conn.recv("UTF-8").unwrap()[..], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn recv_iso885915() {
        let data = [0xA4, 0xA6, 0xA8, 0xB4, 0xB8, 0xBC, 0xBD, 0xBE];
        let conn = Connection::new(Cursor::new({
            let mut vec = Vec::new();
            vec.extend("PRIVMSG test :".as_bytes());
            vec.extend(data.iter());
            vec.extend("\r\n".as_bytes());
            vec.into_iter().map(|b| *b).collect::<Vec<_>>()
        }), sink());
        assert_eq!(&conn.recv("l9").unwrap()[..], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }
}
