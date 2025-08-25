use std::sync::mpsc::channel;

use futures_util::{stream::{SplitSink, SplitStream}, task, SinkExt, StreamExt, TryStreamExt};
use tokio::{net::TcpStream, sync::mpsc::Sender};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use bevy::{color::palettes::css::RED, input::{keyboard::KeyboardInput, ButtonState}, prelude::*};


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

#[derive(Resource)]
struct WebsocketChannelSender(Sender<String>);

#[derive(Event)]
struct SendEvent(MoveDirection);

enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

impl MoveDirection {
    fn to_string(&self) -> String {
        match self {
            MoveDirection::Up => "up".to_string(),
            MoveDirection::Down => "down".to_string(),
            MoveDirection::Left => "left".to_string(),
            MoveDirection::Right => "right".to_string(),
        }
    }
}

#[derive(Component)]
struct Ball;

fn main() {
    // -------- tokio runtime 생성
    let runtime = tokio::runtime::Runtime::new().unwrap();
    // -------- websocket connect task 생성..
    let sender = runtime.handle().block_on(async move {
        connect_websocket().await
    });

    // -------- bevy App initialize
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<SendEvent>()
        .insert_resource(TokioRuntimeHandle(runtime.handle().clone()))
        .insert_resource(WebsocketChannelSender(sender))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            keyboard_input_system,
            send_event_system,
            )
        )
        .run();
}

// region: --websocket
/// 어플리케이션 실행 시 WebSocket 연결 함수
/// stream, sink를 처리하는 task를 각각 생성한다.
/// Sender<String>을 반환하여 Bevy의 resource로 만들어 Bevy App에서 사용하도록 하였음. 
/// 
async fn connect_websocket() -> tokio::sync::mpsc::Sender<String> {
    println!("waiting for connecting to server!");
    let (stream, res) = tokio_tungstenite::connect_async("ws://127.0.0.1:9003").await.unwrap();

    println!("websocket connect success!!");

    let (mut sink, ws_stream) = stream.split();   

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<String>(10);

    // start websocket stream receive task 
    let sender_clone = sender.clone();
    tokio::spawn(async move {
        println!("[Start] WebSocket Stream");
        handle_websocket_stream(ws_stream, sender_clone).await;
    });

    // start websocket sink sender task
    tokio::spawn(async move {
        println!("[Start] WebSocket Sink");
        handle_websocket_sink(sink, receiver).await;
    });

    sender
}

// websocket 받기
// todo: websocket으로 받은 내용을 bevy에게 전달하는 로직이 필요함, 아직 구현되어있지 않음
async fn handle_websocket_stream(stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, sender: tokio::sync::mpsc::Sender<String>) {
    // websocket stream으로 받은 데이터를 처리하는 hander 
    let future_stream = stream.try_for_each(|msg| {
        futures_util::future::ok(())
    });

    let _ = future_stream.await;
}

/// websocket 보내기
/// sink를 통해서 연결된 websocket server로 데이터를 보내는 handler 함수
/// 
async fn handle_websocket_sink(mut sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, mut receiver: tokio::sync::mpsc::Receiver<String>) {
    // mpsc receiver를 통해서 받은 데이터를 websocker sink로 보내는 handler 
    loop {
        match receiver.recv().await {
            Some(msg) => {
                // msg의 헤더에 따라서 
                // 보내는 데이터가 달라진다.
                println!("msg: {}", msg);
                let _ = sink.send(Message::text(msg)).await;
            },
            None => {
    
            },
        }
    }
}
// endregion: --websocket

// region: -- setup
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera setting
    commands.spawn(Camera2d);

    commands.spawn((
        // 1. 메시 컴포넌트: 어떤 모양을 그릴지 정의
        Mesh2d(meshes.add(Circle::new(50.0))),
        // 2. 재질 컴포넌트: 메시의 색상, 텍스처 등을 정의
        MeshMaterial2d(materials.add(ColorMaterial::from_color(RED))),
        // 3. 트랜스폼 컴포넌트: 엔티티의 위치, 회전, 크기를 정의
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Ball,
    ));
}

// endregion: -- setup

// region: -- system

// Ball move system
fn ball_move_system() {
        
}

// keyboard input system
fn keyboard_input_system(mut keyboard_events: EventReader<KeyboardInput>, mut send_event: EventWriter<SendEvent>, mut query: Query<&mut Transform, With<Ball>>) {
    for event in keyboard_events.read() {
        if event.state == ButtonState::Pressed {
            println!("Key {:?} was pressed!", event.key_code);
            // let mut transform = query.single_mut().unwrap();

            // ------- key_code convert to move_direction 
            let move_direction = match event.key_code {
                KeyCode::ArrowUp => {
                    // transform.translation.y += 10.0;
                    Some(MoveDirection::Up)
                },
                KeyCode::ArrowDown => {
                    // transform.translation.y -= 10.0;
                    Some(MoveDirection::Down)
                },
                KeyCode::ArrowLeft => {
                    // transform.translation.x -= 10.0;
                    Some(MoveDirection::Left)
                },
                KeyCode::ArrowRight => {
                    // transform.translation.x += 10.0;
                    Some(MoveDirection::Right)
                },
                _ => {
                    None
                }
            };

            if let Some(direction) = move_direction {
                send_event.write(SendEvent(direction));    
            }
        }
    }
}

/// 키보드 input event receiver handler system
/// tokio runtime handle을 이용하여 send task를 생성해 준다. 
/// 미리 생성해둔 resouce인 WebSocketChannelSender로 보내주면 됨.
fn send_event_system(mut send_event: EventReader<SendEvent>, websocket_sender: Res<WebsocketChannelSender>, handle: Res<TokioRuntimeHandle>) {    
    for event in send_event.read() {
        // event 발생 시 websocket을 통해서 server로 보내준다.
        // handle을 이용해줘야하는 듯? 
        let directino_str = event.0.to_string();
        let sender_clone = websocket_sender.0.clone();
        handle.0.spawn(async move {
            // msg 생성 필요
            // up, down, left, right 
            match sender_clone.send(directino_str).await {
                Ok(_) => {
                    println!("send success!!");
                },
                Err(e) => {
                    eprintln!("[send_event_system] fail to send, error: {}", e);
                },
            };
        });
    }
}
// endregion: -- system