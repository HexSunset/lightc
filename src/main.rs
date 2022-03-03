use std::io::{self, Write};
use std::net::TcpStream;
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
    user: String,
    content: String,
}

impl Lcommand {
    fn new(buf: String, user: String) -> Lcommand {
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

        let content = &content.as_str()[0..content.len() - 1];
        //dbg!(&content);

        Lcommand {
            cmd_type,
            user,
            content: content.to_string(),
        }
    }
}

struct Client {
    username: String,
    tx: Option<mpsc::Sender<Lcommand>>, // Channel to send messages to connected server
    rx: Option<mpsc::Receiver<Lcommand>>, // Channel to receive messages from connected server
}

impl Client {
    fn new(user: String) -> Client {
        Client {
            username: user,
            tx: None,
            rx: None,
        }
    }

    fn send_msg(&mut self, msg: Lcommand) {
        let mut msg = msg;
        msg.user = self.username.clone();
        self.tx.as_ref().unwrap().send(msg).unwrap();
    }

    fn connect(&mut self, addr: String) {
        let tx: mpsc::Sender<Lcommand>;
        let rx: mpsc::Receiver<Lcommand>;
        let channel = mpsc::channel();
        tx = channel.0;
        rx = channel.1;
        std::thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            loop {
                let mut end = false;
                let msg = rx.recv().unwrap();
                let mut out_buf = String::new();
                match msg.cmd_type {
                    Lcmd::Conn => out_buf.push_str("Conn\n"),
                    Lcmd::Dc => {out_buf.push_str("Dc\n"); end = true},
                    Lcmd::Say => out_buf.push_str("Say\n"),
                    Lcmd::Whisper => out_buf.push_str("Whisper\n"),
                }
                out_buf.push_str(&msg.user);
                out_buf.push('\n');
                out_buf.push_str(&msg.content);
                let _n = stream.write(out_buf.as_bytes()).unwrap();
                if end {
                    break;
                }
            }
        });
        self.tx = Some(tx);
    }
}

fn main() {
    let mut client = Client::new(String::from("test_user"));
    loop {
        println!("Enter command: ");
        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).unwrap();
        let lcmd = Lcommand::new(cmd, client.username.clone());

        match lcmd.cmd_type {
            Lcmd::Conn => client.connect(lcmd.content),
            _ => client.send_msg(lcmd),
        }
    }
}
