//! Thread-safe connections on `IrcStreams`.
use std::io;
use std::io::prelude::*;
use std::io::Cursor;
use std::net::TcpStream;
use std::sync::Mutex;
use error::Result;
use bufstream::BufStream;
use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;

/// A connection.
pub trait Connection {
    /// Sends a message over this connection.
    fn send(&self, msg: &str, encoding: &str) -> Result<()>;

    /// Receives a single line from this connection.
    fn recv(&self, encoding: &str) -> Result<String>;

    /// Gets the full record of all sent messages if the Connection records this.
    /// This is intended for use in writing tests.
    fn written(&self, encoding: &str) -> Option<String>;

    /// Re-establishes this connection, disconnecting from the existing case if necessary.
    fn reconnect(&self) -> Result<()>;
}

/// Useful internal type definitions.
type NetBufStream = BufStream<NetStream>;

/// A thread-safe connection over a buffered `NetStream`.
pub struct NetConnection {
    host: Mutex<String>,
    port: Mutex<u16>,
    stream: Mutex<NetBufStream>,
}

impl NetConnection {
    fn new(host: &str, port: u16, stream: NetBufStream) -> NetConnection {
        NetConnection {
            host: Mutex::new(host.to_owned()),
            port: Mutex::new(port),
            stream: Mutex::new(stream),
        }
    }

    /// Creates a thread-safe TCP connection to the specified server.
    pub fn connect(host: &str, port: u16) -> Result<NetConnection> {
        let stream = try!(NetConnection::connect_internal(host, port));
        Ok(NetConnection::new(host, port, stream))
    }

    /// connects to the specified server and returns a reader-writer pair.
    fn connect_internal(host: &str, port: u16) -> Result<NetBufStream> {
        let socket = try!(TcpStream::connect((host, port)).into());
        Ok(BufStream::new(NetStream::Unsecured(socket)))
    }

    /// Creates a thread-safe TCP connection to the specified server over SSL.
    /// If the library is compiled without SSL support, this method panics.
    pub fn connect_ssl(host: &str, port: u16) -> Result<NetConnection> {
        let stream = try!(NetConnection::connect_ssl_internal(host, port));
        Ok(NetConnection::new(host, port, stream))
    }

    /// Panics because SSL support is not compiled in.
    fn connect_ssl_internal(host: &str, port: u16) -> Result<NetBufStream> {
        panic!("Cannot connect to {}:{} over SSL without compiling with SSL support.", host, port)
    }
}

impl Connection for NetConnection {
    fn send(&self, msg: &str, encoding: &str) -> Result<()> {
        imp::send(&self.stream, msg, encoding)
    }

    fn recv(&self, encoding: &str) -> Result<String> {
        imp::recv(&self.stream, encoding)
    }

    fn written(&self, _: &str) -> Option<String> {
        None
    }

    fn reconnect(&self) -> Result<()> {
        let use_ssl = match *self.stream.lock().unwrap().get_ref() {
            NetStream::Unsecured(_) =>  false,
        };
        let host = self.host.lock().unwrap();
        let port = self.port.lock().unwrap();
        let stream = if use_ssl {
            try!(NetConnection::connect_ssl_internal(&host, *port))
        } else {
            try!(NetConnection::connect_internal(&host, *port))
        };
        *self.stream.lock().unwrap() = stream;
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
    fn send(&self, msg: &str, encoding: &str) -> Result<()> {
        imp::send(&self.writer, msg, encoding)
    }

    fn recv(&self, encoding: &str) -> Result<String> {
        imp::recv(&self.reader, encoding)
    }

    fn written(&self, encoding: &str) -> Option<String> {
        encoding_from_whatwg_label(encoding).and_then(|enc| {
            enc.decode(&self.writer.lock().unwrap(), DecoderTrap::Replace)
                .ok()
        })
    }

    fn reconnect(&self) -> Result<()> {
        Ok(())
    }
}

mod imp {
    use std::io::{Error, ErrorKind};
    use std::sync::Mutex;
    use encoding::{DecoderTrap, EncoderTrap};
    use encoding::label::encoding_from_whatwg_label;
    use error::Result;
    use client::data::kinds::{IrcRead, IrcWrite};

    pub fn send<T: IrcWrite>(writer: &Mutex<T>, msg: &str, encoding: &str) -> Result<()> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    &format!("Failed to find encoder. ({})", encoding)[..],
                ).into())
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
                ).into())
            }
        };
        let mut writer = writer.lock().unwrap();
        try!(writer.write_all(&data));
        writer.flush().map_err(|e| e.into())
    }

    pub fn recv<T: IrcRead>(reader: &Mutex<T>, encoding: &str) -> Result<String> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    &format!("Failed to find decoder. ({})", encoding)[..],
                ).into())
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
            .map_err(|e| e.into())
    }

}

/// An abstraction over different networked streams.
pub enum NetStream {
    /// An unsecured TcpStream.
    Unsecured(TcpStream),
}

impl Read for NetStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            NetStream::Unsecured(ref mut stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            NetStream::Ssl(ref mut stream) => stream.read(buf),
        }
    }
}

impl Write for NetStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            NetStream::Unsecured(ref mut stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            NetStream::Unsecured(ref mut stream) => stream.flush(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Connection, MockConnection};
    use error::Result;
    use client::data::Message;
    use client::data::Command::PRIVMSG;

    fn send_to<C: Connection, M: Into<Message>>(conn: &C, msg: M, encoding: &str) -> Result<()> {
        conn.send(&msg.into().to_string(), encoding)
    }

    #[test]
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
    fn send_utf8_str() {
        let exp = "PRIVMSG test :€ŠšŽžŒœŸ\r\n";
        let conn = MockConnection::empty();
        assert!(send_to(&conn, exp, "UTF-8").is_ok());
        let data = conn.written("UTF-8").unwrap();
        assert_eq!(&data[..], exp);
    }

    #[test]
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
    fn send_iso885915_str() {
        let exp = "PRIVMSG test :€ŠšŽžŒœŸ\r\n";
        let conn = MockConnection::empty();
        assert!(send_to(&conn, exp, "l9").is_ok());
        let data = conn.written("l9").unwrap();
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn recv_utf8() {
        let conn = MockConnection::new("PRIVMSG test :Testing!\r\n");
        assert_eq!(
            &conn.recv("UTF-8").unwrap()[..],
            "PRIVMSG test :Testing!\r\n"
        );
    }

    #[test]
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
