use conn::Connection;
use data::kinds::{IrcReader, IrcWriter};
use data::message::Message;

pub struct IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub conn: Connection<T, U>
}

pub struct ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub server: &'a IrcServer<'a, T, U>
}

impl<'a, T, U> Iterator<Message> for ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn next(&mut self) -> Option<Message> {
        let line = self.server.conn.recv();
        if let Err(_) = line { return None }
        from_str(line.unwrap()[])
    }
}
