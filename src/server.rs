use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use conn::Connection;
use data::command::Command;
use data::config::Config;
use data::kinds::{IrcReader, IrcWriter};
use data::message::Message;

pub trait Server<'a, T, U> {
    fn send(&self, _: Command) -> IoResult<()>;
    fn iter(&'a self) -> ServerIterator<'a, T, U>;
}

pub struct IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub conn: Connection<T, U>,
    pub config: Config
}

impl<'a> IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
     pub fn new(config: &str) -> IoResult<IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let config = try!(Config::load_utf8(config));
        let conn = try!(Connection::connect(config.server[], config.port));
        Ok(IrcServer {
            conn: conn,
            config: config
        })
    }
}

impl<'a, T, U> Server<'a, T, U> for IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader{
    fn send(&self, command: Command) -> IoResult<()> {
        self.conn.send(command.to_message())
    }

    fn iter(&'a self) -> ServerIterator<'a, T, U> {
        ServerIterator::new(self)
    }
}

impl<'a, T, U> IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn from_connection(config: &str, conn: Connection<T, U>) -> IoResult<IrcServer<'a, T, U>> {
        Ok(IrcServer {
            conn: conn,
            config: try!(Config::load_utf8(config))
        })
    }
}

pub struct ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub server: &'a IrcServer<'a, T, U>
}

impl<'a, T, U> ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(server: &'a IrcServer<'a, T, U>) -> ServerIterator<'a, T, U> {
        ServerIterator {
            server: server
        }
    }
}

impl<'a, T, U> Iterator<Message> for ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn next(&mut self) -> Option<Message> {
        let line = self.server.conn.recv();
        if let Err(_) = line { return None }
        from_str(line.unwrap()[])
    }
}
