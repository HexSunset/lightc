use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, queue};
use std::io::{stdout, Write};
use std::time::Duration;

use lightc::{
    client::Client,
    lcommand::{Lcmd, Lcommand},
};

fn main() {
    run_client();
}

fn run_client() {
    let mut client = Client::default();
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
                cmd = Some(client.parse_cmd(prompt_text.clone()));
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
            } else if command.cmd_type == Lcmd::Help {
                client.print_help();
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

fn clear_screen(mut out: &std::io::Stdout) {
    // Clear screen
    queue!(out, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();
}
