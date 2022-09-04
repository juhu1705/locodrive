/// Tests all testable core functions of this module
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::process::exit;
    use std::time::Duration;
    use tokio::time::sleep;
    use tokio_serial::FlowControl;
    use crate::args::{Ack1Arg, AddressArg, Consist, CvDataArg, DecoderType, DirfArg, DstArg, FastClock, FunctionArg, FunctionGroup, IdArg, ImAddress, ImArg, ImFunctionType, InArg, LissyIrReport, LopcArg, MultiSenseArg, Pcmd, ProgrammingAbortedArg, PStat, PxctData, RepStructure, RFID5Report, RFID7Report, SensorLevel, SlotArg, SnArg, SndArg, SourceType, SpeedArg, Stat1Arg, Stat2Arg, State, SwitchArg, SwitchDirection, TrkArg, WheelcntReport, WrSlDataStructure};
    use crate::loco_controller::{LocoDriveController, LocoDriveMessage};
    use crate::protocol::Message;
    use crate::protocol::Message::{GpOn, LocoSpd};

    /// Tests if the message parsing is reliable
    #[test]
    fn messages() {
        // Test 2 byte messages
        test_one_message(Message::Idle);
        test_one_message(GpOn);
        test_one_message(Message::GpOff);
        test_one_message(Message::Busy);

        // Test 4 byte message
        test_one_message(Message::LocoAdr(AddressArg::new(16383)));
        test_one_message(Message::SwAck(SwitchArg::new(2047, SwitchDirection::Curved, true)));
        test_one_message(Message::SwState(SwitchArg::new(0, SwitchDirection::Straight, false)));
        test_one_message(Message::RqSlData(SlotArg::new(10)));
        test_one_message(Message::MoveSlots(SlotArg::new(10), SlotArg::new(10)));
        test_one_message(Message::LinkSlots(SlotArg::new(10), SlotArg::new(1)));
        test_one_message(Message::UnlinkSlots(SlotArg::new(10), SlotArg::new(1)));
        test_one_message(Message::ConsistFunc(SlotArg::new(10), DirfArg::new(true, false, false, true, false, false)));
        test_one_message(Message::SlotStat1(SlotArg::new(10), Stat1Arg::new(true, Consist::Free, State::Idle, DecoderType::Regular28)));
        test_one_message(Message::LongAck(LopcArg::new(10), Ack1Arg::new(true)));
        test_one_message(Message::InputRep(InArg::new(10, SourceType::Ds54Aux, SensorLevel::Low, true)));
        test_one_message(Message::SwRep(SnArg::SwitchType(10, false, true)));
        test_one_message(Message::SwReq(SwitchArg::new(10, SwitchDirection::Curved, false)));
        test_one_message(Message::LocoSnd(SlotArg::new(24), SndArg::new(false, true, false, true)));
        test_one_message(Message::LocoDirf(SlotArg::new(10), DirfArg::new(false, true, false, true, false, false)));
        test_one_message(LocoSpd(SlotArg::new(10), SpeedArg::Drive(122)));

        // Test 6 bytes messages
        test_one_message(Message::MultiSense(MultiSenseArg::new(3, false, 3, 7), AddressArg::new(12)));
        test_one_message(Message::UhliFun(SlotArg::new(128), FunctionArg::new(FunctionGroup::F13TO19)));

        // Test messages of variable byte length
        test_one_message(Message::WrSlData(
            WrSlDataStructure::DataGeneral(
                SlotArg::new(12),
                Stat1Arg::new(false, Consist::Free, State::InUse, DecoderType::Dcc128),
                Stat2Arg::new(false, true, false),
                AddressArg::new(123),
                SpeedArg::Stop,
                DirfArg::new(false, true, false, false, false, false),
                TrkArg::new(true, false, true, true),
                SndArg::new(false, false, false, false),
                IdArg::new(12),
            )
        ));
        test_one_message(Message::WrSlData(
            WrSlDataStructure::DataPt(
                Pcmd::new(false, true, false, false, false),
                AddressArg::new(64),
                TrkArg::new(true, true, true, true),
                *CvDataArg::new().set_data(1, true).set_cv(2, true),
            )
        ));
        test_one_message(Message::WrSlData(
            WrSlDataStructure::DataTime(
                FastClock::new(12, 23, 2, 12, 22, 0x30),
                TrkArg::new(false, true, true, true),
                IdArg::new(123),
            )
        ));
        test_one_message(Message::SlRdData(
            SlotArg::new(12),
            Stat1Arg::new(true, Consist::LogicalSubMember, State::Common, DecoderType::Dcc128),
            AddressArg::new(3),
            SpeedArg::EmergencyStop,
            DirfArg::new(true, true, true, true, true, true),
            TrkArg::new(true, true, true, true),
            Stat2Arg::new(true, true, true),
            SndArg::new(true, true, true, true),
            IdArg::new(12),
        ));
        test_one_message(Message::ProgrammingFinalResponse(
            SlotArg::new(124),
            Stat1Arg::new(false, Consist::LogicalSubMember, State::Idle, DecoderType::AdrMobile28),
            AddressArg::new(0),
            SpeedArg::Stop,
            DirfArg::new(false, false, false, false, false, false),
            TrkArg::new(false, false, false, false),
            Stat2Arg::new(false, false, false),
            SndArg::new(false, false, false, false),
            IdArg::new(0),
            Pcmd::new(true, true, false, false, true),
            PStat::new(false, false, false, false),
            AddressArg::new(0),
            CvDataArg::new()
        ));
        test_one_message(
            Message::ProgrammingAborted(
                ProgrammingAbortedArg::new(
                    0x15, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21]
                )
            )
        );
        test_one_message(
            Message::ProgrammingAborted(
                ProgrammingAbortedArg::new(
                    0x10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21]
                )
            )
        );

        test_one_message(Message::PeerXfer(
            SlotArg::new(54),
            DstArg::new(123),
            PxctData::new(23, 42, 33, 32, 1, 0, 92, 34, 54)
        ));

        test_one_message(Message::Rep(RepStructure::LissyIrReport(LissyIrReport::new(
            true, 77, 66
        ))));
        test_one_message(Message::Rep(RepStructure::WheelcntReport(WheelcntReport::new(
            23, true, 12,
        ))));
        test_one_message(Message::Rep(RepStructure::RFID5Report(RFID5Report::new(
            12, 3, 4, 5, 6, 7, 23
        ))));
        test_one_message(Message::Rep(RepStructure::RFID7Report(RFID7Report::new(
            12, 3, 4, 5, 6, 7, 23, 23, 4,
        ))));

        test_one_message(Message::ImmPacket(ImArg::new(
            32, ImAddress::Long(44), ImFunctionType::F9to12, 0
        )));
        test_one_message(Message::ImmPacket(ImArg::new(
            32, ImAddress::Short(12), ImFunctionType::F21to28, 0
        )));
    }

    /// Tests if the message is the same when parsing it to a `LocoNet`
    /// message and then back parsing it to a [`Message`].
    ///
    /// More shortly: Tests if the parsing of messages is consistent.
    fn test_one_message(message: Message) {
        assert_eq!(
            Message::parse(
                message.to_message().as_slice()
            ).unwrap(),
            message
        );
    }

    #[tokio::test]
    async fn test_message_sending() {
        println!("Start test!");

        for port in tokio_serial::available_ports().unwrap() {
            println!("Port: {:?}", port)
        }

        let (sender, mut receiver) = tokio::sync::broadcast::channel(10);

        println!("Try to connect to port!");

        let mut loco_controller = match LocoDriveController::new(
            "/dev/ttyUSB0",
            115_200,
            50000,
            FlowControl::None,
            sender,
            false,
        ).await {
            Ok(loco_controller) => loco_controller,
            Err(err) => {
                eprintln!("Error: Could not connect to the serial port! {:?}", err);
                println!();
                return;
            }
        };

        let _m = Message::parse(
            Message::SwReq(SwitchArg::new(15, SwitchDirection::Curved, true)).to_message().as_slice()
        ).unwrap();

        println!("Message: {:?}", Message::SwReq(SwitchArg::new(15, SwitchDirection::Curved, true)).to_message());

        loco_controller.send_message(GpOn).await.unwrap();

        println!("Setup test train");

        let adr = AddressArg::new(5);

        let mut slot_adr_map: HashMap<AddressArg, SlotArg> = HashMap::new();

        match loco_controller.send_message(Message::LocoAdr(adr)).await {
            Ok(()) => {},
            Err(err) => {
                eprintln!("Message was not send! {:?}", err);
                println!();
                exit(1)
            }
        };

        loop {
            match receiver.recv().await {
                Ok(message) =>
                    match message {
                        LocoDriveMessage::Message(message) => {
                            match message {
                                Message::SlRdData(slot, _, address, ..) => {
                                    slot_adr_map.insert(address, slot);
                                    println!("Added {:?}, {:?} to {:?}", address, slot, slot_adr_map);
                                    break;
                                },
                                _ => {}
                            }
                        }
                        LocoDriveMessage::Answer(_, _) => {}
                        LocoDriveMessage::Error(err) => {
                            eprintln!("Message could not be read! {:?}", err);
                            exit(1)
                        }
                        LocoDriveMessage::SerialPortError(err) => {
                            eprintln!("Connection refused! {:?}", err);
                            exit(1)
                        }
                    },
                Err(err) => {
                    println!("WHAT? {:?}", err);
                }
            }
        }

        println!("Known Trains: {:?}", slot_adr_map);

        for i in 1..3 {
            println!("Drive round {}", i);

            if i % 2 == 0 {
                loco_controller.send_message(Message::SwReq(SwitchArg::new(15, SwitchDirection::Straight, true))).await.unwrap();
                loco_controller.send_message(Message::SwReq(SwitchArg::new(18, SwitchDirection::Straight, true))).await.unwrap();
            } else {
                loco_controller.send_message(Message::SwReq(SwitchArg::new(15, SwitchDirection::Curved, true))).await.unwrap();
                loco_controller.send_message(Message::SwReq(SwitchArg::new(18, SwitchDirection::Curved, true))).await.unwrap();
            }

            loco_controller.send_message(LocoSpd(*slot_adr_map.get(&adr).unwrap(), SpeedArg::Drive(100))).await.unwrap();

            let mut waiting = true;

            while let Ok(message) = receiver.recv().await {
                match message {
                    LocoDriveMessage::Message(message) => {
                        match message {
                            Message::InputRep(in_arg) => {
                                if i % 2 == 0 && in_arg.address() == 3 && in_arg.input_source() == SourceType::Switch && in_arg.sensor_level() == SensorLevel::High {
                                    waiting = false;
                                    loco_controller.send_message(LocoSpd(*slot_adr_map.get(&adr).unwrap(), SpeedArg::Drive(50))).await.unwrap();
                                } else if i % 2 == 1 && in_arg.address() == 8 && in_arg.input_source() == SourceType::Switch && in_arg.sensor_level() == SensorLevel::High {
                                    waiting = false;
                                    loco_controller.send_message(LocoSpd(*slot_adr_map.get(&adr).unwrap(), SpeedArg::Drive(50))).await.unwrap();
                                } else if !waiting && in_arg.address() == 8 && in_arg.input_source() == SourceType::Ds54Aux && in_arg.sensor_level() == SensorLevel::Low {
                                    break;
                                } else if !waiting && in_arg.address() == 1 && in_arg.input_source() == SourceType::Ds54Aux && in_arg.sensor_level() == SensorLevel::Low {
                                    break;
                                }
                            },
                            _ => {}
                        }
                    }
                    LocoDriveMessage::Answer(_, _) => {}
                    LocoDriveMessage::Error(err) => {
                        eprintln!("Message could not be read! {:?}", err);
                        exit(1)
                    }
                    LocoDriveMessage::SerialPortError(err) => {
                        eprintln!("Connection refused! {:?}", err);
                        exit(1)
                    }
                }
            }

            loco_controller.send_message(LocoSpd(*slot_adr_map.get(&adr).unwrap(), SpeedArg::Stop)).await.unwrap();

            sleep(Duration::from_secs(6)).await;
        }

        println!("Drive 10 rounds!");

        drop(loco_controller);

        println!("Closed loco net!");
    }
}
