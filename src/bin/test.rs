use futures_util::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use bevy::{input::{keyboard::KeyboardInput, ButtonState}, prelude::*};

#[derive(Resource)]
struct TokioRuntimeHandle(tokio::runtime::Handle);

// websocket 연결 시 클라이언트 엔티티 생성 시 필요한 컴포넌트
#[derive(Component)]
struct Client;

// weboscket이 연결된 후 resource형태로 sink와 stream을 가지고 있도록 설정
#[derive(Resource)]
struct ConnectedWebsocket {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

#[derive(Event)]
struct SendEvent;

fn main() {
    // 처음에 시작 시 연결을 해주면 되는거아님? 

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (mut sink, stream) = runtime.handle().block_on(async move {
        connect_websocket().await
    });

    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<SendEvent>()
        .insert_resource(TokioRuntimeHandle(runtime.handle().clone()))
        .insert_resource(ConnectedWebsocket{
            sink,
            stream,
        })
        .add_systems(Update, keyboard_input_system)
        .run();
}

async fn connect_websocket() -> (SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    let (stream, res) = tokio_tungstenite::connect_async("ws://127.0.0.1:9003").await.unwrap();
    let (mut sink, ws_stream) = stream.split();        
    (sink, ws_stream)
}

fn setup_client_websocket(mut commands: Commands) {
    // Event도 설정해줘야함...? 
}

// keyboard input system
fn keyboard_input_system(mut keyboard_events: EventReader<KeyboardInput>, mut send_event: EventWriter<SendEvent>) {
    for event in keyboard_events.read() {
        if event.state == ButtonState::Pressed {
            println!("Key {:?} was pressed!", event.key_code);
            send_event.write(SendEvent);
        }
    }
}

fn send_event_system(mut send_event: EventReader<SendEvent>, client_res: Res<ConnectedWebsocket>, handle: Res<TokioRuntimeHandle>) {    
    for event in send_event.read() {
        // event 발생 시 websocket을 통해서 server로 보내준다.
        // handle을 이용해줘야하는 듯? 
        client_res.sink.send()
    }
}
