
use nostr_proto::{ClientMessage, Event, EventKind, PreEvent, PrivateKey, Unixtime};
use tungstenite::protocol::Message;
use zeroize::Zeroize;

fn main() {
    let encrypted_private_key = "b1l/OWdYnR4fmbKYAqOJNO+efo2o4LeRhySKIyRDVIYBcYQ0jxO43IqbcZVDonTLD3KR/Tm/d34PqhzlhBnupg==";

    let private_key = {
        let mut password = rpassword::prompt_password("Password: ").unwrap();
        let private_key = PrivateKey::import_encrypted(encrypted_private_key, &password)
            .expect("Could not import private key hex string");
        password.zeroize();
        private_key
    };

    let public_key = private_key.public_key();

    let pre_event = PreEvent {
        pubkey: public_key,
        created_at: Unixtime::now().unwrap(),
        kind: EventKind::TextNote,
        tags: vec![],
        content: "Test X7b".to_owned(),
        ots: None
    };

    let event = Event::new(pre_event, private_key).expect("Could not create event");

    let client_message = ClientMessage::Event(Box::new(event));

    let client_message_string = serde_json::to_string(&client_message)
        .expect("Could not serialize client message");

    println!("{}", client_message_string);

    let relay_urls = [
        //"wss://nostr.mikedilger.com",
        //"wss://nostr-pub.wellorder.net",
        //"wss://nostr-relay.wlvs.space",
        // DOWN "wss://nostr.oxtr.dev",
        //"wss://relay.nostr.info",
        // HANGING "wss://relayer.fiatjaf.com",
        "wss://nostr.onsats.org",
        "wss://relay.damus.io"
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
