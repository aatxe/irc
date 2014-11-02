use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use {Bot, process};
use conn::Connection;
use data::{Config, IrcReader, IrcWriter, Message, User};

pub struct IrcBot<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub conn: Connection<T, U>,
    pub config: Config,
    process: RefCell<|&IrcBot<T, U>, &str, &str, &[&str]|:'a -> IoResult<()>>,
    chanlists: RefCell<HashMap<String, Vec<User>>>,
}

impl<'a> IrcBot<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    pub fn new(process: |&IrcBot<BufferedWriter<TcpStream>, BufferedReader<TcpStream>>, &str, &str, &[&str]|:'a -> IoResult<()>) -> IoResult<IrcBot<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let config = try!(Config::load());
        let conn = try!(Connection::connect(config.server[], config.port));
        Ok(IrcBot {
            conn: conn,
            config: config,
            process: RefCell::new(process),
            chanlists: RefCell::new(HashMap::new()),
        })
    }

    pub fn new_with_config(config: Config, process: |&IrcBot<BufferedWriter<TcpStream>, BufferedReader<TcpStream>>, &str, &str, &[&str]|:'a -> IoResult<()>) -> IoResult<IrcBot<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let conn = try!(Connection::connect(config.server[], config.port));
        Ok(IrcBot {
            conn: conn,
            config: config,
            process: RefCell::new(process),
            chanlists: RefCell::new(HashMap::new()),
        })
    }
}

impl<'a, T, U> Bot for IrcBot<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn send_sanick(&self, old_nick: &str, new_nick: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "SANICK", [old_nick, new_nick], false))
    }

    fn send_nick(&self, nick: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "NICK", [nick], true))
    }

    fn send_user(&self, username: &str, real_name: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "USER", [username, "0", "*", real_name], true))
    }

    fn send_join(&self, chan: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "JOIN", [chan], true))
    }

    fn send_samode(&self, target: &str, mode: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "SAMODE", [target, mode], false))
    }

    fn send_mode(&self, target: &str, mode: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "MODE", [target, mode], false))
    }

    fn send_oper(&self, name: &str, password: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "OPER", [name, password], false))
    }

    fn send_topic(&self, chan: &str, topic: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "TOPIC", [chan, topic], true))
    }

    fn send_invite(&self, person: &str, chan: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "INVITE", [person, chan], true))
    }

    fn send_kick(&self, chan: &str, user: &str, msg: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "KICK", [chan, user, msg], true))
    }

    fn send_kill(&self, nick: &str, msg: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "KILL", [nick, msg], true))
    }

    fn send_privmsg(&self, chan: &str, msg: &str) -> IoResult<()> {
        for line in msg.split_str("\r\n") {
            try!(self.conn.send(Message::new(None, "PRIVMSG", [chan, line], true)));
        }
        Ok(())
    }

    fn identify(&self) -> IoResult<()> {
        try!(self.send_nick(self.config.nickname[]));
        self.send_user(self.config.username[], self.config.realname[])
    }

    fn output(&mut self) -> IoResult<()> {
        let mut reader = self.conn.reader();
        for line in reader.lines() {
            match line {
                Ok(ln) => {
                    let (source, command, args) = try!(process(ln[]));
                    try!(self.handle_command(source, command, args[]));
                    println!("{}", ln)
                },
                Err(e) => {
                    println!("{}", e);
                    return Err(e)
                },
            }
        }
        Ok(())
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn get_users(&self, chan: &str) -> Option<Vec<User>> {
        self.chanlists.borrow_mut().find_copy(&chan.into_string())
    }
}

impl<'a, T, U> IrcBot<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn from_connection(conn: Connection<T, U>, process: |&IrcBot<T, U>, &str, &str, &[&str]|:'a -> IoResult<()>) -> IoResult<IrcBot<'a, T, U>> {
        Ok(IrcBot {
            conn: conn,
            config: try!(Config::load()),
            process: RefCell::new(process),
            chanlists: RefCell::new(HashMap::new()),
        })
    }

    fn handle_command(&self, source: &str, command: &str, args: &[&str]) -> IoResult<()> {
        match (command, args) {
            ("PING", [msg]) => {
                try!(self.conn.send(Message::new(None, "PONG", [msg], true)));
            },
            ("376", _) => { // End of MOTD
                for chan in self.config.channels.iter() {
                    try!(self.send_join(chan[]));
                }
            },
            ("422", _) => { // Missing MOTD
                for chan in self.config.channels.iter() {
                    try!(self.send_join(chan[]));
                }
            },
            ("353", [_, _, chan, users]) => { // /NAMES
                for user in users.split_str(" ") {
                    if !match self.chanlists.borrow_mut().find_mut(&String::from_str(chan)) {
                        Some(vec) => { vec.push(User::new(user)); true },
                        None => false,
                    } {
                        self.chanlists.borrow_mut().insert(chan.into_string(), vec!(User::new(user)));
                    }
                }
            },
            ("JOIN", [chan]) => {
                if let Some(vec) = self.chanlists.borrow_mut().find_mut(&String::from_str(chan)) {
                    if let Some(i) = source.find('!') {
                        vec.push(User::new(source[..i]));
                    }
                }
            },
            ("PART", [chan, _]) => {
                if let Some(vec) = self.chanlists.borrow_mut().find_mut(&String::from_str(chan)) {
                    if let Some(i) = source.find('!') {
                        if let Some(n) = vec.as_slice().position_elem(&User::new(source[..i])) {
                            vec.swap_remove(n);
                        }
                    }
                }
            },
            _ => (),
        };
        (*self.process.borrow_mut().deref_mut())(self, source, command, args)
    }
}

#[cfg(test)]
mod test {
    use Bot;
    use super::IrcBot;
    use std::io::{BufReader, MemWriter};
    use std::io::util::{NullReader, NullWriter};
    use conn::Connection;
    use data::{IrcReader, User};

    fn data<U>(conn: Connection<MemWriter, U>) -> String where U: IrcReader {
        String::from_utf8(conn.writer().deref_mut().get_ref().to_vec()).unwrap()
    }

    #[test]
    fn from_connection() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        assert!(IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).is_ok());
    }

    #[test]
    fn send_sanick() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_sanick("test", "test2").unwrap();
        assert_eq!(data(b.conn), format!("SANICK test test2\r\n"));
    }

    #[test]
    fn send_nick() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_nick("test").unwrap();
        assert_eq!(data(b.conn), format!("NICK :test\r\n"));
    }

    #[test]
    fn send_user() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_user("test", "Test").unwrap();
        assert_eq!(data(b.conn), format!("USER test 0 * :Test\r\n"));
    }

    #[test]
    fn send_join() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_join("#test").unwrap();
        assert_eq!(data(b.conn), format!("JOIN :#test\r\n"));
    }

    #[test]
    fn send_samode() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_samode("#test", "+i").unwrap();
        assert_eq!(data(b.conn), format!("SAMODE #test +i\r\n"));
    }

    #[test]
    fn send_mode() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_mode("#test", "+i").unwrap();
        assert_eq!(data(b.conn), format!("MODE #test +i\r\n"));
    }

    #[test]
    fn send_oper() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_oper("test", "test").unwrap();
        assert_eq!(data(b.conn), format!("OPER test test\r\n"));
    }

    #[test]
    fn send_topic() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_topic("#test", "This is a test topic.").unwrap();
        assert_eq!(data(b.conn), format!("TOPIC #test :This is a test topic.\r\n"));
    }

    #[test]
    fn send_invite() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_invite("test2", "#test").unwrap();
        assert_eq!(data(b.conn), format!("INVITE test2 :#test\r\n"));
    }

    #[test]
    fn send_kick() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_kick("#test", "test2", "Goodbye.").unwrap();
        assert_eq!(data(b.conn), format!("KICK #test test2 :Goodbye.\r\n"));
    }

    #[test]
    fn send_kill() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_kill("test", "Goodbye.").unwrap();
        assert_eq!(data(b.conn), format!("KILL test :Goodbye.\r\n"));
    }

    #[test]
    fn send_privmsg() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_privmsg("#test", "This is a test message.").unwrap();
        assert_eq!(data(b.conn), format!("PRIVMSG #test :This is a test message.\r\n"));
    }

    #[test]
    fn send_privmsg_multiline() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.send_privmsg("#test", "This is a test message.\r\nIt has two lines.").unwrap();
        let mut exp = format!("PRIVMSG #test :This is a test message.\r\n");
        exp.push_str("PRIVMSG #test :It has two lines.\r\n");
        assert_eq!(data(b.conn), format!("{}", exp));

    }

    #[test]
    fn identify() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.identify().unwrap();
        assert_eq!(data(b.conn), format!("NICK :test\r\nUSER test 0 * :test\r\n"));
    }

    #[test]
    fn ping_response() {
        let r = BufReader::new(":embyr.tx.us.fyrechat.net PING :01R6\r\n".as_bytes());
        let c = Connection::new(MemWriter::new(), r).unwrap();
        let mut b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.output().unwrap();
        assert_eq!(data(b.conn), format!("PONG :01R6\r\n"));
    }

    #[test]
    fn end_of_motd_response() {
        let r = BufReader::new(":embyr.tx.us.fyrechat.net 376 test :End of /MOTD command.\r\n".as_bytes());
        let c = Connection::new(MemWriter::new(), r).unwrap();
        let mut b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.output().unwrap();
        assert_eq!(data(b.conn), format!("JOIN :#test\r\nJOIN :#test2\r\n"));
    }

    #[test]
    fn missing_motd_response() {
        let r = BufReader::new(":flare.to.ca.fyrechat.net 422 pickles :MOTD File is missing\r\n".as_bytes());
        let c = Connection::new(MemWriter::new(), r).unwrap();
        let mut b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.output().unwrap();
        assert_eq!(data(b.conn), format!("JOIN :#test\r\nJOIN :#test2\r\n"));
    }

    #[test]
    fn generate_user_list() {
        let r = BufReader::new(":flare.to.ca.fyrechat.net 353 test @ #test :test test2 test3\r\n".as_bytes());
        let c = Connection::new(NullWriter, r).unwrap();
        let mut b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.output().unwrap();
        let vec_res = match b.chanlists.borrow_mut().find_mut(&String::from_str("#test")) {
                Some(v) => Ok(v.clone()),
                None => Err("Could not find vec for channel."),
        };
        assert!(vec_res.is_ok());
        let vec = vec_res.unwrap();
        assert_eq!(vec, vec![User::new("test"), User::new("test2"), User::new("test3")]);
    }

    #[test]
    fn add_to_user_list() {
        let r = BufReader::new(":flare.to.ca.fyrechat.net 353 test @ #test :test test2\r\n:test3!test@test JOIN :#test\r\n".as_bytes());
        let c = Connection::new(NullWriter, r).unwrap();
        let mut b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.output().unwrap();
        let vec_res = match b.chanlists.borrow_mut().find_mut(&String::from_str("#test")) {
                Some(v) => Ok(v.clone()),
                None => Err("Could not find vec for channel."),
        };
        assert!(vec_res.is_ok());
        let vec = vec_res.unwrap();
        assert_eq!(vec, vec![User::new("test"), User::new("test2"), User::new("test3")]);
    }

    #[test]
    fn remove_from_user_list() {
        let r = BufReader::new(":flare.to.ca.fyrechat.net 353 test @ #test :test test2 test3\r\n:test3!test@test PART #test :\r\n".as_bytes());
        let c = Connection::new(NullWriter, r).unwrap();
        let mut b = IrcBot::from_connection(c, |_, _, _, _| { Ok(()) }).unwrap();
        b.output().unwrap();
        let vec_res = match b.chanlists.borrow_mut().find_mut(&String::from_str("#test")) {
                Some(v) => Ok(v.clone()),
                None => Err("Could not find vec for channel."),
        };
        assert!(vec_res.is_ok());
        let vec = vec_res.unwrap();
        // n.b. ordering is not guaranteed, this only ought to hold because we're removing the last user
        assert_eq!(vec, vec![User::new("test"), User::new("test2")]);
    }
}
