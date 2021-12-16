use crate::data::{Message, convert_to_json, read_from_json};
use std::fs::File;
pub mod client;
pub mod crypto;
pub mod data;

use rsa::pkcs1::{ToRsaPrivateKey, ToRsaPublicKey};
use std::fs::OpenOptions;
use std::io::{Read, Write};
fn main() {
    println!("Choose a username: ");
    let mut public_key = String::new();
    let mut private_key = String::new();
    match OpenOptions::new()
        .create_new(true)
        .write(true)
        .open("pub_key.pem")
    {
        Ok(mut pub_file) => {
            println!("Generating Keys, please wait...");
            let keys = crypto::rsa_gen_keys();
            println!("Keys generated !");
            public_key = keys.1.to_pkcs1_pem().unwrap();
            private_key = keys.0.to_pkcs1_pem().unwrap().to_string();
            pub_file
                .write_all(public_key.as_bytes())
                .expect("Could not write to file");

            let mut priv_file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open("priv_key.pem")
                .unwrap();

            priv_file
                .write_all(private_key.as_bytes())
                .expect("Could not write to file");
        }
        _ => {
            let mut pub_file = OpenOptions::new().read(true).open("pub_key.pem").unwrap();
            pub_file
                .read_to_string(&mut public_key)
                .expect("Could not read file");
            let mut priv_file = OpenOptions::new().read(true).open("priv_key.pem").unwrap();
            priv_file
                .read_to_string(&mut private_key)
                .expect("Could not read file");
        }
    }

    println!("uuid: ");

    let uuid = client::get_user_input();

    println!("Your password");
    let pass = client::get_user_input();

    let mut server_public_key = String::new();

    let mut server_pub_file = OpenOptions::new()
        .read(true)
        .open("server_pub.pem")
        .unwrap();
    server_pub_file
        .read_to_string(&mut server_public_key)
        .expect("Could not read file");
        let curr_client = client::Client::login(uuid, public_key, private_key, pass, server_public_key.clone());
    curr_client.run(server_public_key);
}
