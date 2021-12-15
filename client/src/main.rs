pub mod client;
pub mod crypto;
use std::fmt::Result;
use std::io::{ErrorKind, Read, Write, stdin};
use std::net::TcpStream;
use std::{thread, usize};
use base64;
use json::{self, JsonValue, object};
use serde_json;
use serde::{Serialize};


#[derive(Serialize)]
struct Message {
    sender_uid: String,
    receiver_uid: String,
    message: String,
    isRead: bool,
    time: String,
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

fn new_messages_file(messages: Vec<Message>) -> serde_json::Result<()> {
    let mut messages_serialized = String::new();
    for i in messages {
        let msg = serde_json::to_string(&i)?;
        messages_serialized += &msg;
    }
    Ok(())
}

const BUFF_SIZE: usize = 50;

fn send_message(stream: &mut TcpStream, mut msg:Vec<u8>) {
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

fn setup_receive_stream(uuid: String) {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333");
            
            let msg_data = object!{
                sender_uuid: uuid,
                task: "setup_receiver_stream",
            };
            stream.write(msg_data.to_string().as_bytes()).unwrap();
            
            loop {
                let data_rcv = String::from_utf8(read_message(&mut stream)).unwrap();
                //println!("{:?}", data_rcv);
                let parsed_data = json::parse(data_rcv.as_str()).unwrap();

                let msg_rcv = String::from_utf8(base64::decode(parsed_data["message"].to_string()).unwrap()).unwrap();
                if msg_rcv.len() == 0 {
                    println!("Break");
                    break;
                }else{
                    println!("{:?}: {:?}", parsed_data["sender_uuid"].to_string(), msg_rcv);
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}

fn new_message(msg_data: JsonValue){
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            
            send_message(&mut stream, msg_data.to_string().as_bytes().to_vec());
            //println!("you: {:?}", String::from_utf8(base64::decode(msg_data["message"].to_string()).unwrap()).unwrap());
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}


fn main() {
    
    println!("uuid: ");

    /* let uuid = client::get_user_input();

    let curr_client = client::Client::new(uuid);

    curr_client.run(); */
}
