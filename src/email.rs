use std::fmt::Display;

#[derive(PartialEq)]
enum State {
    New,
    Hello,
    MailFrom,
    RcptTo,
    Data,
    Quit,
}

pub struct Mail {
    pub helo: Option<String>,
    pub mail_from: Option<String>,
    pub rcpt_to: Vec<String>,
    pub data: Option<String>,
}

impl Mail {
    pub fn new() -> Mail {
        Mail {
            helo: None,
            mail_from: None,
            rcpt_to: Vec::new(),
            data: None,
        }
    }

    fn add_hello(&mut self, hello: &str) {
        self.helo = Some(String::from(hello.trim()));
    }

    fn add_mail_from(&mut self, mail_from: &str) {
        self.mail_from = Some(String::from(mail_from.trim()));
    }

    fn add_rcpt_to(&mut self, rcpt_to: &str) {
        self.rcpt_to.push(String::from(rcpt_to.trim()));
    }

    fn add_data_chunk(&mut self, data_chunk: &str) {
        if let None = self.data {
            self.data = Some(String::from(data_chunk));
        } else {
            self.data.as_mut().map(|s| s.push_str(data_chunk));
        }
    }
}

impl Display for Mail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        if let Some(helo) = &self.helo {
            output.push_str(&format!("HELO {}\n", helo));
        }

        if let Some(mail_from) = &self.mail_from {
            output.push_str(&format!("MAIL FROM: {}\n", mail_from));
        }

        for rcpt_to in self.rcpt_to.iter() {
            output.push_str(&format!("RCPT TO: {}\n", rcpt_to));
        }

        if let Some(data) = &self.data {
            output.push_str(&format!("DATA \n{}\n", data));
        }

        write!(f, "{}\n", output)
    }
}

pub struct MailFSM {
    current_state: State,
    server_name: String,
    pub mail: Mail,
}

const HELO: &str = "HELO";
const EHLO: &str = "EHLO";
const MAIL_FROM: &str = "MAIL FROM:";
const RCPT_TO: &str = "RCPT TO:";
const DATA: &str = "DATA";
const QUIT: &str = "QUIT";
const DOT: &str = ".";

impl MailFSM {
    pub fn new(server_name: String) -> MailFSM {
        MailFSM {
            current_state: State::New,
            server_name,
            mail: Mail::new(),
        }
    }

    pub fn process_line(&mut self, line: &str) -> Option<String> {
        let curated_line = line.trim().to_uppercase();
        match &self.current_state {
            State::New if curated_line.starts_with(HELO) || curated_line.starts_with(EHLO) => {
                self.mail.add_hello(&line.trim()[HELO.len()..]);
                self.current_state = State::Hello;
                Some(format!("250 {}\n", self.server_name))
            }
            State::Hello if curated_line.starts_with(MAIL_FROM) => {
                self.mail.add_mail_from(&line.trim()[MAIL_FROM.len()..]);
                self.current_state = State::MailFrom;
                Some(String::from("250 Ok\n"))
            }
            State::MailFrom if curated_line.starts_with(RCPT_TO) => {
                self.mail.add_rcpt_to(&line.trim()[RCPT_TO.len()..]);
                self.current_state = State::RcptTo;
                Some(String::from("250 Ok\n"))
            }
            State::RcptTo if curated_line.starts_with(RCPT_TO) => {
                self.mail.add_rcpt_to(&line.trim()[RCPT_TO.len()..]);
                Some(String::from("250 Ok\n"))
            }
            State::RcptTo if curated_line.starts_with(DATA) => {
                self.mail.add_rcpt_to(&line.trim()[DATA.len()..]);
                self.current_state = State::Data;
                Some(String::from("354 End data with <CR><LF>.<CR><LF>\n"))
            }
            State::Data if line.trim() == DOT => Some(format!(
                "250 Ok: queued as {}\n",
                self.mail.data.as_ref().unwrap_or(&String::from("")).len()
            )),
            State::Data if curated_line.starts_with(QUIT) => {
                self.current_state = State::Quit;
                Some(String::from("221 Bye\n"))
            }
            State::Data => {
                self.mail.add_data_chunk(line);
                None
            }
            _ => Some(String::from("Unknown command")),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current_state == State::Quit
    }

    pub fn greeting(&self) -> String {
        String::from(format!("220 {} simple-smtp\n", self.server_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail() {
        let mut mail = Mail::new();
        mail.add_hello("server");
        assert_eq!(mail.helo, Some(String::from("server")));
        mail.add_mail_from("some@email");
        assert_eq!(mail.mail_from, Some(String::from("some@email")));
        mail.add_rcpt_to("email@some");
        mail.add_rcpt_to("email1@some");
        assert_eq!(mail.rcpt_to, vec!["email@some", "email1@some"]);
        mail.add_data_chunk("abc");
        mail.add_data_chunk("def");
        assert_eq!(mail.data, Some(String::from("abcdef")));
    }

    #[test]
    fn test_mail_fsm() {
        let mut mail_fsm = MailFSM::new(String::from("test.server"));
        assert_eq!(
            mail_fsm.process_line("HELO server\n"),
            Some(String::from("250 test.server\n"))
        );
        assert_eq!(
            mail_fsm.process_line("MAIL FROM: sender@email\n"),
            Some(String::from("250 Ok\n"))
        );
        assert_eq!(
            mail_fsm.process_line("RCPT TO: rcpt1@email\n"),
            Some(String::from("250 Ok\n"))
        );
        assert_eq!(
            mail_fsm.process_line("RCPT TO: rcpt2@email\n"),
            Some(String::from("250 Ok\n"))
        );
        assert_eq!(
            mail_fsm.process_line("DATA\n"),
            Some(String::from("354 End data with <CR><LF>.<CR><LF>\n"))
        );
        assert_eq!(mail_fsm.process_line("qwert\n"), None);
        assert_eq!(
            mail_fsm.process_line(".\n"),
            Some(String::from("250 Ok: queued as 6\n"))
        );
        assert_eq!(
            mail_fsm.process_line("QUIT\n"),
            Some(String::from("221 Bye\n"))
        );
        assert!(mail_fsm.is_finished())
    }
}
