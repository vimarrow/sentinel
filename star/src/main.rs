use std::io::Read;
use std::thread;
use std::{
    fs,
    path::{Path},
    os::unix::net::{UnixStream, UnixListener}
};

fn handle_client(stream: UnixStream) {
    println!("Incomming");
    //let payload = Vec::new();
    for byte in stream.bytes() {
        if let Ok(b) = byte {
            print!("{}", b);
        }
    }
    println!("Done");
}

fn main() {
    let socket = Path::new("/tmp/sentinel/star.sock");

    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let listener = match UnixListener::bind(&socket) {
        Err(_) => panic!("failed to bind socket"),
        Ok(listener) => listener,
    };

    println!("Server started, waiting for clients");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                println!("Error before spawn");
                println!("{:?}", err);
            }
        }
    }}

