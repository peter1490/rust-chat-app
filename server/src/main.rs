pub mod server;
pub mod client;
//pub mod db;
pub mod data;



fn main() {
    server::Server::run();
}
