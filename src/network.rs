use std::{io::{BufRead, BufReader, BufWriter, Read, Write}, net::{TcpListener, TcpStream}};

pub enum MessageType {
    Connected,
    Disconnected,
    Accepted,
    Rejected,
    Payload
}

pub struct Payload {

}

pub fn connect(address: &str, is_host: bool) -> anyhow::Result<(BufReader<TcpStream>, BufWriter<TcpStream>)> {
    
    let tcp_stream = match is_host {
        true => TcpListener::bind(address)?.accept()?.0,
        false => TcpStream::connect(address)?,
    };

    // https://doc.rust-lang.org/book/ch21-01-single-threaded.html
    // https://github.com/thepacketgeek/rust-tcpstream-demo/blob/master/protocol/README.md
    println!("{:?}", tcp_stream);

    let reader = BufReader::new(tcp_stream.try_clone()?);
    let writer = BufWriter::new(tcp_stream);

    Ok((reader, writer))
}

pub fn read_stream(stream: &mut BufReader<TcpStream>) -> anyhow::Result<String> {
    // let mut buf = [0; 1];
    // stream.read_exact(&mut buf)?;
    // let msg = String::from_utf8(buf.to_vec())?;

    let mut msg: String = String::default();
    stream.read_line(&mut msg)?;

    // println!("######## {}", msg);

    Ok(msg.trim().to_string())
}

pub fn write_stream(stream: &mut BufWriter<TcpStream>, data: Vec<u8>) -> anyhow::Result<()> {
    stream.write_all(&data)?;
    stream.write_all(b"\n")?; // remove when using bytes, only needed for read_line
    stream.flush()?;

    Ok(())
}
