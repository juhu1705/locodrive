use std::{env, process};
use serialport::FlowControl;
use locodrive::args::{SlotArg, SpeedArg};

use locodrive::error::MessageParseError;
use locodrive::loco_controller::LocoNetConnector;
use locodrive::protocol::Message;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <serial port>", args[0]);
        process::exit(1);
    }

    let arg = env::args_os().nth(1).unwrap();

    // let mut loco_controller = LocoNetConnector::new(arg.to_str().unwrap(), 115_200, 500, 5000, FlowControl::Software).unwrap();

    // loco_controller.start_reader();
    // loco_controller.write(Message::LocoSpd(SlotArg::new(3), SpeedArg::new(120)));

    loop {
        print!("Read: ");
        /*match Message::parse(&mut stream) {
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
        }*/
    }
}
