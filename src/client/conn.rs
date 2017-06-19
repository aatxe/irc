//! Thread-safe connections on `IrcStreams`.
#[cfg(feature = "ssl")]
use std::error::Error as StdError;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Cursor, Result};
#[cfg(feature = "ssl")]
use std::io::Error;
#[cfg(feature = "ssl")]
use std::io::ErrorKind;
use std::net::TcpStream;
#[cfg(feature = "ssl")]
use std::result::Result as StdResult;
use std::sync::Mutex;
#[cfg(feature = "encode")]
use encoding::DecoderTrap;
#[cfg(feature = "encode")]
use encoding::label::encoding_from_whatwg_label;
#[cfg(feature = "ssl")]
use openssl::ssl::{SslContext, SslMethod, SslStream};
#[cfg(feature = "ssl")]
use openssl::ssl::error::SslError;

/// A connection.
pub trait Connection {
    /// Sends a message over this connection.
    #[cfg(feature = "encode")]
    fn send(&self, msg: &str, encoding: &str) -> Result<()>;

    /// Sends a message over this connection.
    #[cfg(not(feature = "encode"))]
    fn send(&self, msg: &str) -> Result<()>;

    /// Receives a single line from this connection.
    #[cfg(feature = "encoding")]
    fn recv(&self, encoding: &str) -> Result<String>;

    /// Receives a single line from this connection.
    #[cfg(not(feature = "encoding"))]
    fn recv(&self) -> Result<String>;

    /// Gets the full record of all sent messages if the Connection records this.
    /// This is intended for use in writing tests.
    #[cfg(feature = "encoding")]
    fn written(&self, encoding: &str) -> Option<String>;

    /// Gets the full record of all sent messages if the Connection records this.
    /// This is intended for use in writing tests.
    #[cfg(not(feature = "encoding"))]
    fn written(&self) -> Option<String>;

    /// Re-establishes this connection, disconnecting from the existing case if necessary.
    fn reconnect(&self) -> Result<()>;
}


/// Useful internal type definitions.
type NetReader = BufReader<NetStream>;
type NetWriter = BufWriter<NetStream>;
type NetReadWritePair = (NetReader, NetWriter);

/// A thread-safe connection over a buffered `NetStream`.
pub struct NetConnection {
    host: Mutex<String>,
    port: Mutex<u16>,
    reader: Mutex<NetReader>,
    writer: Mutex<NetWriter>,
}

impl NetConnection {
    fn new(host: &str, port: u16, reader: NetReader, writer: NetWriter) -> NetConnection {
        NetConnection {
            host: Mutex::new(host.to_owned()),
            port: Mutex::new(port),
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        }
    }

    /// Creates a thread-safe TCP connection to the specified server.
    pub fn connect(host: &str, port: u16) -> Result<NetConnection> {
        let (reader, writer) = try!(NetConnection::connect_internal(host, port));
        Ok(NetConnection::new(host, port, reader, writer))
    }

    /// connects to the specified server and returns a reader-writer pair.
    fn connect_internal(host: &str, port: u16) -> Result<NetReadWritePair> {
        let socket = try!(TcpStream::connect(&format!("{}:{}", host, port)[..]));
        Ok((
            BufReader::new(
                NetStream::Unsecured(try!(socket.try_clone())),
            ),
            BufWriter::new(NetStream::Unsecured(socket)),
        ))
    }

    /// Creates a thread-safe TCP connection to the specified server over SSL.
    /// If the library is compiled without SSL support, this method panics.
    pub fn connect_ssl(host: &str, port: u16) -> Result<NetConnection> {
        let (reader, writer) = try!(NetConnection::connect_ssl_internal(host, port));
        Ok(NetConnection::new(host, port, reader, writer))
    }

    /// Connects over SSL to the specified server and returns a reader-writer pair.
    #[cfg(feature = "ssl")]
    fn connect_ssl_internal(host: &str, port: u16) -> Result<NetReadWritePair> {
        let socket = try!(TcpStream::connect(&format!("{}:{}", host, port)[..]));
        let ssl = try!(ssl_to_io(SslContext::new(SslMethod::Sslv23)));
        let ssl_socket = try!(ssl_to_io(SslStream::connect_generic(&ssl, socket)));
        Ok((
            BufReader::new(NetStream::Ssl(try!(ssl_socket.try_clone()))),
            BufWriter::new(NetStream::Ssl(ssl_socket)),
        ))
    }

    /// Panics because SSL support is not compiled in.
    #[cfg(not(feature = "ssl"))]
    fn connect_ssl_internal(host: &str, port: u16) -> Result<NetReadWritePair> {
        panic!(
            "Cannot connect to {}:{} over SSL without compiling with SSL support.",
            host,
            port
        )
    }
}

/// Converts a `Result<T, SslError>` into an `io::Result<T>`.
#[cfg(feature = "ssl")]
fn ssl_to_io<T>(res: StdResult<T, SslError>) -> Result<T> {
    match res {
        Ok(x) => Ok(x),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            &format!("An SSL error occurred. ({})", e.description())[..],
        )),
    }
}

impl Connection for NetConnection {
    #[cfg(feature = "encode")]
    fn send(&self, msg: &str, encoding: &str) -> Result<()> {
        imp::send(&self.writer, msg, encoding)
    }

    #[cfg(not(feature = "encode"))]
    fn send(&self, msg: &str) -> Result<()> {
        imp::send(&self.writer, msg)
    }

    #[cfg(feature = "encoding")]
    fn recv(&self, encoding: &str) -> Result<String> {
        imp::recv(&self.reader, encoding)
    }

    #[cfg(not(feature = "encoding"))]
    fn recv(&self) -> Result<String> {
        imp::recv(&self.reader)
    }

    #[cfg(feature = "encoding")]
    fn written(&self, _: &str) -> Option<String> {
        None
    }

    #[cfg(not(feature = "encoding"))]
    fn written(&self) -> Option<String> {
        None
    }

    fn reconnect(&self) -> Result<()> {
        let use_ssl = match *self.reader.lock().unwrap().get_ref() {
            NetStream::Unsecured(_) => false,
            #[cfg(feature = "ssl")]
            NetStream::Ssl(_) => true,
        };
        let host = self.host.lock().unwrap();
        let port = self.port.lock().unwrap();
        let (reader, writer) = if use_ssl {
            try!(NetConnection::connect_ssl_internal(&host, *port))
        } else {
            try!(NetConnection::connect_internal(&host, *port))
        };
        *self.reader.lock().unwrap() = reader;
        *self.writer.lock().unwrap() = writer;
        Ok(())
    }
}

/// A mock connection for testing purposes.
pub struct MockConnection {
    reader: Mutex<Cursor<Vec<u8>>>,
    writer: Mutex<Vec<u8>>,
}

impl MockConnection {
    /// Creates a new mock connection with an empty read buffer.
    pub fn empty() -> MockConnection {
        MockConnection::from_byte_vec(Vec::new())
    }

    /// Creates a new mock connection with the specified string in the read buffer.
    pub fn new(input: &str) -> MockConnection {
        MockConnection::from_byte_vec(input.as_bytes().to_vec())
    }

    /// Creates a new mock connection with the specified bytes in the read buffer.
    pub fn from_byte_vec(input: Vec<u8>) -> MockConnection {
        MockConnection {
            reader: Mutex::new(Cursor::new(input)),
            writer: Mutex::new(Vec::new()),
        }
    }
}

impl Connection for MockConnection {
    #[cfg(feature = "encode")]
    fn send(&self, msg: &str, encoding: &str) -> Result<()> {
        imp::send(&self.writer, msg, encoding)
    }

    #[cfg(not(feature = "encode"))]
    fn send(&self, msg: &str) -> Result<()> {
        imp::send(&self.writer, msg)
    }

    #[cfg(feature = "encoding")]
    fn recv(&self, encoding: &str) -> Result<String> {
        imp::recv(&self.reader, encoding)
    }

    #[cfg(not(feature = "encoding"))]
    fn recv(&self) -> Result<String> {
        imp::recv(&self.reader)
    }

    #[cfg(feature = "encoding")]
    fn written(&self, encoding: &str) -> Option<String> {
        encoding_from_whatwg_label(encoding).and_then(|enc| {
            enc.decode(&self.writer.lock().unwrap(), DecoderTrap::Replace)
                .ok()
        })
    }

    #[cfg(not(feature = "encoding"))]
    fn written(&self) -> Option<String> {
        String::from_utf8(self.writer.lock().unwrap().clone()).ok()
    }

    fn reconnect(&self) -> Result<()> {
        Ok(())
    }
}

mod imp {
    use std::io::Result;
    use std::io::Error;
    use std::io::ErrorKind;
    use std::sync::Mutex;
    #[cfg(feature = "encode")]
    use encoding::{DecoderTrap, EncoderTrap};
    #[cfg(feature = "encode")]
    use encoding::label::encoding_from_whatwg_label;
    use client::data::kinds::{IrcRead, IrcWrite};

    #[cfg(feature = "encode")]
    pub fn send<T: IrcWrite>(writer: &Mutex<T>, msg: &str, encoding: &str) -> Result<()> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    &format!("Failed to find encoder. ({})", encoding)[..],
                ))
            }
        };
        let data = match encoding.encode(msg, EncoderTrap::Replace) {
            Ok(data) => data,
            Err(data) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    &format!(
                        "Failed to encode {} as {}.",
                        data,
                        encoding.name()
                    )
                        [..],
                ))
            }
        };
        let mut writer = writer.lock().unwrap();
        try!(writer.write_all(&data));
        writer.flush()
    }

    #[cfg(not(feature = "encode"))]
    pub fn send<T: IrcWrite>(writer: &Mutex<T>, msg: &str) -> Result<()> {
        let mut writer = writer.lock().unwrap();
        try!(writer.write_all(msg.as_bytes()));
        writer.flush()
    }

    #[cfg(feature = "encoding")]
    pub fn recv<T: IrcRead>(reader: &Mutex<T>, encoding: &str) -> Result<String> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    &format!("Failed to find decoder. ({})", encoding)[..],
                ))
            }
        };
        let mut buf = Vec::new();
        reader
            .lock()
            .unwrap()
            .read_until(b'\n', &mut buf)
            .and_then(|_| match encoding.decode(&buf, DecoderTrap::Replace) {
                _ if buf.is_empty() => Err(Error::new(ErrorKind::Other, "EOF")),
                Ok(data) => Ok(data),
                Err(data) => Err(Error::new(
                    ErrorKind::InvalidInput,
                    &format!(
                        "Failed to decode {} as {}.",
                        data,
                        encoding.name()
                    )
                        [..],
                )),
            })
    }

    #[cfg(not(feature = "encoding"))]
    pub fn recv<T: IrcRead>(reader: &Mutex<T>) -> Result<String> {
        let mut ret = String::new();
        try!(reader.lock().unwrap().read_line(&mut ret));
        if ret.is_empty() {
            Err(Error::new(ErrorKind::Other, "EOF"))
        } else {
            Ok(ret)
        }
    }
}

/// An abstraction over different networked streams.
pub enum NetStream {
    /// An unsecured TcpStream.
    Unsecured(TcpStream),
    /// An SSL-secured TcpStream.
    /// This is only available when compiled with SSL support.
    #[cfg(feature = "ssl")]
    Ssl(SslStream<TcpStream>),
}

impl Read for NetStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match *self {
            NetStream::Unsecured(ref mut stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            NetStream::Ssl(ref mut stream) => stream.read(buf),
        }
    }
}

impl Write for NetStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match *self {
            NetStream::Unsecured(ref mut stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            NetStream::Ssl(ref mut stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match *self {
            NetStream::Unsecured(ref mut stream) => stream.flush(),
            #[cfg(feature = "ssl")]
            NetStream::Ssl(ref mut stream) => stream.flush(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Connection, MockConnection};
    use std::io::Result;
    use client::data::Message;
    use client::data::Command::PRIVMSG;

    #[cfg(feature = "encode")]
    fn send_to<C: Connection, M: Into<Message>>(conn: &C, msg: M, encoding: &str) -> Result<()> {
        conn.send(&msg.into().to_string(), encoding)
    }

    #[cfg(not(feature = "encode"))]
    fn send_to<C: Connection, M: Into<Message>>(conn: &C, msg: M) -> Result<()> {
        conn.send(&msg.into().to_string())
    }


    #[test]
    #[cfg(not(feature = "encode"))]
    fn send() {
        let conn = MockConnection::empty();
        assert!(send_to(&conn, PRIVMSG("test".to_owned(), "Testing!".to_owned())).is_ok());
        let data = conn.written().unwrap();
        assert_eq!(&data[..], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    #[cfg(not(feature = "encode"))]
    fn send_str() {
        let exp = "PRIVMSG test :Testing!\r\n";
        let conn = MockConnection::empty();
        assert!(send_to(&conn, exp).is_ok());
        let data = conn.written().unwrap();
        assert_eq!(&data[..], exp);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_utf8() {
        let conn = MockConnection::empty();
        assert!(
            send_to(
                &conn,
                PRIVMSG("test".to_owned(), "€ŠšŽžŒœŸ".to_owned()),
                "UTF-8",
            ).is_ok()
        );
        let data = conn.written("UTF-8").unwrap();
        assert_eq!(&data[..], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_utf8_str() {
        let exp = "PRIVMSG test :€ŠšŽžŒœŸ\r\n";
        let conn = MockConnection::empty();
        assert!(send_to(&conn, exp, "UTF-8").is_ok());
        let data = conn.written("UTF-8").unwrap();
        assert_eq!(&data[..], exp);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_iso885915() {
        let conn = MockConnection::empty();
        assert!(
            send_to(
                &conn,
                PRIVMSG("test".to_owned(), "€ŠšŽžŒœŸ".to_owned()),
                "l9",
            ).is_ok()
        );
        let data = conn.written("l9").unwrap();
        assert_eq!(&data[..], "PRIVMSG test :€ŠšŽžŒœŸ\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn send_iso885915_str() {
        let exp = "PRIVMSG test :€ŠšŽžŒœŸ\r\n";
        let conn = MockConnection::empty();
        assert!(send_to(&conn, exp, "l9").is_ok());
        let data = conn.written("l9").unwrap();
        assert_eq!(&data[..], exp);
    }

    #[test]
    #[cfg(not(feature = "encode"))]
    fn recv() {
        let conn = MockConnection::new("PRIVMSG test :Testing!\r\n");
        assert_eq!(&conn.recv().unwrap()[..], "PRIVMSG test :Testing!\r\n");
    }

    #[test]
    #[cfg(feature = "encode")]
    fn recv_utf8() {
        let conn = MockConnection::new("PRIVMSG test :Testing!\r\n");
        assert_eq!(
            &conn.recv("UTF-8").unwrap()[..],
            "PRIVMSG test :Testing!\r\n"
        );
    }

    #[test]
    #[cfg(feature = "encode")]
    fn recv_iso885915() {
        let data = [0xA4, 0xA6, 0xA8, 0xB4, 0xB8, 0xBC, 0xBD, 0xBE];
        let conn = MockConnection::from_byte_vec({
            let mut vec = Vec::new();
            vec.extend("PRIVMSG test :".as_bytes());
            vec.extend(data.iter());
            vec.extend("\r\n".as_bytes());
            vec.into_iter().collect::<Vec<_>>()
        });
        assert_eq!(
            &conn.recv("l9").unwrap()[..],
            "PRIVMSG test :€ŠšŽžŒœŸ\r\n"
        );
    }
}
