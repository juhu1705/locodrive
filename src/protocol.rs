use crate::args::*;
use crate::error::MessageParseError;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Message {
    Idle,
    GpOn,
    GpOff,
    Busy,

    LocoAdr(AddressArg),
    SwAck(SwitchArg),
    SwState(SwitchArg),
    RqSlData(SlotArg),
    MoveSlots(SlotArg, SlotArg),
    LinkSlots(SlotArg, SlotArg),
    UnlinkSlots(SlotArg, SlotArg),
    ConsistFunc(SlotArg, DirfArg),
    SlotStat1(SlotArg, Stat1Arg),
    LongAck(LopcArg, Ack1Arg),
    InputRep(InArg),
    SwRep(SnArg),
    SwReq(SwitchArg),
    LocoSnd(SlotArg, SndArg),
    LocoDirf(SlotArg, DirfArg),
    LocoSpd(SlotArg, SpeedArg),
    MultiSense(MultiSenseArg, AddressArg),
    UhliFun(SlotArg, FunctionArg),
    WrSlData(WrSlDataStructure),
    SlRdData(
        SlotArg,
        Stat1Arg,
        AddressArg,
        SpeedArg,
        DirfArg,
        TrkArg,
        Stat2Arg,
        SndArg,
        IdArg,
    ),
    ImmPacket(ImArg),
    Rep(RepStructure),
    PeerXfer(SlotArg, DstArg, PxctData),
}

impl Message {
    /// Reads and Parses the next message from `stream`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the message could not be parsed:
    ///
    /// - [`UnknownOpcode`] if the message has an unknown opcode
    /// - [`UnexpectedEnd`] if `stream` unexpectedly yields [`None`]
    /// - [`InvalidChecksum`] if the checksum is invalid
    ///
    /// [`UnknownOpcode`]: MessageParseError::UnknownOpcode
    /// [`UnexpectedEnd`]: MessageParseError::UnexpectedEnd
    /// [`InvalidChecksum`]: MessageParseError::InvalidChecksum
    pub fn parse(buf: &[u8], opc: u8, len: usize) -> Result<Self, MessageParseError> {
        // validate checksum
        if !Self::validate(&buf[0..len]) {
            return Err(MessageParseError::InvalidChecksum);
        }

        // call appropriate parse function
        match len {
            2 => Self::parse2(opc),
            4 => Self::parse4(opc, &buf[1..3]),
            6 => Self::parse6(opc, &buf[1..5]),
            var => Self::parse_var(opc, &buf[1..var - 1]),
        }
    }

    fn parse2(opc: u8) -> Result<Self, MessageParseError> {
        match opc {
            0x85 => Ok(Self::Idle),
            0x83 => Ok(Self::GpOn),
            0x82 => Ok(Self::GpOff),
            0x81 => Ok(Self::Busy),
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    fn parse4(opc: u8, args: &[u8]) -> Result<Self, MessageParseError> {
        if args.len() != 2 {
            return Err(MessageParseError::UnexpectedEnd)
        }
        match opc {
            0xBF => Ok(Self::LocoAdr(AddressArg::parse(args[0], args[1]))),
            0xBD => Ok(Self::SwAck(SwitchArg::parse(args[0], args[1]))),
            0xBC => Ok(Self::SwState(SwitchArg::parse(args[0], args[1]))),
            0xBB => Ok(Self::RqSlData(SlotArg::parse(args[0]))),
            0xBA => Ok(Self::MoveSlots(
                SlotArg::parse(args[0]),
                SlotArg::parse(args[1]),
            )),
            0xB9 => Ok(Self::LinkSlots(
                SlotArg::parse(args[0]),
                SlotArg::parse(args[1]),
            )),
            0xB8 => Ok(Self::UnlinkSlots(
                SlotArg::parse(args[0]),
                SlotArg::parse(args[1]),
            )),
            0xB6 => Ok(Self::ConsistFunc(
                SlotArg::parse(args[0]),
                DirfArg::parse(args[1]),
            )),
            0xB5 => Ok(Self::SlotStat1(
                SlotArg::parse(args[0]),
                Stat1Arg::parse(args[1]),
            )),
            0xB4 => Ok(Self::LongAck(
                LopcArg::parse(args[0]),
                Ack1Arg::parse(args[1]),
            )),
            0xB2 => Ok(Self::InputRep(InArg::parse(args[0], args[1]))),
            0xB1 => Ok(Self::SwRep(SnArg::parse(args[0], args[1]))),
            0xB0 => Ok(Self::SwReq(SwitchArg::parse(args[0], args[1]))),
            0xA2 => Ok(Self::LocoSnd(
                SlotArg::parse(args[0]),
                SndArg::parse(args[1]),
            )),
            0xA1 => Ok(Self::LocoDirf(
                SlotArg::parse(args[0]),
                DirfArg::parse(args[1]),
            )),
            0xA0 => Ok(Self::LocoSpd(
                SlotArg::parse(args[0]),
                SpeedArg::parse(args[1]),
            )),
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    fn parse6(opc: u8, args: &[u8]) -> Result<Self, MessageParseError> {
        if args.len() != 4 {
            return Err(MessageParseError::UnexpectedEnd)
        }
        match opc {
            0xD0 => Ok(Self::MultiSense(
                MultiSenseArg::parse(args[0], args[1]),
                AddressArg::parse(args[2], args[3]),
            )),
            0xD4 => {
                if 0x20 != args[0] {
                    return Err(MessageParseError::InvalidFormat(format!(
                        "Expected first arg of UhliFun to be 0x20 got {:02x}", args[0]
                    ).into()));
                }
                Ok(Self::UhliFun(
                    SlotArg::parse(args[1]),
                    FunctionArg::parse(args[2], args[3]),
                ))
            }
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    fn parse_var(opc: u8, args: &[u8]) -> Result<Self, MessageParseError> {
        if args.len() + 2 != args[0] as usize {
            return Err(MessageParseError::UnexpectedEnd)
        }
        match opc {
            0xE7 => Ok(Self::SlRdData(
                SlotArg::parse(args[1]),
                Stat1Arg::parse(args[2]),
                AddressArg::parse(args[8], args[3]),
                SpeedArg::parse(args[4]),
                DirfArg::parse(args[5]),
                TrkArg::parse(args[6]),
                Stat2Arg::parse(args[7]),
                SndArg::parse(args[9]),
                IdArg::parse(args[10], args[11]),
            )),
            0xED => {
                if args[1] != 0x7F {
                    return Err(
                        MessageParseError::InvalidFormat(
                            format!("The check byte of the received message was invalid. \
                            Expected 0x7F got {:02x}", args[1]).into()
                        )
                    )
                }

                Ok(Self::ImmPacket(ImArg::parse(
                    args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8],
                )))
            },
            0xEF => Ok(Self::WrSlData(WrSlDataStructure::parse(
                args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8], args[9],
                args[10], args[11],
            ))),
            0xE4 => Ok(Self::Rep(
                match RepStructure::parse(args[0], &args[1..]) {
                    Err(err) => return Err(err),
                    Ok(rep) => rep
                }
            )),
            0xE5 => Ok(Self::PeerXfer(
                SlotArg::parse(args[1]),
                DstArg::parse(args[2], args[3]),
                PxctData::parse(
                    args[4], args[5], args[6], args[7], args[8], args[9], args[10], args[11],
                    args[12], args[13],
                ),
            )),
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    fn validate(msg: &[u8]) -> bool {
        return msg.iter().fold(0, |acc, &b| acc ^ b) == 0xFF;
    }

    pub(crate) fn to_message(&self) -> Vec<u8> {
        let mut message = match *self {
            Message::Idle => vec![0x85_u8],
            Message::GpOn => vec![0x83_u8],
            Message::GpOff => vec![0x82_u8],
            Message::Busy => vec![0x81_u8],
            Message::LocoAdr(adr_arg) => vec![0xBF_u8, adr_arg.adr2(), adr_arg.adr1()],
            Message::SwAck(switch_arg) => vec![0xBD_u8, switch_arg.sw1(), switch_arg.sw2()],
            Message::SwState(switch_arg) => vec![0xBC_u8, switch_arg.sw1(), switch_arg.sw2()],
            Message::RqSlData(slot_arg) => vec![0xBB_u8, slot_arg.slot(), 0x00_u8],
            Message::MoveSlots(src, dst) => vec![0xBA_u8, src.slot(), dst.slot()],
            Message::LinkSlots(sl1, sl2) => vec![0xB9_u8, sl1.slot(), sl2.slot()],
            Message::UnlinkSlots(sl1, sl2) => vec![0xB8_u8, sl1.slot(), sl2.slot()],
            Message::ConsistFunc(slot, dirf) => vec![0xB6_u8, slot.slot(), dirf.dirf()],
            Message::SlotStat1(slot, stat1) => vec![0xB5_u8, slot.slot(), stat1.stat1()],
            Message::LongAck(lopc, ack1) => vec![0xB4_u8, lopc.lopc(), ack1.ack1()],
            Message::InputRep(input) => vec![0xB2_u8, input.in1(), input.in2()],
            Message::SwRep(sn_arg) => vec![0xB1_u8, sn_arg.sn1(), sn_arg.sn2()],
            Message::SwReq(sw) => vec![0xB0_u8, sw.sw1(), sw.sw2()],
            Message::LocoSnd(slot, snd) => vec![0xA2_u8, slot.slot(), snd.snd()],
            Message::LocoDirf(slot, dirf) => vec![0xA1_u8, slot.slot(), dirf.dirf()],
            Message::LocoSpd(slot, spd) => vec![0xA0_u8, slot.slot(), spd.spd()],
            Message::MultiSense(multi_sense, address) => vec![
                0xD0_u8,
                multi_sense.m_type(),
                multi_sense.zas(),
                address.adr1(),
                address.adr2(),
            ],
            Message::UhliFun(slot, function) => vec![
                0xD4_u8,
                0x20_u8,
                slot.slot(),
                function.group(),
                function.function(),
            ],
            Message::WrSlData(wr_slot_data_arg) => wr_slot_data_arg.to_message(),
            Message::SlRdData(slot, stat1, adr, spd, dirf, trk, stat2, snd, id) => vec![
                0xE7_u8,
                0x0E_u8,
                slot.slot(),
                stat1.stat1(),
                adr.adr1(),
                spd.spd(),
                dirf.dirf(),
                trk.trk_arg(),
                stat2.stat2(),
                adr.adr2(),
                snd.snd(),
                id.id1(),
                id.id2(),
            ],
            Message::ImmPacket(im) => vec![
                0xED_u8,
                0x0B_u8,
                0x7F_u8,
                im.reps(),
                im.dhi(),
                im.im1(),
                im.im2(),
                im.im3(),
                im.im4(),
                im.im5(),
            ],
            Message::Rep(rep) => match rep {
                RepStructure::RFID7Report(report) => report.to_message(),
                RepStructure::RFID5Report(report) => report.to_message(),
                RepStructure::LissyIrReport(report) => report.to_message(),
                RepStructure::WheelcntReport(report) => report.to_message(),
            },
            Message::PeerXfer(src, dst, pxct) => vec![
                0xE5,
                0x10,
                src.slot(),
                dst.dst_low(),
                dst.dst_high(),
                pxct.pxct1(),
                pxct.d1(),
                pxct.d2(),
                pxct.d3(),
                pxct.d4(),
                pxct.pxct2(),
                pxct.d5(),
                pxct.d6(),
                pxct.d7(),
                pxct.d8(),
            ],
        };

        message.push(Self::check_sum(&message));

        message
    }

    fn check_sum(msg: &[u8]) -> u8 {
        0xFF - msg.iter().fold(0, |acc, &b| acc ^ b)
    }

    pub fn lack_follows(&self) -> bool {
        matches!(
            self,
            Message::LocoAdr(_) |
            Message::SwAck(_) |
            Message::SwState(_) |
            Message::SwReq(_) |
            Message::WrSlData(_) |
            Message::ImmPacket(_)
        )
    }
}
