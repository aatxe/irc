use std::cell::{RefCell, RefMut};
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream, Writer};
use data::{IrcReader, IrcWriter, Message};

pub struct Connection<T, U> where T: IrcWriter, U: IrcReader {
    writer: RefCell<T>,
    reader: RefCell<U>,
}

impl Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    pub fn connect(host: &str, port: u16) -> IoResult<Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let socket = try!(TcpStream::connect(host, port));
        Connection::new(BufferedWriter::new(socket.clone()), BufferedReader::new(socket.clone()))
    }
}

impl<T, U> Connection<T, U> where T: IrcWriter, U: IrcReader {
    fn new(writer: T, reader: U) -> IoResult<Connection<T, U>> {
        Ok(Connection {
            writer: RefCell::new(writer),
            reader: RefCell::new(reader),
        })
    }

    fn send_internal(&self, msg: &str) -> IoResult<()> {
        let mut send = self.writer.borrow_mut();
        try!(send.write_str(msg));
        send.flush()
    }

    pub fn send(&self, msg: Message) -> IoResult<()> {
        let mut send = msg.command.to_string();
        send.push_str(" ");
        send.push_str(msg.args.init().connect(" ").as_slice());
        send.push_str(" :");
        send.push_str(*msg.args.last().unwrap());
        send.push_str("\r\n");
        self.send_internal(send.as_slice())
    }

    pub fn reader<'a>(&'a self) -> RefMut<'a, U> {
        self.reader.borrow_mut()
    }
}
