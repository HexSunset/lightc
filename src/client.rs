use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent},
    queue,
    style::Print,
    terminal,
};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

use super::lcommand::{Lcmd, Lcommand};

pub struct Client {
    pub username: String,
    pub connected: Option<String>,
    pub tx: Option<mpsc::Sender<Lcommand>>, // Channel to send messages to connected server
    pub rx: Option<mpsc::Receiver<Lcommand>>, // Channel to receive messages from connected server
    pub messages: Vec<String>,
    pub user_in: mpsc::Receiver<char>,
}

impl Client {
    pub fn new(user: String) -> Client {
        let user_tx: mpsc::Sender<char>;
        let user_in: mpsc::Receiver<char>;
        let channel = mpsc::channel();
        user_tx = channel.0;
        user_in = channel.1;

        // User input monitor thread
        thread::spawn(move || loop {
            let k = read().unwrap();
            match k {
                // entered random character
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: _m,
                }) => user_tx.send(c).unwrap(),

                // backspace
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: _m,
                }) => user_tx.send(0x8 as char).unwrap(), // Backspace ascii Pog

                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: _m,
                }) => user_tx.send(0xA as char).unwrap(), // Newline ascii Pog

                _ => (), // Ignore other events
            }
        });

        Client {
            username: user,
            connected: None,
            tx: None,
            rx: None,
            messages: vec![],
            user_in,
        }
    }

    // returns true if connected to a server, false if not
    pub fn send_msg(&mut self, msg: Lcommand) -> bool {
        let mut msg = msg;
        if msg.cmd_type == Lcmd::Nick {
        } else {
            msg.user = self.username.clone();
        }
        if msg.cmd_type == Lcmd::Dc {
            self.connected = None;
        }
        //self.tx.as_ref().unwrap().send(msg).unwrap();
        if self.tx.is_some() {
            let tx = self.tx.as_ref().unwrap();
            let ret = tx.send(msg);
            if ret.is_err() {
                return false;
            }
            true
        } else {
            false
        }
    }

    pub fn connect(&mut self, addr: String) {
        let tx: mpsc::Sender<Lcommand>;
        let out_rx: mpsc::Receiver<Lcommand>;
        let channel = mpsc::channel();
        tx = channel.0;
        out_rx = channel.1;
        let out_stream = TcpStream::connect(addr.clone());
        if out_stream.is_err() {
            self.messages
                .push(format!("[CLIENT]: failed to join {}", addr));
            return;
        }
        let mut out_stream = out_stream.unwrap();
        self.messages
            .push(format!("[CLIENT]: you joined {}", &addr));
        self.connected = Some(addr);
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
                    Lcmd::Nick => out_buf.push_str("NICK\n"),
                    Lcmd::Quit => {
                        out_buf.push_str("DISCONNECT\n");
                        end = true // Stop handling the stream when Quit is passed
                    }
                }
                out_buf.push_str(&msg.user);
                out_buf.push('\n');
                out_buf.push_str(&msg.content);
                out_buf.push('\n');
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
            loop {
                let n = rec_stream.read(&mut msgbuf);
                if n.is_err() {
                    break;
                }
                in_tx
                    .send(Lcommand::from(String::from_utf8(msgbuf.clone()).unwrap()))
                    .unwrap();
            }
        });
        self.tx = Some(tx.clone());
        self.rx = Some(rx);
        tx.send(Lcommand {
            cmd_type: Lcmd::Conn,
            user: self.username.clone(),
            content: String::new(),
        })
        .unwrap();
    }

    pub fn display_messages(&self, mut out: &std::io::Stdout) {
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

    pub fn print_prompt(&self, mut out: &std::io::Stdout, text: String) {
        let status: String;
        if self.connected.is_some() {
            status = format!("[{}]", self.connected.as_ref().unwrap());
        } else {
            status = String::from("[]");
        }
        queue!(
            out,
            cursor::MoveTo(0, terminal::size().unwrap().1),
            Print(format!("{}: {}", status, text)),
        )
        .unwrap();
    }

    pub fn print_welcome(&mut self) {
        self.messages.push("Hi! Welcome to LightC! :)".to_string());
        self.messages
            .push("Set your nickname with '/nick ...'".to_string());
        self.messages
            .push("Connect to a server with '/connect ...'".to_string());
    }
}
