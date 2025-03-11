use std::{io::{BufReader, BufWriter}, net::{TcpListener, TcpStream}};

pub enum MessageType {
    Connected,
    Disconnected,
    Accepted,
    Rejected,
    Payload
}

pub struct Payload {

}

pub fn connect(address: &str) -> anyhow::Result<(BufReader<TcpStream>, BufWriter<TcpStream>)> {
    let (tcp_stream, socket_addr) = TcpListener::bind(address)?.accept()?;

    // https://doc.rust-lang.org/book/ch21-01-single-threaded.html
    // https://github.com/thepacketgeek/rust-tcpstream-demo/blob/master/protocol/README.md
    println!("{:?} ## {:?}", tcp_stream, socket_addr);

    let reader = BufReader::new(tcp_stream.try_clone()?);
    let writer = BufWriter::new(tcp_stream);

    Ok((reader, writer))
}
