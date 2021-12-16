use crate::data::{Message, Packet};

use hex_literal::hex;
use std::io::{stdin, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::{thread, usize};
pub struct Client {
    uuid: String,
}

impl Client {
    pub fn new(uuid: String) -> Self {
        Client { uuid }
    }

    pub fn run(&self) {
        let tmp_uuid = self.uuid.clone();

        thread::spawn(move || Client::setup_receive_stream(tmp_uuid));

        println!("client:");
        let end_client = get_user_input();

        loop {
            let msg = get_user_input();

            /* if msg != "--end--".to_string() {
                break;
            } */
            let key = hex!("603DEB1015CA71BE2B73AEF0857D77811F352C073B6108D72D9810A30914DFF4");

            let message = Message::new(self.uuid.clone(), end_client.clone(), msg);
            let mut packet = Packet::new(
                String::from(""),
                end_client.clone(),
                String::from("send_message"),
                message.to_string().unwrap(),
            );
            packet.encrypt_data(&key);
            self.new_message(packet);
        }
    }

    fn new_message(&self, packet: Packet) {
        match TcpStream::connect("localhost:3333") {
            Ok(mut stream) => {
                send_message(&mut stream, packet.to_vec().unwrap());
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }

    fn setup_receive_stream(uuid: String) {
        match TcpStream::connect("localhost:3333") {
            Ok(mut stream) => {
                println!("Successfully connected to server in port 3333");

                let packet = Packet::new(
                    String::from(uuid),
                    String::from(""),
                    String::from("setup_receiver_stream"),
                    String::from(""),
                );
                stream.write(&packet.to_vec().unwrap()).unwrap();

                loop {
                    match Packet::from_vec(read_message(&mut stream)) {
                        Ok(mut packet) => {
                            let key = hex!(
                                "603DEB1015CA71BE2B73AEF0857D77811F352C073B6108D72D9810A30914DFF4"
                            );

                            packet.decrypt_data(&key);

                            match Message::from_string(packet.data) {
                                Ok(msg) => {
                                    if msg.message.len() == 0 {
                                        println!("Break");
                                        break;
                                    } else {
                                        println!(
                                            "{:?}: {:?}",
                                            msg.sender_uid,
                                            msg.message
                                        );
                                    }
                                }
                                _=>{
                                    println!("Could not read message !");
                                }
                            }
                        }
                        _ => {
                            println!("Could not read packet !");
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
}

//--------------------------------FUNCTIONS--------------------------------

const BUFF_SIZE: usize = 50;

fn send_message(stream: &mut TcpStream, mut msg: Vec<u8>) {
    if msg.len() % BUFF_SIZE == 0 {
        msg.extend([32].iter());
    }
    stream.write(&msg).unwrap();
    //println!("{}", String::from_utf8(msg).unwrap().len());
}

fn read_message(stream: &mut TcpStream) -> Vec<u8> {
    let mut last_msg_size: usize = BUFF_SIZE;
    let mut buff = [0 as u8; BUFF_SIZE];
    let mut msg_rcv_vec = Vec::new();

    loop {
        match stream.read(&mut buff) {
            Ok(size) => {
                last_msg_size = size;
                msg_rcv_vec.extend(buff[0..size].iter().copied());
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(e) => {
                println!("Failed to receive data: {}", e);
                break;
            }
        }
        if last_msg_size != BUFF_SIZE {
            break;
        }
    }
    msg_rcv_vec
}

pub fn get_user_input() -> String {
    let mut buff = String::new();
    let mut msg = String::new();

    stdin()
        .read_line(&mut buff)
        .expect("Can't read from stdin !");
    msg.push_str(buff.trim());
    msg
}
