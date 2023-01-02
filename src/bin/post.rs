
use nostr_types::{ClientMessage, EncryptedPrivateKey, Event, EventKind, Id, PreEvent, PrivateKey, Tag, Unixtime, Url};
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

    // Text Note
//    let pubkey = PublicKey::try_from_hex_string("32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245")
//        .expect("Could not import public key");
    let pre_event = PreEvent {
        pubkey: public_key,
        created_at: Unixtime::now().unwrap(),
        kind: EventKind::TextNote,
        tags: vec![],
        /*
            Tag::Pubkey {
                pubkey,
                recommended_relay_url: Some(Url("wss://relay.damus.io".to_string())),
                petname: Some("jb55".to_owned()),
            }
        ],
        */
        content: "Testing proof of work".to_owned(),
        ots: None
    };

    // Reply
    /*
    let reply_to_id = Id::try_from_hex_string("ca8537625e94a4095051469a43cefa6431107ab259bbdbb23d764058f2cbc77d")
        .expect("Could not import event Id");
    let pre_event = PreEvent {
        pubkey: public_key,
        created_at: Unixtime::now().unwrap(),
        kind: EventKind::TextNote,
        tags: vec![
            Tag::Event {
                id: reply_to_id,
                recommended_relay_url: Some(Url("wss://nostr-pub.wellorder.net".to_owned())),
                marker: Some("reply".to_owned())
            }
        ],
        content: "I've used astral heaps. Thanks for it. I hear you about being overwhelmed. My client gossip is still even not born yet, has zero users (presumably), and I have 32 open issues!".to_owned(),
        ots: None
    };
     */

    // Reaction
    /*
    let react_to_id = Id::try_from_hex_string("09e253bd8b614720c1c099c7d6bb046cb1f8d519ab3602d1f6785da1e6f9874e")
        .expect("Could not import event Id");
    let pre_event = PreEvent {
        pubkey: public_key,
        created_at: Unixtime::now().unwrap(),
        kind: EventKind::Reaction,
        tags: vec![
            Tag::Event {
                id: react_to_id,
                recommended_relay_url: Some(Url("wss://relay.damus.io".to_string())),
                marker: None
            }
        ],
        content: "+".to_owned(),
        ots: None
    };
     */

    let event = Event::new_with_pow(pre_event, &private_key, 24).expect("Could not create event");

    let client_message = ClientMessage::Event(Box::new(event));

    let client_message_string = serde_json::to_string(&client_message)
        .expect("Could not serialize client message");

    println!("{}", client_message_string);

    let relay_urls = [
        // monlovesmango:
        "wss://nostr.mikedilger.com",
        "wss://nostr-pub.wellorder.net",
        "wss://nostr-relay.wlvs.space",
        // DOWN "wss://nostr.oxtr.dev",
        "wss://relay.nostr.info",
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
