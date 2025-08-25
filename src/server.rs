use std::collections::HashMap;

use bevy::prelude::*;
use futures_util::{future, stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
use tokio::{net::TcpStream, sync::mpsc::{Receiver, Sender}};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use crate::common::ServerMessage;

pub fn run_server() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(TokioRuntime(runtime.handle().clone()))
        .insert_resource(UuidMap(HashMap::new()))
        .add_event::<ClientMoveEvent>()
        .add_event::<SinkEvent>()
        .add_systems(Startup, setup_server)
        .add_systems(Update, (
            clinet_event_receive_system,
            client_move_event_system
        ))
        .run();
}

struct ClientConnectInfo {
    uuid: Uuid,
    sender: Sender<Message>,
}

impl ClientConnectInfo {
    pub fn new(uuid: Uuid, sender: Sender<Message>) -> Self {
        ClientConnectInfo { uuid, sender }
    }
}

// enum
enum ClientEventMessage {
    Connect(ClientConnectInfo), // 연결
    Move(MoveDirection, Uuid),
}

enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl From<&str> for MoveDirection {
    fn from(value: &str) -> Self {
        match value {
            "up" => MoveDirection::Up,
            "down" => MoveDirection::Down,
            "left" => MoveDirection::Left,
            "right" => MoveDirection::Right,
            _ => MoveDirection::None,
        }
    }
}

// ----------------- component
#[derive(Component)]
struct Client(Uuid);

#[derive(Component)]
struct ClientSender(Sender<Message>);

// ----------------- resource

#[derive(Resource)]
struct TokioRuntime(tokio::runtime::Handle);

#[derive(Resource)]
struct WebSocketAcceptEvent(Receiver<ClientEventMessage>);  // Websocket으로 받아온 데이터를 처리해야함..

#[derive(Resource)]
struct WebSocketSinkEvent(Sender<Message>);

#[derive(Resource)]
struct UuidMap(pub HashMap<Uuid, Entity>);

// ----------------- event
#[derive(Event)]
struct ClientMoveEvent{
    uuid: Uuid,
    move_direction: MoveDirection, 
}

#[derive(Event)]
struct SinkEvent;

// ----------------- system
fn setup_server(mut commands: Commands, tokio_runtime: Res<TokioRuntime>) {
    // websocket server Message channel
    let (stream_tx, stream_rx) = tokio::sync::mpsc::channel::<ClientEventMessage>(10);
    let (sink_tx, sink_rx) = tokio::sync::mpsc::channel::<Message>(10);

    // resource 추가 
    commands.insert_resource(WebSocketAcceptEvent(stream_rx));
    commands.insert_resource(WebSocketSinkEvent(sink_tx));

    let handle = tokio_runtime.0.clone();
    handle.spawn(async move {
        handle_websocket(stream_tx).await;
        println!("finish the websocker waiting...");
    });
}

/// ClinetEventMessage
/// ## Connect: 
/// Component
/// - Client(Uuid): 클라이언트 식별
/// - ClientSink(sink): 연결된 클라이언트에게 데이터 전송
/// - Transform: 위치 정보 
/// 
/// todo here ...
fn clinet_event_receive_system(mut commands: Commands, mut recv: ResMut<WebSocketAcceptEvent>, mut client_move_event: EventWriter<ClientMoveEvent>, mut uuid_map: ResMut<UuidMap>) {
    match recv.0.try_recv() {
        Ok(msg) => {
            match msg {
                ClientEventMessage::Connect(info) => {
                    println!("client connect success!!, it will make client entity");
                    // Client entity 생성: Transform Componenet를 가지고 있어야함
                    // 이 후 방향 메시지가 왔을 때, 해당 Transform 위치를 변경시켜줘야함.
                    let entity = commands.spawn((
                        Client(info.uuid),
                        ClientSender(info.sender),
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    ));

                    // uuid - entity 추가 
                    uuid_map.0.insert(info.uuid, entity.id());
                },
                ClientEventMessage::Move(move_event, uuid) => {
                    // todo here ...
                    // client move event 발생!
                    // event를 발생시켜서 위치를 변경시키는 작업을 하면 좋을 것 같음. 
                    match move_event {
                        MoveDirection::Up => println!("up"),
                        MoveDirection::Down => println!("down"),
                        MoveDirection::Left => println!("left"),
                        MoveDirection::Right => println!("right"),
                        MoveDirection::None => print!("none"),
                    }

                    // client move event write!
                    client_move_event.write(ClientMoveEvent{ uuid: uuid,move_direction: move_event});
                },
            }
        },
        Err(_) => {

        },
    }
}

fn client_move_event_system(mut client_move_event: EventReader<ClientMoveEvent>, mut query: Query<(&mut Transform, &ClientSender, &Client)>, tokio_handle: Res<TokioRuntime>, uuid_map: Res<UuidMap>) {
    
    for event in client_move_event.read() {
        // 1. client Entity의 transform component 값을 변경시킨다.
        // 2. 변경된 위치 좌표를 ClientSink Component의 sink 값을 통해서 보내준다.
        let entity = uuid_map.0.get(&event.uuid).unwrap();
        let (mut transform, sender, _) = query.get_mut(*entity).unwrap();
        
        match event.move_direction {
            MoveDirection::Up => transform.translation.y += 10.0,
            MoveDirection::Down => transform.translation.y -= 10.0,
            MoveDirection::Left => transform.translation.x -= 10.0,
            MoveDirection::Right => transform.translation.x += 10.0,
            MoveDirection::None => {},
        }
        println!("move event occur!");

        let cloned_transform = transform.translation.clone();
        let cloned_tx = sender.0.clone();

        tokio_handle.0.spawn(async move {
            let server_msg = ServerMessage::PlayerUpdate { translation: cloned_transform };
            let json_str = serde_json::to_string(&server_msg).unwrap();
            println!("json: {}", json_str);
            let msg = Message::text(json_str);
            match cloned_tx.send(msg).await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("error: {}", e);
                },
            }
        });
    }    
}

// handler 
/// StartUp 시에 클라이언트의 접속을 처리해주는 함수 
/// 성공적으로 연결이되면 `stream`을 새로운 task로 넘겨준다. 새로 생성된 task에서는 `handle_accept`를 호출해서 처리해준다.
async fn handle_websocket(tx: Sender<ClientEventMessage>) {
    let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:9003").await.unwrap();

    loop {
        let cloned_tx = tx.clone();
        match tcp_listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move { 
                    handle_accept(stream, cloned_tx).await;
                });
            },
            Err(e) => {
                eprintln!("accept error occured!, err: {}", e);
            },
        }
    }
}

/// 
/// 연결된 각 클라이언트마다 task로 존재함.
async fn handle_accept(stream: tokio::net::TcpStream, tx: Sender<ClientEventMessage>) {
    println!("[Websocket Recv] start handle websocket strream");
    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();

    // uuid generate
    let uuid = uuid::Uuid::new_v4();
    println!("uuid: {}", uuid);

    let (sink, stream) = ws_stream.split();
    let (sink_tx, sink_recv) = tokio::sync::mpsc::channel::<Message>(100);

    // -------- Entity를 생성하기 위해서 메시지를 보내준다? 
    // Uuid는 Clone, Copy가 구현되어있으므로 자동으로 값복사가 일어나서 소유권 이동이 발생하지 않는다.
    match tx.send(ClientEventMessage::Connect(ClientConnectInfo::new(uuid, sink_tx))).await {
        Ok(_) => {
            
        },
        Err(e) => {
            eprintln!("fail to send message that requests to make client entity, error: {}", e);
            return;
        },
    }
    
    let cloned_tx = tx.clone();
    let stream_future = stream.try_for_each(|msg| {
        if msg.is_empty() {
            return futures_util::future::ok(());
        }

        println!("message recevied!, msg: {}", msg);

        // 여기서 만약에 msg가 연결에 대한 요청이라면 entity를 만들어주고 
        // 다른 내용이라면 내용에 따라서 처리를 해주어야한다.
        let tx_in_future = cloned_tx.clone();
        tokio::spawn(async move {
            let msg_str = msg.to_text().unwrap();
            if let Err(e) = tx_in_future.send(ClientEventMessage::Move(msg_str.into(), uuid)).await {
                eprintln!("ClientEventMessage send error: {}", e);
            }
        });
        
        futures_util::future::ok(())
    });

    // sink task generate
    tokio::spawn(async move {
        println!("sink loop start!");
        sink_handler(sink_recv, sink).await;
    });

    stream_future.await.unwrap();

    println!("[Websocket Recv] finish handle websocket strream");
}

/// sink handler 
/// 
async fn sink_handler(mut recv: Receiver<Message>, mut sink: SplitSink<WebSocketStream<TcpStream>, Message>) {
    println!("wait for recv sink message");
    while let Some(msg) = recv.recv().await {
        sink.send(msg).await.unwrap();
    }
}