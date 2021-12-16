use std::sync::mpsc::Sender;
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as JsonError};
pub struct ConnectedClient {
    pub thread_sender: Sender<Vec<u8>>,
    pub uuid: String,
}

#[derive(Serialize, Deserialize)]
pub struct Client {
    pub uuid: String,
    pub password: String,
    pub pub_key: String
}

impl Client {
    pub fn to_string(&self) -> Result<String, JsonError> {
        serde_json::to_string(&self)
    }

    pub fn from_string(client_string: String) -> Result<Self, JsonError> {
        serde_json::from_str(&client_string[..])
    }
}