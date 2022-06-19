use tungstenite::{connect, Message};

pub async fn fetch_ws(
    url: String,
    header: HashMap<String, String>,
) -> Vec<ResourceData> {
    use url::Url;

    let (mut socket, response) =
        connect_async(Url::parse("wss://data.alpaca.markets/stream").unwrap())
            .expect("Can't connect");

    socket.write_message(Message::Text(
        r#"{
            "action": "authenticate",
            "data": {
                "key_id": "API-KEY",
                "secret_key": "SECRET-KEY"
            }
        }"#
        .into(),
    ));

    socket.write_message(Message::Text(
        r#"{
            "action": "listen",
            "data": {
                "streams": ["AM.SPY"]
            }
        }"#
        .into(),
    ));

    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
}
