use bevy::prelude::*;
use futures_util::{future, StreamExt, TryStreamExt};
use tokio::sync::mpsc::{Receiver, Sender};

pub fn run_server() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(TokioRuntime(runtime.handle().clone()))
        .add_systems(Startup, setup_server)
        .add_systems(Update, (
            clinet_event_receive_system,
        ))
        .run();
}

// enum
enum ClientEventMessage {
    Connect, // 연결
    Move(MoveDirection),
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
struct Client;

// ----------------- resource

#[derive(Resource)]
struct TokioRuntime(tokio::runtime::Handle);

#[derive(Resource)]
struct WebSocketAcceptEvent(Receiver<ClientEventMessage>);  // Websocket으로 받아온 데이터를 처리해야함..

// ----------------- system
fn setup_server(mut commands: Commands, tokio_runtime: Res<TokioRuntime>) {
    // websocket server Message channel
    let (tx, rx) = tokio::sync::mpsc::channel::<ClientEventMessage>(10);

    // resource 추가 
    commands.insert_resource(WebSocketAcceptEvent(rx));

    let handle = tokio_runtime.0.clone();
    handle.spawn(async move {
        handle_websocket(tx).await;
        println!("finish the websocker waiting...");
    });
}

fn clinet_event_receive_system(mut recv: ResMut<WebSocketAcceptEvent>) {
    match recv.0.try_recv() {
        Ok(msg) => {
            match msg {
                ClientEventMessage::Connect => {
                    println!("client connect success!!, it will make client entity");
                },
                ClientEventMessage::Move(move_direction) => {
                    // todo here ...
                    match move_direction {
                        MoveDirection::Up => println!("up"),
                        MoveDirection::Down => println!("down"),
                        MoveDirection::Left => println!("left"),
                        MoveDirection::Right => println!("right"),
                        MoveDirection::None => print!("none"),
                    }
                },
            }
        },
        Err(_) => {

        },
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
                tokio::spawn(async {
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
/// 
async fn handle_accept(stream: tokio::net::TcpStream, tx: Sender<ClientEventMessage>) {
    println!("[Websocket Recv] start handle websocket strream");
    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();

    // -------- Entity를 생성하기 위해서 메시지를 보내준다? 
    match tx.send(ClientEventMessage::Connect).await {
        Ok(_) => {
            
        },
        Err(e) => {
            eprintln!("fail to send message that requests to make client entity, error: {}", e);
            return;
        },
    }
    
    let (_, stream) = ws_stream.split();
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
            if let Err(e) = tx_in_future.send(ClientEventMessage::Move(msg_str.into())).await {
                eprintln!("ClientEventMessage send error: {}", e);
            }
        });
        
        futures_util::future::ok(())
    });

    stream_future.await.unwrap();

    println!("[Websocket Recv] finish handle websocket strream");
}
