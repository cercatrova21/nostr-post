
use nostr_types::{ClientMessage, Filter, RelayMessage, SubscriptionId};
use tungstenite::protocol::Message;

fn main() {
    let filter = Filter::new();
    let message = ClientMessage::Req(SubscriptionId("dump".to_owned()), vec![filter]);
    let wire = serde_json::to_string(&message)
        .expect("Could not serialize message");

    let relay_url = "wss://nostr.mikedilger.com";
    let uri: http::Uri = relay_url.parse::<http::Uri>()
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

    let (mut websocket, _response) = tungstenite::connect(request)
        .expect("Could not connect to relay");

    websocket.write_message(Message::Text(wire))
        .expect("Could not send message to relay");

    loop {
        let message = websocket.read_message()
            .expect("Problem reading from websocket");

        match message {
            Message::Text(s) => {
                let relay_message: RelayMessage = serde_json::from_str(&s)
                    .expect("Unable to deserialize RelayMessage");
                match relay_message {
                    RelayMessage::Event(_,e) => println!("{}", serde_json::to_string(&e).
                                                         expect("Cannot serialize event")),
                    RelayMessage::Notice(s) => println!("NOTICE: {}", s),
                    RelayMessage::Eose(_) => {
                        let message = ClientMessage::Close(SubscriptionId("dump".to_owned()));
                        let wire = serde_json::to_string(&message)
                            .expect("Could not serialize message");
                        websocket.write_message(Message::Text(wire))
                            .expect("Could not write close subscription message");
                        websocket.write_message(Message::Close(None))
                            .expect("Could not write websocket close message");
                    },
                    RelayMessage::Ok(id, ok, reason) =>
                        println!("OK: ok={} reason={}", ok, reason),
                }
            },
            Message::Binary(_) => println!("IGNORING BINARY MESSAGE"),
            Message::Ping(vec) => websocket.write_message(Message::Pong(vec))
                .expect("Unable to write message"),
            Message::Pong(_) => println!("IGNORING PONG"),
            Message::Close(_) => {
                println!("Closing");
                break;
            },
            Message::Frame(_) => println!("UNEXPECTED RAW WEBSOCKET FRAME"),
        }
    }
}
