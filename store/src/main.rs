#![feature(box_into_inner)]

use std::{
    fs,
    thread,
    io::{Write, Read},
    path::Path,
    time::Duration,
    sync::{Arc, RwLock},
    os::unix::net::{UnixStream, UnixListener, SocketAddr}
};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use serde::{Deserialize, Serialize};
use kv::{Config, Store, Bucket, Value, Key};

#[derive(Debug, Serialize, Deserialize)]
struct StatementVariables {
    key: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReceivedStatement {
    statement: String,
    vars: Vec<String>,
}

fn get_bucket<'a, K: Key<'a>, V: Value>(store: &Arc<RwLock<Store>>) -> Result<Bucket<'a, K, V>, u8> {
    let readable = match store.read() {
        Ok(r) => r,
        Err(err) => {
            println!("{:?}", err);
            println!("Failed to get readable!");
            return Err(130);
        }
    };
    match readable.bucket::<K, V>(Some("Default")) {
        Ok(b) => Ok(b),
        Err(err) => {
            println!("{:?}", err);
            println!("Failed to get bucket!");
            return Err(131);
        }
    }
} 

fn handle_client(mut stream: UnixStream, _pool: Pool<SqliteConnectionManager>, store: Arc<RwLock<Store>>) {
    let addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => SocketAddr::from_pathname("/unkwonw").unwrap()
    };
    println!("Incomming: {:?}", addr);
    loop {
        let mut buf: [u8; 1048594] = [0; 1048594];
        let count = match stream.read(&mut buf) {
            Ok(size) => size,
            Err(err) => {
                println!("Error on read!");
                println!("{:?}", err.to_string());
                break;
            }
        };
        if count == 0 { // 0 means EOF package
            break;
        }
        let cmd = buf[0];
        println!("Got: {:?}", cmd);
        let bucket = match get_bucket::<Vec<u8>, Vec<u8>>(&store) {
            Ok(b) => b,
            Err(flag) => {
                stream.write_all(&[cmd, flag]).unwrap();
                continue;
            }
        };
        if count < 17 {
            stream.write_all(&[cmd, 136]).unwrap();
            println!("Bad length!");
            continue;
        }
        let key = &buf[1..17].to_vec();
        match cmd {
            1 => { // Define new statement
                let value = bucket.get(key);
                match value {
                    Ok(res) => {
                        match res {
                            Some(val) => {
                                let mut resp: Vec<u8> = vec![cmd, 0];
                                resp.append(&mut val.clone());
                                stream.write_all(&resp).unwrap();
                            },
                            None => {
                                stream.write_all(&[cmd, 132]).unwrap();
                            }
                        }
                    },
                    Err(err) => {
                        stream.write_all(&[cmd, 133]).unwrap();
                        println!("{:?}", err);
                    }
                }
            },
            _ => {
                println!("UNKNOWN CMD!");
                stream.write_all(&[cmd, 128]).unwrap();
            },
        }
    }
    println!("DONE! {:?}", addr);
    println!("Done");
}

fn main() {
    let socket = Path::new("/tmp/sentinel/store.sock");

    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let manager = SqliteConnectionManager::file("/tmp/sentinel/store.db");
    let pool = r2d2::Pool::new(manager).unwrap();

    let cfg = Config::new("/tmp/sentinel/store.bin");
    let store = Arc::new(RwLock::new(Store::new(cfg).unwrap()));

    let listener = match UnixListener::bind(&socket) {
        Err(_) => panic!("failed to bind socket"),
        Ok(listener) => listener,
    };

    println!("Star started, waiting for clients");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                stream.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
                let pool = pool.clone();
                let store_instance = Arc::clone(&store);
                thread::spawn(move || handle_client(stream, pool, store_instance));
            }
            Err(err) => {
                println!("Error before spawn");
                println!("{:?}", err);
            }
        }
    }
}


