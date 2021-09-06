use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

use email::MailFSM;

pub mod email;

pub fn handle_connection(stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut mail_fsm = MailFSM::new(String::from("my.server"));

    loop {
        let mut buf = String::new();
        let data_size = reader.read_line(&mut buf).expect("Unable to read line");

        if data_size == 0 {
            break;
        };

        if let Some(msg) = mail_fsm.process_line(&buf) {
            writer
                .write(&msg.as_bytes()[..])
                .expect("Unable to write to stream");
            print!("{}", mail_fsm.mail);
            writer.flush().unwrap();
        }
        if mail_fsm.is_finished() {
            break;
        }
    }
}
