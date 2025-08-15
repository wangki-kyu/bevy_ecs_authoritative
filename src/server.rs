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
                },
            }
        },
        Err(_) => {

        },
    }
}

// handler 
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

async fn handle_accept(stream: tokio::net::TcpStream, tx: Sender<ClientEventMessage>) {
    println!("[Websocket Recv] start handle websocket strream");
    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    let (_, stream) = ws_stream.split();
    let cloned_tx = tx.clone();
    let stream_future = stream.try_for_each(|msg| {
        if msg.is_empty() {
            return futures_util::future::ok(());
        }

        println!("message recevied!");
        let tx_in_future = cloned_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = tx_in_future.send(ClientEventMessage::Connect).await {
                eprintln!("ClientEventMessage send error: {}", e);
            }
        });
        
        futures_util::future::ok(())
    });

    stream_future.await.unwrap();

    println!("[Websocket Recv] finish handle websocket strream");
}
