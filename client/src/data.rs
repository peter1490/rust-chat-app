use crate::crypto;
use base64;
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as JsonError};
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::{Write, Read};
use std::fs::OpenOptions;


#[derive(Serialize, Deserialize)]
pub struct Message {
    pub sender_uid: String,
    pub receiver_uid: String,
    pub message: String,
    pub isRead: bool,
    pub time: String,
}

impl Message {
    pub fn new(sender_uid: String, receiver_uid: String, message: String) -> Self {
        Message {
            sender_uid,
            receiver_uid,
            message,
            isRead: false,
            time: String::from("null"),
        }
    }

    pub fn from_string(message_string: String) -> Result<Self, JsonError> {
        serde_json::from_str(&message_string[..])
    }

    pub fn to_string(&self) -> Result<String, JsonError> {
        serde_json::to_string(&self)
    }
}

pub fn add_message_vector(message: Message) {
    let mut messages = read_from_json("history.json".to_string());
    messages.push(message);
    convert_to_json(messages);
}

pub fn convert_to_json(messages: Vec<Message>) -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(&messages)?;
    let mut file = OpenOptions::new()
        .create_new(false)
        .write(true)
        .append(false)
        .open("history.json")
        .unwrap();
    file.write_all(json.as_bytes()).expect("Cannot write in file");
    Ok(())
}

pub fn read_from_json(path: String) -> Vec<Message>{

    if !Path::new("history.json").exists() {
        File::create("history.json").expect("Cannot create file");
    }

    let mut messages = Vec::new();
    let mut file = OpenOptions::new()
        .create_new(false)
        .read(true)
        .open(path)
        .unwrap();
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Error while reading file");
    if json.len() > 0{
        messages = serde_json::from_str(&json).expect("Error while creating Vector");
    }
    messages
}

#[derive(Serialize, Deserialize)]
pub struct Packet {
    pub sender_uid: String,
    pub receiver_uid: String,
    pub task: String,
    pub data: String,
    encrypted_data: bool,
}

impl Packet {
    pub fn new(sender_uid: String, receiver_uid: String, task: String, data: String) -> Self {
        Packet {
            sender_uid,
            receiver_uid,
            task,
            data,
            encrypted_data: false,
        }
    }

    pub fn from_vec(packet_vec: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        match String::from_utf8(packet_vec.clone()) {
            Ok(packet_string) => match Packet::from_string(packet_string) {
                Ok(packet) => {
                    return Ok(packet);
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

    pub fn from_string(packet_string: String) -> Result<Self, JsonError> {
        serde_json::from_str(&packet_string[..])
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.to_string() {
            Ok(packet_string) => {
                return Ok(packet_string.as_bytes().to_vec());
            }
            Err(e) => {
                return Err("Could not serialize Packet to vec")?;
            }
        }
    }

    pub fn to_string(&self) -> Result<String, JsonError> {
        serde_json::to_string(&self)
    }

    pub fn encrypt_data(&mut self, key: &[u8]) {
        if self.data.len() != 0 {
            match crypto::aes_256_ctr_encrypt(&self.data.as_bytes().to_vec(), key) {
                Ok(etext) => {
                    self.data = base64::encode(etext);
                    self.encrypted_data = true;
                }
                Err(e) => {
                    println!("Failed to encrypt data: {}", e);
                }
            }
        }
    }

    pub fn decrypt_data(&mut self, key: &[u8]) {
        if self.data.len() != 0 {
            match base64::decode(self.data.clone()) {
                Ok(etext) => match crypto::aes_256_ctr_decrypt(&etext, key) {
                    Ok(ctext) => {
                        self.data = String::from_utf8(ctext).unwrap();
                        self.encrypted_data = false;
                    }
                    Err(e) => {
                        println!("Failed to decrypt data: {}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to decode base64 data: {}", e);
                }
            }
        }
    }
}
