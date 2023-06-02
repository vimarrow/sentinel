use kv::{Config, Store, Bucket, Value, Key};
use std::{
    io::{Read, Write},
    sync::{Arc, RwLock},
    os::unix::net::{UnixStream, UnixListener, SocketAddr},
    path::Path,
    thread, usize, fs, time::Duration,
};

static BUCKETS: [&'static str; 10] = ["config", "groups", "users", "flows", "actions", "params", "tables", "queries", "files", "links"];

fn get_bucket<'a, K: Key<'a>, V: Value>(buffer: &[u8], store: &Arc<RwLock<Store>>) -> Result<Bucket<'a, K, V>, u8> {
    let bucket_raw = usize::from(buffer[1]);
    if bucket_raw >= BUCKETS.len() {
        println!("Invalid bucket requested!");
        return Err(129);
    }
    let readable = match store.read() {
        Ok(r) => r,
        Err(err) => {
            println!("{:?}", err);
            println!("Failed to get readable!");
            return Err(130);
        }
    };
    match readable.bucket::<K, V>(Some(BUCKETS[bucket_raw])) {
        Ok(b) => Ok(b),
        Err(err) => {
            println!("{:?}", err);
            println!("Failed to get bucket!");
            return Err(131);
        }
    }
}

fn handle_client(mut stream: UnixStream, store: Arc<RwLock<Store>>) {
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
        let bucket = match get_bucket::<Vec<u8>, Vec<u8>>(&buf, &store) {
            Ok(b) => b,
            Err(flag) => {
                stream.write_all(&[cmd, flag]).unwrap();
                continue;
            }
        };
        if count < 18 {
            stream.write_all(&[cmd, 136]).unwrap();
            println!("Bad length!");
            continue;
        }
        let key = &buf[2..18].to_vec();

        match cmd {
            1 => { // GET Key
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
            2 => { // SET Key
                let mut payload = vec![];
                for i in 18..count {
                    payload.insert(i-18, buf[i]);
                }
                let value = bucket.set(key, &payload);
                match value {
                    Ok(_) => {
                        stream.write_all(&[cmd, 0]).unwrap();
                    },
                    Err(err) => {
                        stream.write_all(&[cmd, 134]).unwrap();
                        println!("{:?}", err);
                    }
                }
                let flush_op = bucket.flush();
                if flush_op.is_err() {
                    println!("Failed to flush");
                }
            },
            3 => { // DEL Key
                let key = &buf[2..18].to_vec();
                let value = bucket.remove(key);
                match value {
                    Ok(_) => {
                        stream.write_all(&[cmd, 0]).unwrap();
                    },
                    Err(err) => {
                        stream.write_all(&[cmd, 135]).unwrap();
                        println!("{:?}", err);
                    }
                }
                let flush_op = bucket.flush();
                if flush_op.is_err() {
                    println!("Failed to flush");
                }
            },
            _ => {
                println!("UNKNOWN CMD!");
                stream.write_all(&[cmd, 128]).unwrap();
            },
        }
    }
    println!("DONE! {:?}", addr);
}

fn main() {
    let socket = Path::new("/tmp/sentinel/star.sock");

    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let cfg = Config::new("/tmp/sentinel/start.bin");

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
                let store_instance = Arc::clone(&store);
                thread::spawn(move || handle_client(stream, store_instance));
            }
            Err(err) => {
                println!("Error before spawn");
                println!("{:?}", err);
            }
        }
    }
}

// use idgenerator::{IdGeneratorOptions, IdInstance};
// let options = IdGeneratorOptions::new().worker_id(7321);
// let _ = IdInstance::init(options).unwrap();
// let id = IdInstance::next_id();
// let key_for_storage = id.to_be_bytes();
