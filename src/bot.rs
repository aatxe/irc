use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use {Server, process};
use conn::Connection;
use data::{Config, IrcReader, IrcWriter, Message, User};

pub struct IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub conn: Connection<T, U>,
    pub config: Config,
    chanlists: RefCell<HashMap<String, Vec<User>>>,
}

impl<'a> IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    pub fn new() -> IoResult<IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let config = try!(Config::load_utf8("config.json"));
        let conn = try!(Connection::connect(config.server[], config.port));
        Ok(IrcServer {
            conn: conn,
            config: config,
            chanlists: RefCell::new(HashMap::new()),
        })
    }

    pub fn new_with_config(config: Config) -> IoResult<IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let conn = try!(Connection::connect(config.server[], config.port));
        Ok(IrcServer {
            conn: conn,
            config: config,
            chanlists: RefCell::new(HashMap::new()),
        })
    }
}

impl<'a, T, U> Iterator<Message> for IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn next(&mut self) -> Option<Message> {
        let line_res = self.conn.reader().read_line();
        if let Err(e) = line_res { println!("{}", e); return None; }
        let line = line_res.unwrap();
        let processed = process(line[]);
        if let Err(e) = processed { println!("{}", e); return None; }
        let (source, command, args) = processed.unwrap();
        Some(Message::new(Some(source), command, args, None))
    }
}

impl<'a, T, U> Server<'a> for IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn send(&self, message: Message) -> IoResult<()> {
        self.conn.send(message)
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn get_users(&self, chan: &str) -> Option<Vec<User>> {
        self.chanlists.borrow_mut().find_copy(&chan.into_string())
    }
}

impl<'a, T, U> IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn from_connection(conn: Connection<T, U>) -> IoResult<IrcServer<'a, T, U>> {
        Ok(IrcServer {
            conn: conn,
            config: try!(Config::load_utf8("config.json")),
            chanlists: RefCell::new(HashMap::new()),
        })
    }
}
