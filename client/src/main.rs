pub mod client;

fn main() {
    println!("uuid: ");
    
    let uuid = client::get_user_input();

    let curr_client = client::Client::new(uuid);

    curr_client.run();
}
