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
pub mod server;
pub mod client;

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
/* fn upload_msg(msg: String) -> Result<(), PostGresErr>{
    let mut client = Client::connect("postgresql://postgres:postgres@192.168.0.19/chatdb", NoTls)?;
    client.execute("
        INSERT INTO messages (sender_id, receiver_id, message, isRead, time) VALUES ($1,$2,$3,$4,$5)
        ", &[&msg.sender_uid, &msg.receiver_uid, &msg.message, &msg.isRead, &msg.time],
    )?;
    Ok(())
} */

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


fn main() {
    server::Server::run();
}
