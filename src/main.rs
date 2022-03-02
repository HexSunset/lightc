use std::io;
use std::sync::mpsc;

#[derive(Debug)]
enum Lcmd {
    Conn,
    Dc,
    Say,
    Whisper,
}

#[derive(Debug)]
struct Lcommand {
    cmd_type: Lcmd,
    content: String,
}

impl Lcommand {
    fn new(buf: String) -> Lcommand {
        let cmd_split: Vec<&str> = buf.split(' ').collect();
        let cmd_type = match cmd_split[0] {
            "/connect" => Lcmd::Conn,
            "/disconnect" => Lcmd::Dc,
            "/whisper" => Lcmd::Whisper,
            _ => Lcmd::Say,
        };
        let content = match cmd_type {
            Lcmd::Say => cmd_split.join(" "),
            _ => cmd_split[1..].join(" "),
        };

        Lcommand { cmd_type, content }
    }
}

struct Client {
    username: String,
}

impl Client {
    fn new(user: String) -> Client {
        let (tx, rx) = mpsc::channel();
        Client {username: user}
    }
}

fn main() {
    let client = Client::new(String::from("test_user"));
    loop {
        println!("Enter command: ");
        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).unwrap();
        let lcmd = Lcommand::new(cmd);
        //dbg!(lcmd);
        match lcmd.cmd_type {
            Lcmd::Conn => println!("unimplemented command"),
            Lcmd::Dc => println!("unimplemented command"),
            Lcmd::Say => println!("unimplemented command"),
            Lcmd::Whisper => println!("unimplemented command"),
        }
    }
}
