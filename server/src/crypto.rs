use rand::rngs::ThreadRng;
use rsa::{errors::Error as RsaError, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

const RSA_KEY_SIZE: usize = 4096;

pub fn rsa_gen_keys() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = ThreadRng::default();
    let private_key = RsaPrivateKey::new(&mut rng, RSA_KEY_SIZE).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

pub fn rsa_encrypt(ctext: &[u8], pub_key: RsaPublicKey) -> Result<Vec<u8>, RsaError> {
    let mut rng = ThreadRng::default();
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    pub_key.encrypt(&mut rng, padding, &ctext[..])
}

pub fn rsa_decrypt(ptext: &[u8], priv_key: RsaPrivateKey) -> Result<Vec<u8>, RsaError> {
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    priv_key.decrypt(padding, &ptext)
}