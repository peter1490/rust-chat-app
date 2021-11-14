//pub mod client;
//pub mod crypto;
use openssl::rsa::{Padding, Rsa};

use rand::rngs::ThreadRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

fn main() {
    let mut rng = ThreadRng::default();
    let bits = 4096;

    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    println!("test0: ");
    let public_key = RsaPublicKey::from(&private_key);

    println!("test1: ");

    // Encrypt
    let data = b"hello world";
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let enc_data = public_key
        .encrypt(&mut rng, padding, &data[..])
        .expect("failed to encrypt");

    println!("test2: ");
    // Decrypt
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let dec_data = private_key
        .decrypt(padding, &enc_data)
        .expect("failed to decrypt");

    println!("{:?}", String::from_utf8(dec_data));

    /* let rsa = Rsa::generate(4096).unwrap();
    let data = b"Hello bitch";
    let data_len = data.len();
    let mut buf = vec![0; rsa.size() as usize];
    let encrypted_len = rsa.public_encrypt(data, &mut buf, Padding::PKCS1).unwrap();

    let data = buf;

    let mut buf = vec![0; rsa.size() as usize];
    let decrypted_len = rsa.private_decrypt(&data, &mut buf, Padding::PKCS1).unwrap();
    println!("{:?}", String::from_utf8(buf[0..data_len].to_vec()));
 */
    println!("uuid: ");

    /* let uuid = client::get_user_input();

    let curr_client = client::Client::new(uuid);

    curr_client.run(); */
}
