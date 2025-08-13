use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;


#[tokio::main]
async fn main() {
    let (stream, res) = tokio_tungstenite::connect_async("ws://127.0.0.1:9003").await.unwrap();

    let (mut sink, ws_stream) = stream.split();    
    let msg = Message::text("hi");
    sink.send(msg).await.unwrap();
    sink.send(Message::Close(None)).await.unwrap();
}