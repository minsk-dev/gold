extern crate core;

use std::collections::HashMap;
use std::env;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;

use async_recursion::async_recursion;
use json::object::Object;
use json::JsonValue;
use tokio::io::Result;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::server::ServerMode;

mod http;
mod resp;
mod server;

pub type Database = Arc<Mutex<HashMap<String, Object>>>;

#[tokio::main]
async fn main() -> Result<()> {
    // For some reason, 'localhost' is not the same as '127.0.0.1'.
    // Using 'localhost' doesn't work.
    let server_addr = "0.0.0.0:5445";
    match TcpListener::bind(server_addr).await {
        Ok(listener) => {
            println!("listening on '{}'...", server_addr);
            let db: Database = Arc::new(Mutex::new(HashMap::new()));

            // TODO: Use a CLI framework
            let mode = match env::args().nth(1) {
                Some(arg) => match arg.as_str() {
                    "--resp" => ServerMode::RESP,
                    _ => ServerMode::HTTP,
                },
                None => ServerMode::HTTP,
            };

            mode.run(listener, db).await?;
        }
        Err(error) => match error.kind() {
            ErrorKind::AddrInUse => eprintln!("address {} is already in use.", server_addr),
            _ => eprintln!("{:?}", error.to_string()),
        },
    }

    Ok(())
}

#[async_recursion]
async fn update_db(db: &Database, data: JsonValue, key: String, socket_addr: Option<SocketAddr>) {
    match data {
        JsonValue::Object(data) => {
            let mut db = db.lock().await;
            db.insert(key, data);
        }
        _ => {
            if socket_addr.is_none() {
                eprintln!("invalid JSON provided. only objects and arrays are accepted");
                return;
            }

            eprintln!(
                "[{}]: invalid JSON provided. only objects and arrays are accepted",
                socket_addr.unwrap()
            )
        }
    }
}

async fn query_db(db: &Database, key: String) -> Option<Object> {
    db.lock().await.get(&key).cloned()
}
