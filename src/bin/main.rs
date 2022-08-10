use std::{env, process};
use tokio_serial::FlowControl;
use locodrive::args::{SlotArg, SpeedArg};

use locodrive::loco_controller::LocoNetConnector;
use locodrive::protocol::Message;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <serial port>", args[0]);
        process::exit(1);
    }

    let arg = env::args_os().nth(1).unwrap();

    let (tx, _) = tokio::sync::mpsc::channel(8);

    let mut loco_controller = LocoNetConnector::new(arg.to_str().unwrap(), 115_200, 500, 5000, FlowControl::Software, tx).unwrap();

    loco_controller.start_reader().await;

    loco_controller.write(Message::LocoSpd(SlotArg::new(3), SpeedArg::new(120))).await;

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
