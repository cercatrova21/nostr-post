
use nostr_types::{PublicKey, PrivateKey};

fn main() {
    let bech32 = rpassword::prompt_password("bech32: ").unwrap();

    if let Ok(key) = PublicKey::try_from_bech32_string(&bech32) {
        println!("Public Key: {}", key.as_hex_string());
    }
    else if let Ok(mut key) = PrivateKey::try_from_bech32_string(&bech32) {
        println!("Private Key: {}", key.as_hex_string());
    }
    else {
        println!("Invalid.");
    }
}
