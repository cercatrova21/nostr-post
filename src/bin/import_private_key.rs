
use nostr_types::{Event, EventKind, PreEvent, PrivateKey, Unixtime};
use zeroize::Zeroize;

// Turn a hex private key into an encrypted private key
fn main() {
    // Put your private key here, but
    // DO NOT COMMIT THIS!
    let hex_private_key = "FIXME";

    let mut password = rpassword::prompt_password("Password: ").unwrap();
    let private_key = PrivateKey::try_from_hex_string(hex_private_key)
        .expect("Could not import private key");
    let encrypted_private_key = private_key.export_encrypted(&password)
        .expect("Could not export encrypted private key");
    println!("Encrypted private key: {}", encrypted_private_key);
    password.zeroize();
}
