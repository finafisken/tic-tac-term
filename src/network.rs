use anyhow::anyhow;
use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Accepted,
    Rejected,
    Payload,
    Handshake,
    HandshakeAck,
}

impl From<MessageType> for u8 {
    fn from(mt: MessageType) -> Self {
        match mt {
            MessageType::Accepted => 0,
            MessageType::Rejected => 1,
            MessageType::Payload => 2,
            MessageType::Handshake => 3,
            MessageType::HandshakeAck => 4,
        }
    }
}

impl TryFrom<u8> for MessageType {
    fn try_from(byte: u8) -> anyhow::Result<Self> {
        match byte {
            0 => Ok(MessageType::Accepted),
            1 => Ok(MessageType::Rejected),
            2 => Ok(MessageType::Payload),
            3 => Ok(MessageType::Handshake),
            4 => Ok(MessageType::HandshakeAck),
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
        if bytes.is_empty() {
            return Err(anyhow!("Empty bytes array"));
        }

        let message_type: MessageType = bytes[0].try_into()?;

        if message_type != MessageType::Payload {
            return Ok(Message {
                message_type,
                payload_size: 0,
                payload: Vec::new(),
            });
        }

        if bytes.len() < 3 {
            return Err(anyhow!("Insufficient bytes for payload message header"));
        }

        let payload_size = u16::from_be_bytes([bytes[1], bytes[2]]);

        if bytes.len() < 3 + payload_size as usize {
            return Err(anyhow!(
                "Incomplete payload: expected {} bytes, got {}",
                payload_size,
                bytes.len()
            ));
        }

        Ok(Message {
            message_type,
            payload_size,
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

pub fn connect(game_id: &str, server_addr: &str, is_host: bool) -> anyhow::Result<UdpSocket> {
    let udp_socket = UdpSocket::bind("0.0.0.0:0")?;

    // get opponent ip:port for game_id from server
    let init_msg = format!("GAME###{}", game_id);
    udp_socket.send_to(init_msg.as_bytes(), server_addr)?;
    let mut init_buf = [0u8; 1024];
    let (nr_bytes, _) = udp_socket.recv_from(&mut init_buf)?;
    let recieved = String::from_utf8_lossy(&init_buf[..nr_bytes]).to_string();
    println!("#### {} ishost: {}", recieved, is_host);
    let opponent_addr: SocketAddr = recieved.trim().parse()?;

    // dedicate socket to opponent_addr
    udp_socket.connect(opponent_addr)?;

    perform_handshake(&udp_socket, is_host)?;

    // connection established, shorten timeouts
    udp_socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    udp_socket.set_write_timeout(Some(Duration::from_millis(100)))?;

    Ok(udp_socket)
}

pub fn read(socket: &UdpSocket) -> anyhow::Result<Message> {
    println!("# READ peer: {}", socket.peer_addr()?);
    let mut buf = [0u8; 1024];
    let bytes_recieved = socket.recv(&mut buf)?;

    Message::try_from(&buf[..bytes_recieved])
}

pub fn write(socket: &UdpSocket, msg: Message) -> anyhow::Result<()> {
    println!("# WRITE peer: {}", socket.peer_addr()?);
    let data: Vec<u8> = msg.into();
    socket.send(&data)?;

    Ok(())
}

fn perform_handshake(socket: &UdpSocket, is_host: bool) -> anyhow::Result<()> {
    socket.set_read_timeout(Some(Duration::from_millis(2000)))?;

    for attempt in 1..=5 {
        if is_host {
            write(socket, Message {
                message_type: MessageType::Handshake,
                payload_size: 0,
                payload: Vec::new(),
            })?;
        }

        match read(socket) {
            Ok(res) => match res.message_type {
                MessageType::Handshake => {
                    println!("Handshake recieved from: {}", socket.peer_addr()?);

                    // send acknowledgment
                    write(
                        socket,
                        Message {
                            message_type: MessageType::HandshakeAck,
                            payload_size: 0,
                            payload: Vec::new(),
                        },
                    )?;

                    return Ok(());
                }
                MessageType::HandshakeAck => return Ok(()),
                _ => println!("Unexpected message during handshake"),
            },
            Err(e) => {
                println!("Failed handshake attempt {} with: {}", attempt, e);
                std::thread::sleep(Duration::from_millis(1000));
            }
        }
    }

    Err(anyhow!("Handshake timed out"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_conversions() {
        // MessageType -> u8
        assert_eq!(u8::from(MessageType::Accepted), 0);
        assert_eq!(u8::from(MessageType::Rejected), 1);
        assert_eq!(u8::from(MessageType::Payload), 2);
        assert_eq!(u8::from(MessageType::Handshake), 3);
        assert_eq!(u8::from(MessageType::HandshakeAck), 4);

        // u8 -> MessageType
        assert_eq!(MessageType::try_from(0).unwrap(), MessageType::Accepted);
        assert_eq!(MessageType::try_from(1).unwrap(), MessageType::Rejected);
        assert_eq!(MessageType::try_from(2).unwrap(), MessageType::Payload);
        assert_eq!(MessageType::try_from(3).unwrap(), MessageType::Handshake);
        assert_eq!(MessageType::try_from(4).unwrap(), MessageType::HandshakeAck);

        // invalid conversion
        assert!(MessageType::try_from(5).is_err());
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
    fn test_error_handling() {
        // invalid message type
        let invalid_data = vec![5]; // Invalid message type
        let result = Message::try_from(invalid_data.as_slice());
        assert!(result.is_err());

        // incomplete payload message
        let incomplete_data = vec![2, 0, 5, 1, 2]; // Payload size 5 but only 2 bytes

        let result = Message::try_from(incomplete_data.as_slice());
        assert!(result.is_err());
    }
}
