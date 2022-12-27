
use nostr_types::{ClientMessage, EncryptedPrivateKey, Event, EventKind, Id, Metadata, PreEvent, PrivateKey, PublicKey, Tag, Unixtime, Url};
use tungstenite::protocol::Message;
use zeroize::Zeroize;

fn main() {
    let encrypted_private_key = EncryptedPrivateKey("b1l/OWdYnR4fmbKYAqOJNO+efo2o4LeRhySKIyRDVIYBcYQ0jxO43IqbcZVDonTLD3KR/Tm/d34PqhzlhBnupg==".to_owned());

    let private_key = {
        let mut password = rpassword::prompt_password("Password: ").unwrap();
        let private_key = PrivateKey::import_encrypted(&encrypted_private_key, &password)
            .expect("Could not import private key hex string");
        password.zeroize();
        private_key
    };

    let public_key = private_key.public_key();

    let metadata = Metadata {
        name: Some("Michael Dilger".to_string()),
        about: Some("Author of Gossip client: https://github.com/mikedilger/gossip
Author of nostr-types rust library: https://crates.io/crates/nostr-types
New Zealander ex-Californian (UC Davis, Sun Microsystems). Programming since 1979.
".to_string()),
        picture: Some("https://avatars.githubusercontent.com/u/1669069".to_string()),
        nip05: Some("_@mikedilger.com".to_string()),
        lud16: Some("lnurl1dp68gurn8ghj7ampd3kx2ar0veekzar0wd5xjtnrdakj7tnhv4kxctttdehhwm30d3h82unvw qhkgetrv4h8gcn4dccnxv563ep".to_string()),
    };

    let pre_event = PreEvent {
        pubkey: public_key,
        created_at: Unixtime::now().unwrap(),
        kind: EventKind::Metadata,
        tags: vec![],
        content: serde_json::to_string(&metadata).unwrap(),
        ots: None
    };

    let event = Event::new(pre_event, &private_key).expect("Could not create event");

    let client_message = ClientMessage::Event(Box::new(event));

    let client_message_string = serde_json::to_string(&client_message)
        .expect("Could not serialize client message");

    println!("{}", client_message_string);

    let relay_urls = [
        "wss://nostr.mikedilger.com",
        "wss://nostr-pub.wellorder.net",
        "wss://nostr-relay.wlvs.space",
        "wss://nostr.oxtr.dev",
        "wss://relay.nostr.info",
        "wss://relay.damus.io"

        // HANGING "wss://relayer.fiatjaf.com",
        //"wss://nostr.onsats.org",
    ];

    for relay in relay_urls.iter() {
        let relay = *relay;

        let uri: http::Uri = relay.parse::<http::Uri>()
            .expect("Could not parse url");
        let authority = uri.authority().
            expect("Has no hostname").as_str();
        let host = authority
            .find('@')
            .map(|idx| authority.split_at(idx + 1).1)
            .unwrap_or_else(|| authority);
        if host.is_empty() {
            panic!("URL has empty hostname");
        }

        let key: [u8; 16] = rand::random();

        let request = http::request::Request::builder()
            .method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", base64::encode(key))
            .uri(uri)
            .body(())
            .expect("Could not build request");

        println!("Sending to {}", relay);

        let (mut websocket, _response) = tungstenite::connect(request)
            .expect("Could not connect to relay");

        websocket.write_message(Message::Text(client_message_string.clone()))
            .expect("Could not send message to relay");
    }

    println!("Done.");
}
