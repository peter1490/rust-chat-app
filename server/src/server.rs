use crate::client::{ConnectedClient, Client};
use crate::{crypto, server, db};
use crate::data::Packet;
use std::fs::OpenOptions;
use rsa::pkcs1::{FromRsaPrivateKey, FromRsaPublicKey};
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    mpsc::{self, Receiver},
    Arc, Mutex,
};
use std::error::Error;
use std::thread;

const BIND_ADDR: &str = "0.0.0.0:3333";
pub struct Server {
    pub pub_key: String,
    priv_key: String,
}

impl Server {
    pub fn new(pub_key: String, priv_key: String) -> Self {
        Server { pub_key, priv_key }
    }
    pub fn run(self) {
        let connected_clients: Vec<ConnectedClient> = Vec::new();
        let listener = TcpListener::bind(BIND_ADDR).unwrap();
        let mutex = Arc::new(Mutex::new(connected_clients));
        // accept connections and process them, spawning a new thread for each one
        println!("Server listening on port 3333");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let public_key = self.pub_key.clone();
                    let private_key = self.priv_key.clone();
                    let mutex_clone = Arc::clone(&mutex);
                    thread::spawn(move || {
                        let temp_public_key = public_key.clone();
                        let temp_private_key = private_key.clone(); 
                        match Server::decrypt_packet(read_message(&mut stream), temp_private_key) {
                            Ok(packet) => {
                                
                                println!("New packet");
                                let task: &str = &packet.task;
                                match task {
                                    "setup_receiver_stream" => {
                                        let (tx, rx) = mpsc::channel::<Vec<u8>>();
                                        let new_client = ConnectedClient {
                                            thread_sender: tx,
                                            uuid: packet.sender_uid,
                                        };

                                        {
                                            let mut mutex_lock = mutex_clone.lock().unwrap();
                                            mutex_lock.push(new_client);
                                        }
                                        // connection succeeded
                                        Server::setup_send_stream_client(stream, mutex_clone, rx)
                                    }
                                    "send_message" => {
                                        Server::client_to_client_msg(packet, mutex_clone)
                                    }
                                    "register_user" => {
                                        Server::register_user(stream, packet)
                                    }
                                    _ => {
                                        println!("Bad task !");
                                    }
                                }
                            }
                            _ => {
                                println!("Could not read packet !");
                            }
                        }
                    });
                }
                Err(e) => panic!("encountered error: {}", e),
            }
        }
        // close the socket server
        drop(listener);
    }

    fn client_to_client_msg(
        packet: Packet,
        thread_mutex: Arc<Mutex<Vec<ConnectedClient>>>,
    ) {
        let connected_clients = thread_mutex.lock().unwrap();
        for connected_client in connected_clients.iter() {
            if connected_client.uuid == packet.receiver_uid {
                println!("New message sent");

                connected_client
                    .thread_sender
                    .send(packet.to_vec().unwrap())
                    .unwrap();
            }
        }
    }

    fn setup_send_stream_client(
        mut stream: TcpStream,
        thread_mutex: Arc<Mutex<Vec<ConnectedClient>>>,
        thread_channel_receiver: Receiver<Vec<u8>>,
    ) {
       /*  {
            let connected_clients = thread_mutex.lock().unwrap();
            for connected_client in connected_clients.iter() {
                println!("{}", connected_client.uuid);
            }
        } */

        while match thread_channel_receiver.recv() {
            Ok(ch_msg) => {
                /* if let Err(e) = upload_msg(String::from_utf8(ch_msg.clone()).unwrap()) {
                    println!("{}", e);
                } */
                send_message(&mut stream, ch_msg);
                true
            }
            Err(e) => {
                panic!("encountered error: {}", e);
            }
        } {}
        println!("Connection ended");
    }

    fn register_user(mut stream: TcpStream, mut packet: Packet) {
        send_message(&mut stream, "OK".as_bytes().to_vec());
        let pub_key = String::from_utf8(read_message(&mut stream)).unwrap();
        let mut client = Client::from_string(packet.data).unwrap();
        client.pub_key = pub_key.clone();

        packet.sender_uid = String::new();
        packet.receiver_uid = String::new();
        packet.data = String::from("OK");

        db::new_user(client.uuid.clone(), client.password.clone(), client.pub_key);

        println!("{}", packet.to_string().unwrap());

        let pub_key = FromRsaPublicKey::from_pkcs1_pem(&pub_key).unwrap();
        let packet_string = packet.encrypt(pub_key);
        send_message(&mut stream, packet_string.as_bytes().to_vec());
    }

    fn encrypt_packet(packet: Packet, public_key: RsaPublicKey) -> Vec<u8> {
        packet.encrypt(public_key).as_bytes().to_vec()
    }

    fn decrypt_packet(packet_vec: Vec<u8>, private_key: String) -> Result<Packet, Box<dyn Error>> {
        let priv_key: RsaPrivateKey = FromRsaPrivateKey::from_pkcs1_pem(&private_key).unwrap();
        Packet::decrypt(String::from_utf8(packet_vec).unwrap(), priv_key)
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
