use crate::client::ConnectedClient;
use json::{self, JsonValue};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    mpsc::{self, Receiver},
    Arc, Mutex,
};
use std::thread;

const BIND_ADDR: &str = "0.0.0.0:3333";
pub struct Server {
}

impl Server {
    pub fn run() {
        let connected_clients: Vec<ConnectedClient> = Vec::new();
        let listener = TcpListener::bind(BIND_ADDR).unwrap();
        let mutex = Arc::new(Mutex::new(connected_clients));
        // accept connections and process them, spawning a new thread for each one
        println!("Server listening on port 3333");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mutex_clone = Arc::clone(&mutex);
                    thread::spawn(move || {
                        let data_rcv = String::from_utf8(read_message(&mut stream)).unwrap();
                        let parsed_data = json::parse(data_rcv.as_str()).unwrap();

                        let client_uuid = parsed_data["sender_uuid"].to_string();
                        let client_task = parsed_data["task"].to_string();

                        if client_task == String::from("setup_receiver_stream") {
                            println!("New connection: {}", stream.peer_addr().unwrap());
                            let (tx, rx) = mpsc::channel::<Vec<u8>>();
                            let new_client = ConnectedClient {
                                thread_sender: tx,
                                uuid: client_uuid,
                            };

                            {
                                let mut mutex_lock = mutex_clone.lock().unwrap();
                                mutex_lock.push(new_client);
                            }
                            // connection succeeded
                            Server::setup_send_stream_client(stream, mutex_clone, rx)
                        } else {
                            Server::client_to_client_msg(parsed_data, mutex_clone)
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
        message_data: JsonValue,
        thread_mutex: Arc<Mutex<Vec<ConnectedClient>>>,
    ) {
        let connected_clients = thread_mutex.lock().unwrap();
        for connected_client in connected_clients.iter() {
            if connected_client.uuid == message_data["receiver_uuid"].to_string() {
                println!("New message sent");

                connected_client
                    .thread_sender
                    .send(message_data.to_string().as_bytes().to_vec())
                    .unwrap();
            }
        }
    }

    fn setup_send_stream_client(
        mut stream: TcpStream,
        thread_mutex: Arc<Mutex<Vec<ConnectedClient>>>,
        thread_channel_receiver: Receiver<Vec<u8>>,
    ) {
        {
            let connected_clients = thread_mutex.lock().unwrap();
            for connected_client in connected_clients.iter() {
                println!("{}", connected_client.uuid);
            }
        }

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
