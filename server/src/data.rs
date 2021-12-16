use crate::crypto;
use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as JsonError};
use rsa::{RsaPrivateKey, RsaPublicKey};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub sender_uid: String,
    pub receiver_uid: String,
    pub message: String,
    pub isRead: bool,
    pub time: String,
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
    pub fn new(sender_uid: String, receiver_uid: String , task: String, data: String) -> Self {
        Packet {
            sender_uid,
            receiver_uid,
            task,
            data,
            encrypted_data: false,
        }
    }

    pub fn from_vec(packet_vec: Vec<u8>) -> Result<Self, Box<dyn Error>>{
        match String::from_utf8(packet_vec.clone()) {
            Ok(packet_string) => {
                match Packet::from_string(packet_string) {
                    Ok(packet) => {
                        return Ok(packet);
                    }
                    Err(_) => {
                        return Err("Could not desirialize string")?;
                    }
                }
            }
            Err(_) => {
                return Err("Could not encode vec to utf8 string")?;
            }
        }
    }

    pub fn from_string(packet_string: String) -> Result<Self, JsonError> {
        serde_json::from_str(&packet_string[..])
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, Box<dyn Error>>{
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

    pub fn encrypt(&self, public_key: RsaPublicKey) -> String {
        match crypto::rsa_encrypt(&self.to_vec().unwrap(), public_key) {
            Ok(etext) => {
                return base64::encode(etext);
            }
            Err(e) => {
                println!("Failed to encrypt data: {}", e);
                return String::from("");
            }
        }
    }

    pub fn decrypt(packet_string: String, private_key: RsaPrivateKey) -> Result<Self, Box<dyn Error>> {
        match base64::decode(packet_string) {
            Ok(etext) => match crypto::rsa_decrypt(&etext, private_key) {
                Ok(ctext) => {
                    return Packet::from_vec(ctext);
                }
                Err(_) => {
                    return Err("Could not decrypt Message")?;
                }
            },
            Err(_) => {
                return Err("Failed to decode base64 Message")?;
            }
        }
    }
}
