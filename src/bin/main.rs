use locodrive::args::{SlotArg, SpeedArg};
use std::{env, process};
use tokio_serial::FlowControl;

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

    let (tx, mut rx) = tokio::sync::mpsc::channel(8);

    let mut loco_controller = LocoNetConnector::new(
        arg.to_str().unwrap(),
        115_200,
        5000,
        5000,
        FlowControl::Software,
        tx,
    )
    .unwrap();

    loco_controller.start_reader().await;

    println!(
        "{}",
        loco_controller
            .send_message(Message::LocoSpd(SlotArg::new(7), SpeedArg::new(70)))
            .await
    );

    let mut i = 0;

    while let Some(message) = rx.recv().await {
        println!("GOT = {:?} {}", message, i);
        i += 1;
        if i >= 10 {
            loco_controller.stop_reader().await;
            break;
        }
    }

    /*loop {
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
    //}*/
}
