use crossterm::queue;
use crossterm::terminal::enable_raw_mode;
use crossterm::{
    cursor,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use std::io::{stdout, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::time::Duration;

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
    // Construct command from user input
    fn new(buf: String, user: String) -> Lcommand {
        let cmd_split: Vec<&str> = buf.split(' ').collect();
        //dbg!(cmd_split[0]);
        let cmd_type = match cmd_split[0] {
            "/connect" => Lcmd::Conn,
            "/disconnect\n" => Lcmd::Dc,
            "/whisper" => Lcmd::Whisper,
            _ => Lcmd::Say,
        };
        let mut content = match cmd_type {
            Lcmd::Say => cmd_split.join(" "),
            _ => cmd_split[1..].join(" "),
        };

        if !content.is_empty() {
            content = content.as_str()[0..content.len() - 1].to_string();
        }
        //dbg!(&content);
        //dbg!(&cmd_type);
        //dbg!(&user);
        //dbg!(&content);
        Lcommand {
            cmd_type,
            user,
            content,
        }
    }

    fn display(self) -> String {
        let mut output = String::new();
        match self.cmd_type {
            Lcmd::Say => output.push_str(format!("<{}>: {}", self.user, self.content).as_str()),
            Lcmd::Whisper => output.push_str(format!("({}): {}", self.user, self.content).as_str()),
            Lcmd::Conn => output.push_str(format!("[SERVER]: {} joined", self.user).as_str()),
            Lcmd::Dc => output.push_str(format!("[SERVER]: {} left", self.user).as_str()),
        }
        output
    }

    fn from(buf: String) -> Lcommand {
        let cmd_split: Vec<&str> = buf.split('\n').collect();
        //dbg!(cmd_split.get(0));
        //dbg!(cmd_split.get(1));
        //dbg!(cmd_split.get(2));
        let cmd_type = match cmd_split[0] {
            "SAY" => Lcmd::Say,
            "CONNECT" => Lcmd::Conn,
            "DISCONNECT" => Lcmd::Dc,
            "WHISPER" => Lcmd::Whisper,
            _ => panic!("fucky wucky happened"),
        };
        let user = String::from(cmd_split[1]);
        let content = match cmd_type {
            Lcmd::Say => String::from(cmd_split[2]),
            Lcmd::Whisper => String::from(cmd_split[2]),
            _ => String::new(),
        };

        Lcommand {
            cmd_type,
            user,
            content,
        }
    }
}

struct Client {
    username: String,
    tx: Option<mpsc::Sender<Lcommand>>, // Channel to send messages to connected server
    rx: Option<mpsc::Receiver<Lcommand>>, // Channel to receive messages from connected server
    messages: Vec<String>,
}

impl Client {
    fn new(user: String) -> Client {
        Client {
            username: user,
            tx: None,
            rx: None,
            messages: vec![],
        }
    }

    fn send_msg(&mut self, msg: Lcommand) {
        let mut msg = msg;
        msg.user = self.username.clone();
        self.tx.as_ref().unwrap().send(msg).unwrap();
    }

    fn connect(&mut self, addr: String) {
        let tx: mpsc::Sender<Lcommand>;
        let out_rx: mpsc::Receiver<Lcommand>;
        let channel = mpsc::channel();
        tx = channel.0;
        out_rx = channel.1;
        let mut out_stream = TcpStream::connect(addr).unwrap();
        let mut rec_stream = out_stream.try_clone().unwrap();

        // Output thread
        std::thread::spawn(move || {
            loop {
                let mut end = false;
                let msg = out_rx.recv().unwrap();
                let mut out_buf = String::new();
                match msg.cmd_type {
                    Lcmd::Conn => out_buf.push_str("CONNECT\n"),
                    Lcmd::Dc => {
                        out_buf.push_str("DISCONNECT\n");
                        end = true // Stop handling the stream when Dc is passed
                    }
                    Lcmd::Say => out_buf.push_str("SAY\n"),
                    Lcmd::Whisper => out_buf.push_str("WHISPER\n"),
                }
                out_buf.push_str(&msg.user);
                out_buf.push('\n');
                out_buf.push_str(&msg.content);
                let _n = out_stream.write(out_buf.as_bytes()).unwrap();
                if end {
                    break;
                }
            }
            out_stream.shutdown(std::net::Shutdown::Both).unwrap();
        });

        let rx: mpsc::Receiver<Lcommand>;
        let in_tx: mpsc::Sender<Lcommand>;
        let channel = mpsc::channel();
        in_tx = channel.0;
        rx = channel.1;
        // Receiver thread
        std::thread::spawn(move || {
            let mut msgbuf: Vec<u8> = vec![0; 1024];
            in_tx.send(Lcommand::new(String::from("/connect"), String::from("you"))).unwrap();
            loop {
                let _n = rec_stream.read(&mut msgbuf).unwrap();
                in_tx.send(Lcommand::from(String::from_utf8(msgbuf.clone()).unwrap())).unwrap();
            }
        });
        self.tx = Some(tx);
        self.rx = Some(rx)
    }

    fn display_messages(&self, mut out: &std::io::Stdout) {
        if !self.messages.is_empty() {
            let mut msg_iter = self.messages.clone();
            msg_iter.reverse();
            let mut msg_iter = msg_iter.into_iter();

            // Print messages
            queue!(out, cursor::MoveToRow(terminal::size().unwrap().1 - 1)).unwrap();
            for _ in 0..terminal::size().unwrap().1 {
                let msg = msg_iter.next();
                if msg.is_some() {
                    queue!(out, cursor::MoveToPreviousLine(1), Print(msg.unwrap())).unwrap();
                } else {
                    break;
                }
            }
        }
    }
}

fn main() {
    let mut client = Client::new(String::from("test_user"));
    client.connect(String::from("127.0.0.1:6969"));
    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    crossterm::queue!(stdout, crossterm::terminal::Clear(ClearType::All)).unwrap();

    loop {
        // Get new received messages
        if client.rx.is_some() {
            let new_msg = client.rx.as_ref().unwrap().try_recv();
            if let Ok(new_msg) = new_msg {
                client.messages.push(new_msg.display());
            }
        }

        clear_screen(&stdout);

        client.display_messages(&stdout);
        
        queue!(
            stdout,
            cursor::MoveTo(0, terminal::size().unwrap().1),
            Print("[MSG]: "),
        )
        .unwrap();
        
        stdout.flush().unwrap();
        std::thread::sleep(Duration::from_millis(1000));
    }
}

fn clear_screen(mut out: &std::io::Stdout) {
    // Clear screen
    queue!(
        out,
        cursor::MoveTo(0, 0),
        cursor::Hide,
        Clear(ClearType::All)
    )
    .unwrap();
}
