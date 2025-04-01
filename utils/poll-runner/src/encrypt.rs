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

/// The length of the nonce used for encryption.
pub const NONCE_LENGTH: usize = 12;

#[must_use]
/// Generate a new RSA private key and shards for each staff member.
/// Each shard is encrypted with the staff member's public key.
///
/// # Panics
/// Panics if the staff count is greater than 255.
/// Panics if the private key cannot be generated.
/// Panics if the key cannot be encoded.
pub fn generate_key<S: ::std::hash::BuildHasher>(
    staff: HashMap<UserId, RsaPublicKey, S>,
) -> (RsaPrivateKey, HashMap<UserId, String>) {
    let mut rng = rand::thread_rng();
    let key = rsa::RsaPrivateKey::new(&mut rng, 512).expect("should generate private key");
    let mut shards = ssss::gen_shares(
        SsssConfig::default()
            .set_num_shares(u8::try_from(staff.len()).expect("staff count should be valid u8")),
        key.to_pkcs1_der().expect("should encode key").as_bytes(),
    )
    .expect("should generate shares");
    let mut encrypted_shards = HashMap::new();
    for (member, key) in staff {
        let shard = shards.pop().expect("should have enough shards");
        encrypted_shards.insert(member, STANDARD.encode(encrypt(shard.as_bytes(), &key)));
    }
    (key, encrypted_shards)
}

#[must_use]
/// Rebuild the private key from the shards.
/// Each shard is decrypted using the staff member's private key.
/// The private key is then rebuilt from the decrypted shards.
///
/// # Panics
/// Panics if the shard cannot be decoded.
/// Panics if the shard is not valid utf8.
/// Panics if the key cannot be unlocked.
/// Panics if the key cannot be decoded.
/// Panics if the key cannot be rebuilt.
pub fn rebuild_key(shards: Vec<(String, RsaPrivateKey)>) -> RsaPrivateKey {
    let shards = shards
        .into_iter()
        .map(|(encrypted_shard, key)| {
            let shard = decrypt(
                &STANDARD
                    .decode(encrypted_shard)
                    .expect("shard should be standard base64 encoded"),
                &key,
            );
            String::from_utf8(shard).expect("shard should be valid utf8")
        })
        .collect::<Vec<_>>();
    let key = ssss::unlock(&shards).expect("should unlock key");
    RsaPrivateKey::from_pkcs1_der(&key).expect("should decode key")
}

#[must_use]
/// Generate a ticket for a user to vote in a poll.
/// The ticket is encrypted using the poll's public key.
/// The ticket is in the format:
/// - 1 byte for the length of the poll UUID
/// - The poll UUID
/// - The encrypted data
///
/// The ticket is then encoded using the URL safe base64 encoding.
///
/// # Panics
/// Panics if the poll length is greater than 255.
/// Panics if the data cannot be encrypted.
/// Panics if the data cannot be encoded.
pub fn ticket(poll: &Uuid, member: UserId, key: &RsaPublicKey) -> String {
    let data = encrypt(format!("{member}:{poll}").as_bytes(), key);
    let poll = poll.as_bytes().to_vec();
    let mut out = Vec::with_capacity(1 + poll.len() + data.len());
    out.push(u8::try_from(poll.len()).expect("poll length should be valid u8"));
    out.extend_from_slice(&poll);
    out.extend_from_slice(&data);
    URL_SAFE.encode(out)
}

#[must_use]
/// Encrypt the data using the public key.
/// The data is encrypted using a randomly generated password.
/// The password is then encrypted using the public key.
/// The data returned is in the format:
/// - 1 byte for the length of the encrypted password
/// - The encrypted password
/// - The nonce of length [`NONCE_LENGTH`]
/// - The encrypted data
///
/// # Panics
/// Panics if the password cannot be encrypted.
/// Panics if the data cannot be encrypted.
/// Panics if the password length is greater than 255.
pub fn encrypt(data: &[u8], key: &RsaPublicKey) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let password = rng.gen::<[u8; 32]>();
    let encrypted_password = key
        .encrypt(&mut rng, Pkcs1v15Encrypt, &password)
        .expect("should encrypt password");
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&password));
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let output = cipher.encrypt(&nonce, data).expect("should encrypt data");
    let mut out = Vec::with_capacity(NONCE_LENGTH + output.len() + encrypted_password.len());
    let password_length =
        u8::try_from(encrypted_password.len()).expect("password length should be valid u8");
    out.push(password_length);
    out.extend_from_slice(&encrypted_password);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&output);
    out
}

#[must_use]
/// Decrypt the data using the private key.
/// The data is expected to be in the format:
/// - 1 byte for the length of the encrypted password
/// - The encrypted password
/// - The nonce of length [`NONCE_LENGTH`]
/// - The encrypted data
///
/// The password is decrypted using the private key and used to decrypt the data.
///
/// # Panics
/// Panics if the password cannot be decrypted.
/// Panics if the data cannot be decrypted.
/// Panics if the data is not in the expected format.
pub fn decrypt(data: &[u8], key: &RsaPrivateKey) -> Vec<u8> {
    let password_length = data[0];
    let encrypted_password = &data[1..=(password_length as usize)];
    let nonce = &data[(password_length as usize + 1)..=(NONCE_LENGTH + password_length as usize)];
    let encrypted = &data[NONCE_LENGTH + password_length as usize + 1..];
    let password = key
        .decrypt(Pkcs1v15Encrypt, encrypted_password)
        .expect("should decrypt password");
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&password));
    cipher
        .decrypt(nonce.into(), encrypted)
        .expect("should decrypt data")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
