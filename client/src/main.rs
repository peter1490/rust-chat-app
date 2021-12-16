pub mod client;
pub mod crypto;
pub mod data;

fn main() {
    
    println!("uuid: ");

    let uuid = client::get_user_input();

    let curr_client = client::Client::new(uuid);

    curr_client.run();
}
