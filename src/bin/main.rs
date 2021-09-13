extern crate serial;

use std::io::prelude::*;
use std::io::{BufReader, Bytes};
use std::time::Duration;
use std::{env, process};

use locodrive::protocol::Message;
use serial::prelude::*;

fn main() {
    if env::args_os().len() != 2 {
        eprintln!("Error: Invalid argument count");
        process::exit(1);
    }

    let arg = env::args_os().nth(1).unwrap();
    let mut port = match serial::open(&arg) {
        Ok(port) => port,
        Err(err) => {
            eprintln!("Unable to open device: {}", err);
            process::exit(2);
        }
    };

    let reconfig_result = port
        .reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud115200).unwrap();
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop2);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })
        .and_then(|()| port.set_timeout(Duration::from_secs(10000)));

    if let Err(err) = reconfig_result {
        eprintln!("Error while reconfiguring serial port: {}", err);
        process::exit(2);
    }

    let mut stream = BufReader::new(port).bytes();
    let mut msg = Vec::new();
    msg.push(next_byte(&mut stream));
    loop {
        let byte = next_byte(&mut stream);
        if byte & 0x80 != 0 {
            parse(&msg);
            msg.clear();
        }
        msg.push(byte);
    }
}

fn next_byte<R: Read>(bytes: &mut Bytes<R>) -> u8 {
    match bytes.next() {
        Some(byte_result) => match byte_result {
            Ok(byte) => byte,
            Err(err) => {
                eprintln!("Error while reading from serial port: {}", err);
                process::exit(2);
            }
        },
        None => {
            eprintln!("Error while reading from serial port: next() returned None");
            process::exit(2);
        }
    }
}

fn parse(msg: &[u8]) {
    match Message::parse(msg) {
        Ok(message) => {
            println!("Received: {:02x?} | {:?}", msg, message);
        }
        Err(err) => {
            println!("Received: {:02x?} | PARSE ERROR: {}", msg, err);
        }
    }
}
