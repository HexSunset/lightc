#[derive(Debug, PartialEq, Clone)]
pub enum Lcmd {
    Conn,
    Dc,
    Say,
    Nick,
    Quit,
}

#[derive(Debug, Clone)]
pub struct Lcommand {
    pub cmd_type: Lcmd,
    pub user: String,
    pub content: String,
}
impl Lcommand {
    pub fn display(self, from_client: bool) -> String {
        let mut output = String::new();
        if !from_client {
            match self.cmd_type {
                Lcmd::Say => output.push_str(format!("<{}>: {}", self.user, self.content).as_str()),
                Lcmd::Conn => output.push_str(format!("[SERVER]: {} joined", self.user).as_str()),
                Lcmd::Dc => output.push_str(format!("[SERVER]: {} left", self.user).as_str()),
                Lcmd::Nick => output.push_str(
                    format!(
                        "[SERVER]: {} changed their nickname to {}",
                        self.user, self.content
                    )
                    .as_str(),
                ),
                _ => (),
            }
            output
        } else {
            match self.cmd_type {
                Lcmd::Say => output.push_str(format!("<{}>: {}", self.user, self.content).as_str()),
                Lcmd::Conn => output.push_str("[CLIENT]: you joined"),
                Lcmd::Dc => output.push_str("[CLIENT]: you left"),
                _ => (),
            }
            output
        }
    }

    pub fn from(buf: String) -> Lcommand {
        let cmd_split: Vec<&str> = buf.split('\n').collect();
        //dbg!(cmd_split.get(0));
        //dbg!(cmd_split.get(1));
        //dbg!(cmd_split.get(2));
        let cmd_type = match cmd_split[0] {
            "SAY" => Lcmd::Say,
            "CONNECT" => Lcmd::Conn,
            "DISCONNECT" => Lcmd::Dc,
            _ => panic!("should not be reachable"),
        };
        let user = String::from(cmd_split[1]);
        let content = match cmd_type {
            Lcmd::Say => String::from(cmd_split[2]),
            _ => String::new(),
        };

        Lcommand {
            cmd_type,
            user,
            content,
        }
    }
}


