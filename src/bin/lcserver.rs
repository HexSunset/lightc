use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::env;

use lightc::lcommand::{Lcmd, Lcommand};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        // Run in server mode
        let addr: String = args.get(1).unwrap().to_string();
        run_server(addr);
    }
}

fn run_server(addr: String) {
    let listener = TcpListener::bind(&addr).unwrap();
    println!("Listening on {}", addr);

    let users: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(vec![]));
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let users = Arc::clone(&users);
        users.lock().unwrap().push(stream.try_clone().unwrap());
        std::thread::spawn(move || {
            let mut buf: Vec<u8> = vec![0; 1024];
            let mut end = false;
            loop {
                let socket = stream.peer_addr().unwrap();
                let n = stream.read(&mut buf);
                if n.is_err() {
                    break;
                }
                let cmd = Lcommand::from(String::from_utf8(buf.clone()).unwrap());
                let my_user = cmd.user.clone();
                if cmd.cmd_type == Lcmd::Dc {
                    end = true;
                }
                let mut users_unlocked = users.lock().unwrap();
                for i in 0..users_unlocked.len() {
                    let socket_other = users_unlocked[i].peer_addr();
                    if socket_other.is_err() || socket_other.unwrap() == socket {
                        continue;
                    }
                    let n = users_unlocked[i].write(&buf);
                    if n.is_err() {
                        continue; // TODO: remove invalid streams from the list
                    } else {
                        println!("{} sent {} bytes", my_user, n.unwrap());
                    }
                }
                if end {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        });
    }
}
