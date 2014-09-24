use std::io::{BufferedWriter, IoResult, TcpStream};
use data::Message;

pub struct Connection(pub TcpStream);

pub fn connect(host: &str, port: u16) -> IoResult<Connection> {
    let socket = try!(TcpStream::connect(host, port));
    Ok(Connection(socket))
}

fn send_internal(conn: &Connection, msg: &str) -> IoResult<()> {
    let &Connection(ref tcp) = conn;
    let mut writer = BufferedWriter::new(tcp.clone());
    writer.write_str(msg);
    writer.flush()
}

pub fn send(conn: &Connection, msg: Message) -> IoResult<()> {
    let arg_string = msg.args.init().connect(" ").append(" :").append(*msg.args.last().unwrap());
    send_internal(conn, msg.command.to_string().append(" ").append(arg_string.as_slice()).append("\r\n").as_slice())
}
