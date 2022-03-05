use crossterm::queue;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
};
use std::env;
use std::io::{stdout, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lightc::{
    client::Client,
    lcommand::{Lcmd, Lcommand},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 && args.get(1) == Some(&"server".to_string()) {
        // Run in server mode
        let addr: String = args.get(2).unwrap().to_string();
        run_server(addr);
    } else {
        // Run in client mode
        run_client();
    }
}

fn clear_screen(mut out: &std::io::Stdout) {
    // Clear screen
    queue!(out, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();
}

fn parse_cmd(buf: String, client: &mut Client) -> Lcommand {
    let cmd_split: Vec<&str> = buf.split(' ').collect();
    //dbg!(cmd_split[0]);
    let cmd_type = match cmd_split[0] {
        "/connect" => Lcmd::Conn,
        "/disconnect" => Lcmd::Dc,
        "/nick" => Lcmd::Nick,
        "/quit" => Lcmd::Quit,
        _ => Lcmd::Say,
    };
    let content = match cmd_type {
        Lcmd::Say => cmd_split.join(" "),
        _ => cmd_split[1..].join(" "),
    };

    if cmd_type == Lcmd::Nick {
        let old_username = client.username.clone();
        client.username = content.clone();
        client.messages.push(format!(
            "[CLIENT]: you changed your nickname to {}",
            client.username.clone()
        ));
        return Lcommand {
            cmd_type,
            user: old_username,
            content,
        };
    }
    Lcommand {
        cmd_type,
        user: client.username.clone(),
        content,
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

fn run_client() {
    let mut client = Client::new(String::from("test_user"));
    //client.connect(String::from("127.0.0.1:6969"));
    let mut prompt_text = String::new();
    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    crossterm::queue!(stdout, crossterm::terminal::Clear(ClearType::All)).unwrap();

    client.print_welcome();

    loop {
        // Get new received messages
        if client.connected.is_some() {
            let new_msg = client.rx.as_ref().unwrap().try_recv();
            if let Ok(new_msg) = new_msg {
                client.messages.push(new_msg.display(false));
            }
        }

        clear_screen(&stdout);

        client.display_messages(&stdout);

        let mut cmd: Option<Lcommand> = None;

        let received_char = client.user_in.try_recv();
        if let Ok(character) = received_char {
            if character == 0xA as char {
                // newline
                cmd = Some(parse_cmd(prompt_text.clone(), &mut client));
                prompt_text.clear();
            } else if character == 0x8 as char {
                // backspace
                prompt_text.pop();
            } else {
                prompt_text.push(character);
            }
        }
        client.print_prompt(&stdout, prompt_text.clone());
        // Send command
        if let Some(command) = cmd {
            if command.cmd_type == Lcmd::Quit {
                client.send_msg(command);
                break;
            } else if command.cmd_type == Lcmd::Conn {
                client.connect(command.content);
            } else {
                let success = client.send_msg(command.clone());
                if success {
                    client.messages.push(command.clone().display(true));
                } else {
                    client.connected = None;
                    client
                        .messages
                        .push("[CLIENT]: not connected to a server".to_string());
                }
            }
        }

        stdout.flush().unwrap();
        std::thread::sleep(Duration::from_millis(11));
    }

    // Make terminal normal again
    disable_raw_mode().unwrap();
}
