use std::sync::Mutex;
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use data::kinds::{IrcWriter, IrcReader};
use data::message::Message;

pub struct Connection<T, U> where T: IrcWriter, U: IrcReader {
    writer: Mutex<T>,
    reader: Mutex<U>,
}

impl Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    pub fn connect(host: &str, port: u16) -> IoResult<Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let socket = try!(TcpStream::connect(host, port));
        Ok(Connection::new(BufferedWriter::new(socket.clone()), BufferedReader::new(socket)))
    }
}

impl<T, U> Connection<T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(writer: T, reader: U) -> Connection<T, U> {
        Connection {
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
        }
    }

    pub fn send(&self, message: Message) -> IoResult<()> {
        let mut send = self.writer.lock();
        try!(send.write_str(message.into_string()[]));
        send.flush()
    }

    pub fn recv(&self) -> IoResult<String> {
        self.reader.lock().read_line()
    }
}
