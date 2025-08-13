use std::env;

use crate::{client::run_client, server::run_server};

mod client;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.is_empty() {
        println!("args is empty, plz write server or client!");
        return;
    }

    println!("args: {}", args[1]);

    match args[1].as_str() {
        "server" => {
            run_server();
        },
        "client" => {
            run_client();
        },
        _ => {
            println!("fault params!");
        }
    }
}
