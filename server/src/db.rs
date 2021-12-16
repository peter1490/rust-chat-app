use crate::data::Message;
use std::error::Error;

use postgres::{Client, NoTls, Error as PostGresErr};

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
 pub_key     | text                        |           |          | 
*/



/*
                                         Table "public.user"
    Column     |         Type          | Collation | Nullable |                Default                
---------------+-----------------------+-----------+----------+---------------------------------------
 user_id       | integer               |           | not null | nextval('user_user_id_seq'::regclass)
 user_username | character varying(50) |           | not null | 
 user_password | character varying(50) |           | not null |
 pub_key       | text                  |           |          |
*/

fn upload_msg(msg: Message) -> Result<(), PostGresErr>{
    let mut client = Client::connect("postgresql://chatdbuser:,qY6p}[y]y5wD2=p@20.123.186.55/chatdb", NoTls)?;
    client.execute("
        INSERT INTO messages (sender_id, receiver_id, message, isRead, time) VALUES (SELECT user_id FROM user WHERE user_username = $1,
                                                                                     SELECT user_id FROM user WHERE user_username = $2,
                                                                                     $3,$4,$5)
        ", &[&msg.sender_uid, &msg.receiver_uid, &msg.message, &msg.isRead, &msg.time],
    )?;
    Ok(())
}

fn get_new_messages(uuid: String) -> Result<Vec<Message>, PostGresErr> {
    let mut client = Client::connect("postgresql://chatdbuser:,qY6p}[y]y5wD2=p@20.123.186.55/chatdb", NoTls)?;
    let mut messages = Vec::new();
    let temp_uuid = uuid.clone();
    for row in &client.query("SELECT user_username, message, isRead, time 
                              INNER JOIN user ON sender_id = user_id 
                              FROM messages WHERE isRead = False AND receiver_id = 
                                    SELECT user_id FROM user WHERE user_username = ($1)", &[&uuid]).unwrap() {
        let message = Message {
            sender_uid: row.get(0),
            receiver_uid: temp_uuid.clone(),
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

pub fn new_user(user_uid: String, user_pass: String, user_pb: String) -> Result<u64, PostGresErr>{
    let mut client = Client::connect("postgresql://chatdbuser:,qY6p}[y]y5wD2=p@localhost/chatdb", NoTls)?;
    client.execute("
        INSERT INTO user (user_username, user_password, pub_key) VALUES ($1,$2,$3) WHERE NOT EXIST (SELECT 1 FROM user WHERE  user_username = $4)
        ", &[&user_uid, &user_pass, &user_pb, &user_uid],
    )
}

pub fn get_pub_key(uuid: String) -> Result<String, Box<dyn Error>> {
    let mut client = Client::connect("postgresql://chatdbuser:,qY6p}[y]y5wD2=p@localhost/chatdb", NoTls)?;
    for row in &client.query("SELECT pub_key FROM user WHERE user_username = $1", &[&uuid]).unwrap() {
        return Ok(row.get(0));
    };
    Err("No public key")?
}

pub fn get_user_password(uuid: String) -> Result<String, Box<dyn Error>> {
    let mut client = Client::connect("postgresql://chatdbuser:,qY6p}[y]y5wD2=p@localhost/chatdb", NoTls)?;
    for row in &client.query("SELECT user_password FROM user WHERE user_username = $1", &[&uuid]).unwrap() {
        return Ok(row.get(0));
    };
    Err("No row found")?
}