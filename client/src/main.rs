use crate::data::{Message, convert_to_json, read_from_json};
use std::fs::File;
pub mod client;
pub mod crypto;
pub mod data;

fn main() {
    println!("Choose a username: ");

    let uuid = client::get_user_input();

    let curr_client = client::Client::new(uuid);

    curr_client.run();
}
