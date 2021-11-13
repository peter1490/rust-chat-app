pub mod server;
pub mod client;

/* fn upload_msg(msg: String) -> Result<(), PostGresErr>{
    let mut client = Client::connect("postgresql://postgres:postgres@192.168.0.19/chatdb", NoTls)?;
    client.execute("
        INSERT INTO messages (text) VALUES ($1)
        ", &[&msg],
    )?;
    Ok(())
} */



fn main() {
    server::Server::run();
}
