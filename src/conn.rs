//! Thread-safe connections on any IrcWriters and IrcReaders
#![experimental]
use std::sync::Mutex;
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use data::kinds::{IrcWriter, IrcReader};
use data::message::Message;

/// A thread-safe connection
#[experimental]
pub struct Connection<T, U> where T: IrcWriter, U: IrcReader {
    writer: Mutex<T>,
    reader: Mutex<U>,
}

impl Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    /// Creates a thread-safe TCP connection to the specified server
    #[experimental]
    pub fn connect(host: &str, port: u16) -> IoResult<Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let socket = try!(TcpStream::connect(host, port));
        Ok(Connection::new(BufferedWriter::new(socket.clone()), BufferedReader::new(socket)))
    }
}

impl<T, U> Connection<T, U> where T: IrcWriter, U: IrcReader {
    /// Creates a new connection from any arbitrary IrcWriter and IrcReader
    #[experimental]
    pub fn new(writer: T, reader: U) -> Connection<T, U> {
        Connection {
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
        }
    }

    /// Sends a Message over this connection
    #[experimental]
    pub fn send(&self, message: Message) -> IoResult<()> {
        let mut send = self.writer.lock();
        try!(send.write_str(message.into_string()[]));
        send.flush()
    }

    /// Receives a single line from this connection
    #[experimental]
    pub fn recv(&self) -> IoResult<String> {
        self.reader.lock().read_line()
    }
}
