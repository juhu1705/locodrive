/// Tests all testable core functions of this module
#[cfg(test)]
mod tests {
    use crate::args::{Ack1Arg, AddressArg, Consist, CvDataArg, DecoderType, DirfArg, DstArg, FastClock, FunctionArg, IdArg, ImAddress, ImArg, ImFunctionType, InArg, LissyIrReport, LopcArg, MultiSenseArg, Pcmd, ProgrammingAbortedArg, PxctData, RepStructure, RFID5Report, RFID7Report, SensorLevel, SlotArg, SnArg, SndArg, SourceType, SpeedArg, Stat1Arg, Stat2Arg, State, SwitchArg, SwitchDirection, TrkArg, WheelcntReport, WrSlDataGeneral, WrSlDataPt, WrSlDataStructure, WrSlDataTime};
    use crate::protocol::Message;

    /// Tests if the message parsing is reliable
    #[test]
    fn messages() {
        // Test 2 byte messages
        test_one_message(Message::Idle);
        test_one_message(Message::GpOn);
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
        test_one_message(Message::LocoSpd(SlotArg::new(10), SpeedArg::Drive(122)));

        // Test 6 bytes messages
        test_one_message(Message::MultiSense(MultiSenseArg::new(3, false, 3, 7), AddressArg::new(12)));
        test_one_message(Message::UhliFun(SlotArg::new(128), FunctionArg::new(2)));

        // Test messages of variable byte length
        test_one_message(Message::WrSlData(
            WrSlDataStructure::DataGeneral(
                WrSlDataGeneral::new(
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
            )
        ));
        test_one_message(Message::WrSlData(
            WrSlDataStructure::DataPt(
                WrSlDataPt::new(
                    Pcmd::new(false, true, false, false, false),
                    AddressArg::new(64),
                    TrkArg::new(true, true, true, true),
                    *CvDataArg::new().set_data(1, true).set_cv(2, true),
                )
            )
        ));
        test_one_message(Message::WrSlData(
            WrSlDataStructure::DataTime(
                WrSlDataTime::new(
                    FastClock::new(12, 23, 2, 12, 22, 12, 17),
                    TrkArg::new(false, true, true, true),
                    IdArg::new(123),
                )
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
}
