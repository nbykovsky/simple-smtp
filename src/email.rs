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
    hello: Option<String>,
    mail_from: Option<String>,
    rcpt_to: Vec<String>,
    data: Option<String>,
}

impl Mail {
    pub fn new() -> Mail {
        Mail {
            hello: None,
            mail_from: None,
            rcpt_to: Vec::new(),
            data: None,
        }
    }

    fn add_hello(&mut self, hello: &str) {
        self.hello = Some(String::from(hello.trim()));
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

        if let Some(helo) = &self.hello {
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
    pub mail: Mail,
}

const HELO: &str = "HELO";
const MAIL_FROM: &str = "MAIL FROM:";
const RCPT_TO: &str = "RCPT TO:";
const DATA: &str = "DATA";
const QUIT: &str = "QUIT";
const DOT: &str = ".";

impl MailFSM {
    pub fn new() -> MailFSM {
        MailFSM {
            current_state: State::New,
            mail: Mail::new(),
        }
    }

    pub fn process_line(&mut self, line: &str) -> Option<String> {
        match &self.current_state {
            State::New if line.trim().starts_with(HELO) => {
                self.mail.add_hello(&line.trim()[HELO.len()..]);
                self.current_state = State::Hello;
                Some(String::from(
                    "250 smtp.example.com, I am glad to meet you\n",
                ))
            }
            State::Hello if line.trim().starts_with(MAIL_FROM) => {
                self.mail.add_mail_from(&line.trim()[MAIL_FROM.len()..]);
                self.current_state = State::MailFrom;
                Some(String::from("250 Ok\n"))
            }
            State::MailFrom if line.trim().starts_with(RCPT_TO) => {
                self.mail.add_rcpt_to(&line.trim()[RCPT_TO.len()..]);
                self.current_state = State::RcptTo;
                Some(String::from("250 Ok\n"))
            }
            State::RcptTo if line.trim().starts_with(RCPT_TO) => {
                self.mail.add_rcpt_to(&line.trim()[RCPT_TO.len()..]);
                Some(String::from("250 Ok\n"))
            }
            State::RcptTo if line.trim().starts_with(DATA) => {
                self.mail.add_rcpt_to(&line.trim()[DATA.len()..]);
                self.current_state = State::Data;
                Some(String::from("354 End data with <CR><LF>.<CR><LF>\n"))
            }
            State::Data if line.trim() == DOT => Some(String::from("250 Ok: queued as 12345\n")),
            State::Data if line.trim().starts_with(QUIT) => {
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
}
