use std::sync::mpsc::Sender;
pub struct ConnectedClient {
    pub thread_sender: Sender<Vec<u8>>,
    pub uuid: String,
}