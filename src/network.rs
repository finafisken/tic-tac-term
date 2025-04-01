use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

use anyhow::anyhow;

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Accepted,
    Rejected,
    Payload,
}

impl From<MessageType> for u8 {
    fn from(mt: MessageType) -> Self {
        match mt {
            MessageType::Accepted => 0,
            MessageType::Rejected => 1,
            MessageType::Payload => 2,
        }
    }
}

impl TryFrom<u8> for MessageType {
    fn try_from(byte: u8) -> anyhow::Result<Self> {
        match byte {
            0 => Ok(MessageType::Accepted),
            1 => Ok(MessageType::Rejected),
            2 => Ok(MessageType::Payload),
            _ => Err(anyhow!("Invalid byte value"))
        }
    }

    type Error = anyhow::Error;
}

#[derive(Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub payload_size: u16,
    pub payload: Vec<u8>
}

impl From<Message> for Vec<u8> {
    fn from(msg: Message) -> Self {
        // first byte is msg type
        // u16 (two bytes) for payload length
        // remaining bytes payload
        let mut bytes: Vec<u8> = vec![msg.message_type.into()];
        bytes.extend(msg.payload_size.to_be_bytes());
        bytes.extend(msg.payload);

        bytes
    }
}

impl TryFrom<&[u8]> for Message {
    fn try_from(bytes: &[u8]) -> anyhow::Result<Self> {
        // TODO some check that length of bytes adds up before conversion
        let message_type: MessageType = bytes[0].try_into()?;

        Ok(Message {
            message_type,
            payload_size: u16::from_be_bytes([bytes[1], bytes[2]]),
            payload: bytes[3..].to_vec()
        })
    }

    type Error = anyhow::Error;
}

#[derive(Debug, PartialEq)]
pub enum NetState {
    Active,
    Waiting,
}

pub fn connect(
    address: &str,
    is_host: bool,
) -> anyhow::Result<(BufReader<TcpStream>, BufWriter<TcpStream>)> {
    let tcp_stream = match is_host {
        true => TcpListener::bind(address)?.accept()?.0,
        false => TcpStream::connect(address)?,
    };

    // https://doc.rust-lang.org/book/ch21-01-single-threaded.html
    // https://github.com/thepacketgeek/rust-tcpstream-demo/blob/master/protocol/README.md
    // println!("{:?}", tcp_stream);

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

    Ok(msg.trim().to_string())
}

pub fn write_stream(stream: &mut BufWriter<TcpStream>, data: Vec<u8>) -> anyhow::Result<()> {
    stream.write_all(&data)?;
    stream.write_all(b"\n")?; // remove when using bytes, only needed for read_line
    stream.flush()?;

    Ok(())
}
