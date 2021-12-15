use std::collections::HashMap;
use std::hash::Hash;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::string::String;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use json::{self, JsonValue};
use postgres::{Client, NoTls, Error as PostGresErr};

struct Message {
    sender_uid: String,
    receiver_uid: String,
    message: String,
    isRead: bool,
    time: String,
}

struct ConnectedClient {
    thread_sender: Sender<Vec<u8>>,
    uuid: String,
}

impl Clone for ConnectedClient {
    fn clone(&self) -> Self {
        ConnectedClient {
            thread_sender: self.thread_sender.clone(),
            uuid: self.uuid.clone(),
        }
    }
}

/*
                                           Table "public.messages"
   Column    |            Type             | Collation | Nullable |                 Default                  
-------------+-----------------------------+-----------+----------+------------------------------------------
 msg_id      | integer                     |           | not null | nextval('messages_msg_id_seq'::regclass)
 sender_id   | integer                     |           | not null | 
 receiver_id | integer                     |           | not null | 
 message     | text                        |           |          | 
 isread      | boolean                     |           | not null | 
 time        | timestamp without time zone |           |          | 
*/



/*
                                         Table "public.user"
    Column     |         Type          | Collation | Nullable |                Default                
---------------+-----------------------+-----------+----------+---------------------------------------
 user_id       | integer               |           | not null | nextval('user_user_id_seq'::regclass)
 user_username | character varying(50) |           | not null | 
 user_password | character varying(50) |           | not null | 
*/

fn upload_msg(msg: Message) -> Result<(), PostGresErr>{
    let mut client = Client::connect("postgresql://chatdbuser:,qY6p}[y]y5wD2=p@20.123.186.55/chatdb", NoTls)?;
    client.execute("
        INSERT INTO messages (sender_id, receiver_id, message, isRead, time) VALUES ($1,$2,$3,$4,$5)
        ", &[&msg.sender_uid, &msg.receiver_uid, &msg.message, &msg.isRead, &msg.time],
    )?;
    Ok(())
}

fn get_new_messages(uuid: String) -> Result<Vec<Message>, PostGresErr> {
    let mut client = Client::connect("postgresql://chatdbuser:,qY6p}[y]y5wD2=p@20.123.186.55/chatdb", NoTls)?;
    let mut messages = Vec::new();
    for row in &client.query("SELECT sender_id, receiver_id, message, isRead, time FROM messages WHERE isRead = False AND receiver_id = 
                                         SELECT user_id FROM user WHERE user_username = ($1)", &[&uuid]).unwrap() {
        let message = Message {
            sender_uid: row.get(0),
            receiver_uid: row.get(1),
            message: row.get(2),
            isRead: row.get(3),
            time: row.get(4)
        };
        messages.push(message);
    };
    client.execute("UPDATE messages SET isRead = True WHERE isRead = False AND receiver_id = 
                                    SELECT user_id FROM user WHERE user_username = ($1)", &[&uuid])?;
    Ok(messages)
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
            if let Err(e) = upload_msg(String::from_utf8(ch_msg.clone()).unwrap()) {
                println!("{}", e);
            }
            send_message(&mut stream, ch_msg);
            true
        }
        Err(e) => {
            panic!("encountered error: {}", e);
        }
    } {}
    println!("Connection ended");
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

fn main() {
    let connected_clients: Vec<ConnectedClient> = Vec::new();
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
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
                        setup_send_stream_client(stream, mutex_clone, rx)
                    } else {
                        client_to_client_msg(parsed_data, mutex_clone)
                    }
                });
            }
            Err(e) => panic!("encountered error: {}", e),
        }
    }
    // close the socket server
    drop(listener);
}
