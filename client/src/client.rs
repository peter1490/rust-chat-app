use crate::data::{Message, Packet};

use hex_literal::hex;
use rsa::pkcs1::{FromRsaPrivateKey, FromRsaPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as JsonError};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{stdin, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::str;
use std::{thread, usize};

#[derive(Serialize, Deserialize)]
pub struct Client {
    pub uuid: String,
    pub pub_key: String,
    pub priv_key: String,
    pub password: String,
}

impl Client {
    pub fn new(uuid: String, pub_key: String, priv_key: String, password: String) -> Self {
        Client {
            uuid,
            pub_key,
            priv_key,
            password,
        }
    }

    pub fn from_vec(client_vec: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        match String::from_utf8(client_vec.clone()) {
            Ok(client_string) => match Client::from_string(client_string) {
                Ok(client) => {
                    return Ok(client);
                }
                Err(_) => {
                    return Err("Could not desirialize string")?;
                }
            },
            Err(_) => {
                return Err("Could not encode vec to utf8 string")?;
            }
        }
    }

    pub fn from_string(client_string: String) -> Result<Self, JsonError> {
        serde_json::from_str(&client_string[..])
    }

    pub fn to_vec(self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.to_string() {
            Ok(client_string) => {
                return Ok(client_string.as_bytes().to_vec());
            }
            Err(_) => {
                return Err("Could not serialize Packet to vec")?;
            }
        }
    }

    pub fn to_string(&self) -> Result<String, JsonError> {
        serde_json::to_string(&self)
    }

    pub fn login(
        uuid: String,
        pub_key: String,
        priv_key: String,
        mut password: String,
        server_pub_key: String,
    ) -> Self {
        match OpenOptions::new().read(true).open("user") {
            Ok(mut file) => {
                let mut user_string = String::new();
                file.read_to_string(&mut user_string)
                    .expect("Could not read file");
                let client = Client::from_string(user_string).unwrap();
                loop {
                    if client.password == hash_sha256(password.clone()) {
                        return client;
                    }else{
                        println!("Wrong Password !");
                        println!("Your password");
                        password = get_user_input();
                    }
                }
            }
            _ => {
                println!("Would you like to register or login ? (re/lo)");
                let ch = get_user_input();
                let client = Client::new(uuid, pub_key, priv_key, password);
                if &ch == "re" {
                    client.register(server_pub_key.clone());
                    return client;
                } else {
                    if client.check_account(server_pub_key) {
                        return client;
                    }
                    Client::new(
                        String::from("null"),
                        String::from("null"),
                        String::from("null"),
                        String::from("null"),
                    )
                }
            }
        }
    }

    pub fn to_file(self) {
        let temp_client = Client::new(self.uuid.clone(), String::new(), String::new(), self.password);
        match temp_client.to_string() {
            Ok(user_string) => {
                let mut user_file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open("user")
                    .unwrap();

                user_file
                    .write_all(user_string.as_bytes())
                    .expect("Could not write to file");
            }
            Err(e) => {
                println!("Error writing user file: {}", e);
            }
        }
    }

    pub fn check_account(&self, server_pub_key: String) -> bool {
        match TcpStream::connect("localhost:3333") {
            Ok(mut stream) => {
                let pub_key = FromRsaPublicKey::from_pkcs1_pem(&server_pub_key).unwrap();
                let mut temp_client = Client::new(
                    self.uuid.clone(),
                    String::new(),
                    String::from(""),
                    self.password.clone(),
                );
                let packet = Packet::new(
                    self.uuid.clone(),
                    String::new(),
                    String::from("check_user"),
                    temp_client.to_string().unwrap(),
                );
                let packet_string = packet.encrypt(pub_key);

                send_message(&mut stream, packet_string.as_bytes().to_vec());

                let priv_key = FromRsaPrivateKey::from_pkcs1_pem(&self.priv_key).unwrap();

                match Packet::decrypt(
                    String::from_utf8(read_message(&mut stream)).unwrap(),
                    priv_key,
                ) {
                    Ok(packet) => {
                        if packet.task == "OK" {
                            temp_client.password = hash_sha256(temp_client.password.clone());
                            temp_client.to_file();
                            return true;
                        }
                        println!("failed");
                        return false;
                    }
                    Err(e) => {
                        println!("{}", e);
                        return false;
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                return false;
            }
        }
    }

    pub fn register(&self, server_pub_key: String) -> bool {
        match TcpStream::connect("localhost:3333") {
            Ok(mut stream) => {
                let pub_key = FromRsaPublicKey::from_pkcs1_pem(&server_pub_key).unwrap();
                let mut temp_client = Client::new(
                    self.uuid.clone(),
                    String::from(""),
                    String::from(""),
                    self.password.clone(),
                );
                let packet = Packet::new(
                    self.uuid.clone(),
                    String::new(),
                    String::from("register_user"),
                    temp_client.to_string().unwrap(),
                );
                let packet_string = packet.encrypt(pub_key);
                //println!("{}", packet_string);
                send_message(&mut stream, packet_string.as_bytes().to_vec());

                if str::from_utf8(&read_message(&mut stream)).unwrap() == "OK" {
                    send_message(&mut stream, self.pub_key.as_bytes().to_vec());
                }

                let priv_key = FromRsaPrivateKey::from_pkcs1_pem(&self.priv_key).unwrap();

                match Packet::decrypt(
                    String::from_utf8(read_message(&mut stream)).unwrap(),
                    priv_key,
                ) {
                    Ok(packet) => {
                        if &packet.data == "OK" {
                            temp_client.password = hash_sha256(temp_client.password.clone());
                            temp_client.to_file();
                            return true;
                        }
                        return false;
                    }
                    Err(e) => {
                        println!("{}", e);
                        return false;
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                return false;
            }
        }
    }

    pub fn run(&self, server_pub_key: String) {
        let tmp_uuid = self.uuid.clone();
        let tmp_server_pub_key = server_pub_key.clone();

        thread::spawn(move || Client::setup_receive_stream(tmp_uuid, tmp_server_pub_key));

        println!("client:");
        let end_client = get_user_input();

        loop {
            let msg = get_user_input();

            let key = hex!("603DEB1015CA71BE2B73AEF0857D77811F352C073B6108D72D9810A30914DFF4");

            let message = Message::new(self.uuid.clone(), end_client.clone(), msg);
            let packet = Packet::new(
                String::from(""),
                end_client.clone(),
                String::from("send_message"),
                message.encrypt(&key),
            );
            self.new_message(packet, server_pub_key.clone());
        }
    }

    fn new_message(&self, packet: Packet, server_pub_key: String) {
        match TcpStream::connect("localhost:3333") {
            Ok(mut stream) => {
                let pub_key = FromRsaPublicKey::from_pkcs1_pem(&server_pub_key).unwrap();
                let packet_string = packet.encrypt(pub_key);
                send_message(&mut stream, packet_string.as_bytes().to_vec());
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }

    fn setup_receive_stream(uuid: String, server_pub_key: String) {
        match TcpStream::connect("localhost:3333") {
            Ok(mut stream) => {
                println!("Successfully connected to server in port 3333");
                let pub_key = FromRsaPublicKey::from_pkcs1_pem(&server_pub_key).unwrap();

                let packet = Packet::new(
                    String::from(uuid),
                    String::from(""),
                    String::from("setup_receiver_stream"),
                    String::from(""),
                );
                let packet_string = packet.encrypt(pub_key);
                stream.write(&packet_string.as_bytes().to_vec()).unwrap();

                loop {
                    match Packet::from_vec(read_message(&mut stream)) {
                        Ok(packet) => {
                            let key = hex!(
                                "603DEB1015CA71BE2B73AEF0857D77811F352C073B6108D72D9810A30914DFF4"
                            );

                            //packet.decrypt(&key);

                            match Message::decrypt(packet.data, &key) {
                                Ok(msg) => {
                                    if msg.message.len() == 0 {
                                        println!("Break");
                                        break;
                                    } else {
                                        println!("{:?}: {:?}", msg.sender_uid, msg.message);
                                    }
                                }
                                _ => {
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

pub fn hash_sha256(text: String) -> String {

    // create a Sha256 object
    let mut hasher = Sha256::new();

    // write input message
    hasher.input_str(&text);

    // read hash digest
    hasher.result_str()
}
