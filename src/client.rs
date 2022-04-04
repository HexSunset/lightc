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

impl Default for Client {
    fn default() -> Client {
        let channel = mpsc::channel();
        let user_tx: mpsc::Sender<char> = channel.0;
        let user_in: mpsc::Receiver<char> = channel.1;

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

        let user = String::from("user");

        Client {
            username: user,
            connected: None,
            tx: None,
            rx: None,
            messages: vec![],
            user_in,
        }
    }
}

impl Client {
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
        let channel = mpsc::channel();
        let tx: mpsc::Sender<Lcommand> = channel.0;
        let out_rx: mpsc::Receiver<Lcommand> = channel.1;
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
                    _ => {
                        break;
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

        let channel = mpsc::channel();
        let in_tx: mpsc::Sender<Lcommand> = channel.0;
        let rx: mpsc::Receiver<Lcommand> = channel.1;
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

    pub fn parse_cmd(&mut self, buf: String) -> Lcommand {
        let cmd_split: Vec<&str> = buf.split(' ').collect();
        //dbg!(cmd_split[0]);
        // handle /help separately
        let cmd_type = match cmd_split[0] {
            "/connect" => Lcmd::Conn,
            "/disconnect" => Lcmd::Dc,
            "/nick" => Lcmd::Nick,
            "/quit" => Lcmd::Quit,
            "/help" => Lcmd::Help,
            _ => Lcmd::Say,
        };
        let content = match cmd_type {
            Lcmd::Say => cmd_split.join(" "),
            _ => cmd_split[1..].join(" "),
        };

        if cmd_type == Lcmd::Nick {
            let old_username = self.username.clone();
            self.username = content.clone();
            self.messages.push(format!(
                "[CLIENT]: you changed your nickname to {}",
                self.username.clone()
            ));
            return Lcommand {
                cmd_type,
                user: old_username,
                content,
            };
        }
        Lcommand {
            cmd_type,
            user: self.username.clone(),
            content,
        }
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
            status = format!("[{}@{}]", &self.username, self.connected.as_ref().unwrap());
        } else {
            status = format!("[{}@none]", &self.username);
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
            .push("Use '/help' to see all available commands".to_string());
    }

    pub fn print_help(&mut self) {
        self.messages.push("Commands: ".to_string());
        self.messages
            .push("\t/connect <addr> : connect to server at <addr>".to_string());
        self.messages
            .push("\t/disconnect : disconnect from connected server".to_string());
        self.messages
            .push("\t/nick <nick> : change your username to <nick>".to_string());
        self.messages.push(
            "\t/quit : disconnect from the connected server and close the client".to_string(),
        );
    }
}
