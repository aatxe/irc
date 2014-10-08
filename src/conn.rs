use std::io::{BufferedWriter, IoResult, TcpStream, Writer};
use data::{IrcReader, IrcWriter, Message};

pub enum Connection<T, U> where T: IrcWriter, U: IrcReader {
    Conn(T, U),
}

impl Connection<BufferedWriter<TcpStream>, TcpStream> {
    pub fn connect(host: &str, port: u16) -> IoResult<Connection<BufferedWriter<TcpStream>, TcpStream>> {
        let socket = try!(TcpStream::connect(host, port));
        Ok(Conn(BufferedWriter::new(socket.clone()), socket.clone()))
    }
}

impl<T, U> Connection<T, U> where T: IrcWriter, U: IrcReader {
    fn send_internal(conn: &mut Connection<T, U>, msg: &str) -> IoResult<()> {
        match conn {
            &Conn(ref mut send, _) => {
                try!(send.write_str(msg));
                send.flush()
            }
        }
    }

    pub fn send(conn: &mut Connection<T, U>, msg: Message) -> IoResult<()> {
        let mut send = msg.command.to_string();
        send.push_str(" ");
        send.push_str(msg.args.init().connect(" ").as_slice());
        send.push_str(" :");
        send.push_str(*msg.args.last().unwrap());
        send.push_str("\r\n");
        Connection::send_internal(conn, send.as_slice())
    }
}
