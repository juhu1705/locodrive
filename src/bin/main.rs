extern crate serial;

use std::io::prelude::*;
use std::io::BufReader;
use std::time::Duration;
use std::{env, process};

use locodrive::error::MessageParseError;
use locodrive::protocol::Message;
use serial::prelude::*;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <serial port>", args[0]);
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

    // set up the stream iterator
    let mut stream = BufReader::new(&mut port).bytes().map(|r| match r {
        Ok(byte) => {
            // upon yielding a byte, print it
            print!("{:02x} ", byte);
            byte
        }
        Err(err) => {
            eprintln!("Error while reading from serial port: {}", err);
            process::exit(2);
        }
    });

    loop {
        print!("Read: ");
        match Message::parse(&mut stream) {
            Ok(msg) => {
                print!("=> {:?} ==>", msg);
                for byte in msg.to_message() {
                    print!(" {:02x} ", byte);
                }
                println!()
            },
            Err(err) => {
                println!("=> ERROR: {}", err);
                if let MessageParseError::UnexpectedEnd = err {
                    process::exit(2);
                }
            }
        }
    }
}
