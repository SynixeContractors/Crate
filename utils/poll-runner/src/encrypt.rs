use std::collections::HashMap;

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead},
    AeadCore, Aes256Gcm, KeyInit,
};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE},
    Engine,
};
use rand::{rngs::OsRng, Rng};
use rsa::{
    pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey},
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use serenity::all::UserId;
use ssss::SsssConfig;
use uuid::Uuid;

const NONCE_LENGTH: usize = 12;

pub fn generate_key(
    staff: HashMap<UserId, RsaPublicKey>,
) -> (RsaPrivateKey, HashMap<UserId, String>) {
    let mut rng = rand::thread_rng();
    let key = rsa::RsaPrivateKey::new(&mut rng, 512).unwrap();
    let mut shards = ssss::gen_shares(
        &SsssConfig::default(),
        key.to_pkcs1_der().unwrap().as_bytes(),
    )
    .unwrap();
    let mut encrypted_shards = HashMap::new();
    for (member, key) in staff {
        let shard = shards.pop().unwrap();
        encrypted_shards.insert(member, STANDARD.encode(encrypt(shard.as_bytes(), &key)));
    }
    (key, encrypted_shards)
}

pub fn rebuild_key(shards: Vec<(String, RsaPrivateKey)>) -> RsaPrivateKey {
    let shards = shards
        .into_iter()
        .map(|(encrypted_shard, key)| {
            let shard = decrypt(&STANDARD.decode(encrypted_shard).unwrap(), &key);
            String::from_utf8(shard).unwrap()
        })
        .collect::<Vec<_>>();
    let key = ssss::unlock(&shards).unwrap();
    RsaPrivateKey::from_pkcs1_der(&key).unwrap()
}

pub fn ticket(poll: &Uuid, member: UserId, key: &RsaPrivateKey) -> String {
    let data = encrypt(format!("{member}:{poll}").as_bytes(), &key.to_public_key());
    let poll = poll.as_bytes().to_vec();
    let mut out = Vec::with_capacity(1 + poll.len() + data.len());
    out.push(u8::try_from(poll.len()).unwrap());
    out.extend_from_slice(&poll);
    out.extend_from_slice(&data);
    URL_SAFE.encode(out)
}

pub fn encrypt(data: &[u8], key: &RsaPublicKey) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let password = rng.gen::<[u8; 32]>();
    let encrypted_password = key.encrypt(&mut rng, Pkcs1v15Encrypt, &password).unwrap();
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&password));
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let output = cipher.encrypt(&nonce, data).unwrap();
    let mut out = Vec::with_capacity(NONCE_LENGTH + output.len() + encrypted_password.len());
    let password_length = u8::try_from(encrypted_password.len()).unwrap();
    out.push(password_length);
    out.extend_from_slice(&encrypted_password);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&output);
    out
}

pub fn decrypt(data: &[u8], key: &RsaPrivateKey) -> Vec<u8> {
    let password_length = data[0];
    let encrypted_password = &data[1..=(password_length as usize)];
    let nonce = &data[(password_length as usize + 1)..=(NONCE_LENGTH + password_length as usize)];
    let encrypted = &data[NONCE_LENGTH + password_length as usize + 1..];
    let password = key.decrypt(Pkcs1v15Encrypt, encrypted_password).unwrap();
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&password));
    cipher.decrypt(nonce.into(), encrypted).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let mut rng = rand::thread_rng();
        let key = rsa::RsaPrivateKey::new(&mut rng, 512).unwrap();
        let data = b"Hello, World!";
        let encrypted = encrypt(data, &key.to_public_key());
        let decrypted = decrypt(&encrypted, &key);
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_generate_rebuild_key() {
        let mut rng = rand::thread_rng();
        let user_1_key = rsa::RsaPrivateKey::new(&mut rng, 512).unwrap();
        let user_2_key = rsa::RsaPrivateKey::new(&mut rng, 512).unwrap();
        let user_3_key = rsa::RsaPrivateKey::new(&mut rng, 512).unwrap();
        let user_4_key = rsa::RsaPrivateKey::new(&mut rng, 512).unwrap();
        let user_5_key = rsa::RsaPrivateKey::new(&mut rng, 512).unwrap();
        let (key, shards) = generate_key({
            let mut map = HashMap::new();
            map.insert(UserId::new(1), user_1_key.to_public_key());
            map.insert(UserId::new(2), user_2_key.to_public_key());
            map.insert(UserId::new(3), user_3_key.to_public_key());
            map.insert(UserId::new(4), user_4_key.to_public_key());
            map.insert(UserId::new(5), user_5_key.to_public_key());
            map
        });
        let rebuilt_key = rebuild_key(vec![
            (shards.get(&UserId::new(1)).unwrap().to_string(), user_1_key),
            (shards.get(&UserId::new(2)).unwrap().to_string(), user_2_key),
            (shards.get(&UserId::new(3)).unwrap().to_string(), user_3_key),
            (shards.get(&UserId::new(4)).unwrap().to_string(), user_4_key),
            (shards.get(&UserId::new(5)).unwrap().to_string(), user_5_key),
        ]);
        assert_eq!(key, rebuilt_key);
    }
}
