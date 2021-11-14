use std::fmt::Error;
use aes::{Aes256Ctr, cipher::{
    StreamCipher,NewCipher,
}};

pub fn aes_256_ctr_encrypt(ptext: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    if key.len() != 32 {
        return Err(Error);
    }

    let zero_nonce = [0u8; 16];
    let mut cipher = Aes256Ctr::new(key.into(), (&zero_nonce).into());

    let mut ctext = ptext.to_vec();
    cipher.apply_keystream(&mut ctext);

    Ok(ctext)
}

pub fn aes_256_ctr_decrypt(ctext: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    aes_256_ctr_encrypt(ctext, key)
}
