use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
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
            _ => Err(anyhow!("Invalid byte value")),
        }
    }

    type Error = anyhow::Error;
}

#[derive(Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub payload_size: u16,
    pub payload: Vec<u8>,
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

        if message_type != MessageType::Payload {
            return Ok(Message {
                message_type,
                payload_size: 0,
                payload: Vec::new(),
            });
        }

        Ok(Message {
            message_type,
            payload_size: u16::from_be_bytes([bytes[1], bytes[2]]),
            payload: bytes[3..].to_vec(),
        })
    }

    type Error = anyhow::Error;
}

#[derive(Debug, PartialEq)]
pub enum NetState {
    Active,
    Waiting,
}

// https://doc.rust-lang.org/book/ch21-01-single-threaded.html
// https://github.com/thepacketgeek/rust-tcpstream-demo/blob/master/protocol/README.md

pub fn connect(game_id: &str, server_addr: &str) -> anyhow::Result<UdpSocket> {
    let udp_socket = UdpSocket::bind("0.0.0.0:0")?;

    // get opponent ip:port for game_id from server
    let init_msg = format!("GAME###{}", game_id);
    udp_socket.send_to(init_msg.as_bytes(), server_addr)?;
    let mut init_buf = [0u8; 1024];
    let (nr_bytes, _) = udp_socket.recv_from(&mut init_buf)?;
    let opponent_addr = String::from_utf8_lossy(&init_buf[..nr_bytes]).to_string();

    // dedicate socket to opponent_addr
    udp_socket.connect(opponent_addr)?;

    Ok(udp_socket)
}

pub fn read_stream<R: Read>(stream: &mut BufReader<R>) -> anyhow::Result<Message> {
    let mut mt_buf = [0; 1];
    stream.read_exact(&mut mt_buf)?;
    let message_type: MessageType = mt_buf[0].try_into()?;

    if message_type != MessageType::Payload {
        let message: Message = mt_buf.as_slice().try_into()?;
        return Ok(message);
    }

    let mut payload_size_buf = [0; 2];
    stream.read_exact(&mut payload_size_buf)?;

    let payload_size = u16::from_be_bytes(payload_size_buf);

    let mut payload = vec![0; payload_size as usize];

    stream.read_exact(&mut payload)?;

    Ok(Message {
        message_type,
        payload_size,
        payload,
    })
}

pub fn write_stream<W: Write>(stream: &mut BufWriter<W>, data: Vec<u8>) -> anyhow::Result<()> {
    stream.write_all(&data)?;
    stream.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_message_type_conversions() {
        // MessageType -> u8
        assert_eq!(u8::from(MessageType::Accepted), 0);
        assert_eq!(u8::from(MessageType::Rejected), 1);
        assert_eq!(u8::from(MessageType::Payload), 2);

        // u8 -> MessageType
        assert_eq!(MessageType::try_from(0).unwrap(), MessageType::Accepted);
        assert_eq!(MessageType::try_from(1).unwrap(), MessageType::Rejected);
        assert_eq!(MessageType::try_from(2).unwrap(), MessageType::Payload);

        // invalid conversion
        assert!(MessageType::try_from(3).is_err());
    }

    #[test]
    fn test_message_to_bytes() {
        // accepted message (no payload)
        let accept_msg = Message {
            message_type: MessageType::Accepted,
            payload_size: 0,
            payload: Vec::new(),
        };
        let bytes: Vec<u8> = accept_msg.into();
        assert_eq!(bytes, vec![0, 0, 0]);

        // payload message with data
        let payload_msg = Message {
            message_type: MessageType::Payload,
            payload_size: 5,
            payload: vec![10, 20, 30, 40, 50],
        };
        let bytes: Vec<u8> = payload_msg.into();
        assert_eq!(bytes, vec![2, 0, 5, 10, 20, 30, 40, 50]);
    }

    #[test]
    fn test_message_from_bytes() {
        // accepted message
        let bytes = vec![0];
        let msg = Message::try_from(bytes.as_slice()).unwrap();
        assert_eq!(msg.message_type, MessageType::Accepted);
        assert_eq!(msg.payload_size, 0);
        assert!(msg.payload.is_empty());

        // rejected message
        let bytes = vec![1];
        let msg = Message::try_from(bytes.as_slice()).unwrap();
        assert_eq!(msg.message_type, MessageType::Rejected);

        // payload message
        let bytes = vec![2, 0, 3, 65, 66, 67]; // Payload of "ABC"
        let msg = Message::try_from(bytes.as_slice()).unwrap();
        assert_eq!(msg.message_type, MessageType::Payload);
        assert_eq!(msg.payload_size, 3);
        assert_eq!(msg.payload, vec![65, 66, 67]);
    }

    #[test]
    fn test_read_stream() {
        // mock a simple accepted message
        let mock_data = vec![0]; // MessageType::Accepted
        let mut cursor = Cursor::new(mock_data);
        let mut reader = BufReader::new(&mut cursor);

        // test reading non-payload message
        let msg = read_stream(&mut reader).unwrap();
        assert_eq!(msg.message_type, MessageType::Accepted);

        // mock a payload message
        let mock_data = vec![2, 0, 3, 65, 66, 67]; // Payload message with "ABC"
        let mut cursor = Cursor::new(mock_data);
        let mut reader = BufReader::new(&mut cursor);

        // test reading payload message
        let msg = read_stream(&mut reader).unwrap();
        assert_eq!(msg.message_type, MessageType::Payload);
        assert_eq!(msg.payload_size, 3);
        assert_eq!(msg.payload, vec![65, 66, 67]);
    }

    // test write_stream with mock writer
    #[test]
    fn test_write_stream() {
        let mut buffer = Vec::new();

        {
            let mut writer = BufWriter::new(&mut buffer);
            // write a simple message
            let data = vec![0, 0, 0]; // Accepted message
            write_stream(&mut writer, data).unwrap();
            // writer is dropped at the end of this scope
        }

        // Now buffer can be accessed
        assert_eq!(buffer, vec![0, 0, 0]);
    }

    #[test]
    fn test_error_handling() {
        // invalid message type
        let invalid_data = vec![5]; // Invalid message type
        let result = Message::try_from(invalid_data.as_slice());
        assert!(result.is_err());

        // incomplete payload message
        let incomplete_data = vec![2, 0, 5, 1, 2]; // Payload size 5 but only 2 bytes
        let mut cursor = Cursor::new(incomplete_data);
        let mut reader = BufReader::new(&mut cursor);

        let result = read_stream(&mut reader);
        assert!(result.is_err());
    }
}
