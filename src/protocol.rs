use crate::args::*;
use crate::error::MessageParseError;

/// Represents the types of messages that are specified by the `LocoNet` protocol.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Message {
    /// Forces the `LocoNet` to switch in Idle state. An emergency stop for all trains is broadcast.
    /// Note: The `LocoNet` may not response any more.
    Idle,
    /// Turns the global power on. Activates the railway.
    GpOn,
    /// Turns the global power off. Deactivates the railway.
    GpOff,
    /// This is the master `Busy` code. Receiving this indicates that
    /// the master needs time to fulfill a send request.
    Busy,

    /// Requests a loco address to be put to a free slot by the master.
    ///
    /// Using this slot the train is controllable.
    ///
    /// # Success
    ///
    /// [`Message::SlRdData`] containing all slot and address information.
    ///
    /// # Fail
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::failed()`]
    /// Meaning no free slots are available.
    LocoAdr(AddressArg),
    /// Request state of switch with acknowledgment function
    ///
    /// # Success
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::success()`]
    ///
    /// # Fail
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::failed()`]
    /// Meaning switch state not known / Switch not known.
    SwAck(SwitchArg),
    /// Request state of switch
    ///
    /// # Success
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::success()`]
    ///
    /// # Fail
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::failed()`]
    /// Meaning switch state not known / Switch not known.
    SwState(SwitchArg),
    /// Request slot data or status block
    ///
    /// # Success
    ///
    /// [`Message::SlRdData`] containing all slot information.
    RqSlData(SlotArg),
    /// Moves all slot information from a `source` to a `destination` slot address.
    ///
    /// # Special operations
    ///
    /// ## `NULL`-Move
    ///
    /// A `NULL`-Move (Move with equal source and destination) can be used to mark the slot as [`State::InUse`].
    ///
    /// ## Dispatch put
    ///
    /// Moving a slot **to** the *slot 0* marks it as `DISPATCH`-Slot.
    /// In this case the destination slot is not copied to,
    /// but marked by the system as DISPATCH slot.
    ///
    /// ## Dispatch get
    ///
    /// Moving a slot **from** *slot 0* with no destination given (not needed) will
    /// response with a [`Message::SlRdData`] if a as dispatch marked slot is saved in *slot 0*.
    /// Otherwise a [`Message::LongAck`] with [`Ack1Arg::failed()`] indicates the
    /// failure of the operation.
    ///
    /// # Success
    ///
    /// [`Message::SlRdData`] containing all slot information.
    ///
    /// # Fail
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::failed()`]
    /// Meaning slot data could not be moved.
    MoveSlots(SlotArg, SlotArg),
    /// Links the first given slot to the second one
    ///
    /// # Success
    ///
    /// [`Message::SlRdData`] containing slot information.
    ///
    /// # Fail
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::failed()`]
    /// Meaning slot data could not be linked.
    LinkSlots(SlotArg, SlotArg),
    /// Unlinks the first given slot from the second one
    ///
    /// # Success
    ///
    /// [`Message::SlRdData`] containing slot information.
    ///
    /// # Fail
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::failed()`]
    /// Meaning slot data could not be linked.
    UnlinkSlots(SlotArg, SlotArg),
    /// Sets function bits in a [`Consist::LogicalMid`] or
    /// [`Consist::LogicalSubMember`] linked slot.
    ConsistFunc(SlotArg, DirfArg),
    /// Writes a slots stat1.
    /// See [`Stat1Arg`] for information on what you can configure here.
    SlotStat1(SlotArg, Stat1Arg),
    /// This is a long acknowledgment message mostly used to indicate that some requested
    /// operation has failed or succeed.
    LongAck(LopcArg, Ack1Arg),
    /// This holds general sensor input from an addressed sensor.
    ///
    /// On state change this message is automatically send from the sensor over the `LocoNet`,
    /// but if you want to receive all your sensor states you can configure a switch address that
    /// forces the sensor module to send its state in the `LocoNet` sensor device.
    InputRep(InArg),
    /// Switch sensor report
    ///
    /// Reports the switches type and meta information
    SwRep(SnArg),
    /// Requests a switch function. More precisely requests a switch to switch to a
    /// specific direction and activation.
    ///
    /// # Fail
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::failed()`]
    /// Meaning the requested action could not be performed.
    SwReq(SwitchArg),
    /// Sets a slots sound function bits. (functions 5 to 8)
    LocoSnd(SlotArg, SndArg),
    /// Sets a slots direction and first four function bits.
    LocoDirf(SlotArg, DirfArg),
    /// Sets a slot speed.
    LocoSpd(SlotArg, SpeedArg),

    /// Used for power management and transponding
    MultiSense(MultiSenseArg, AddressArg),
    /// In systems from `Uhlenbrock` this message could be used to
    /// access the slot functions 9 to 28.
    UhliFun(SlotArg, FunctionArg),

    /// Used to write special and more complex slot data.
    ///
    /// # Success
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::success()`]
    ///
    /// # Fail
    ///
    /// [`Message::LongAck`] with [`Ack1Arg::failed()`]
    WrSlData(WrSlDataStructure),
    /// This is a slot data response holding all information on the slot.
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
    /// Indicates that the programming service mode is aborted.
    ProgrammingAborted(ProgrammingAbortedArg),

    /// Moves 8 bytes peer to peer from the source slot to the destination.
    ///
    /// Slot = 0: Slot is master
    /// Slot = 0x70 - 0x7E: reserved
    /// Slot = 0x7F: Throttle message xfer
    ///
    PeerXfer(SlotArg, DstArg, PxctData),

    /// This message holds reports
    /// (I am not really sure what this reports represent
    /// and what they are used for.
    /// If you understand them better,
    /// you may help me to improve this documentation and
    /// the implementation of reading and writing this messages.)
    Rep(RepStructure),

    /// Sends n-byte packet immediate
    ///
    /// # Response
    ///
    /// - [`Message::LongAck`] with [`Ack1Arg::success()`]: Not limited
    /// - [`Message::LongAck`] with [`Ack1Arg::limited_success()`]:
    ///   limited with [`Ack1Arg::ack1()`] as limit
    /// - [`Message::LongAck`] with [`Ack1Arg::failed()`]: Busy
    ImmPacket(ImArg),
}

impl Message {
    /// Parses a `LocoNet` message from `buf`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the message could not be parsed:
    ///
    /// - [`UnknownOpcode`]: If the message has an unknown opcode
    /// - [`UnexpectedEnd`]: If the buf not holds the complete message
    /// - [`InvalidChecksum`]: If the checksum is invalid
    /// - [`InvalidFormat`]: If the message is in invalid format
    ///
    /// [`UnknownOpcode`]: MessageParseError::UnknownOpcode
    /// [`UnexpectedEnd`]: MessageParseError::UnexpectedEnd
    /// [`InvalidChecksum`]: MessageParseError::InvalidChecksum
    /// [`InvalidFormat`]: MessageParseError::InvalidFormat
    pub fn parse(buf: &[u8]) -> Result<Self, MessageParseError> {
        let opc = buf[0];
        // We calculate the length of the remaining message to read
        let len = match opc & 0xE0 {
            0x80 => 2,
            0xA0 => 4,
            0xC0 => 6,
            0xE0 => buf[1] as usize,
            _ => return Err(MessageParseError::UnknownOpcode(opc)),
        };

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

    /// Parse all messages of two bytes length. As the second byte is every time the checksum,
    /// only the `opc` is needed for parsing.
    ///
    /// # Errors
    ///
    /// - [`UnknownOpcode`]: If the message has an unknown opcode
    ///
    /// [`UnknownOpcode`]: MessageParseError::UnknownOpcode
    fn parse2(opc: u8) -> Result<Self, MessageParseError> {
        match opc {
            0x85 => Ok(Self::Idle),
            0x83 => Ok(Self::GpOn),
            0x82 => Ok(Self::GpOff),
            0x81 => Ok(Self::Busy),
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    /// Parse all messages of four bytes length.
    /// Therefore the first byte specifying the message type is passed as `opc` and the
    /// other two message bytes are passed as `args`.
    ///
    /// # Errors
    ///
    /// - [`UnknownOpcode`]: If the message has an unknown opcode
    /// - [`UnexpectedEnd`]: If the buf not holds the complete message
    ///
    /// [`UnknownOpcode`]: MessageParseError::UnknownOpcode
    /// [`UnexpectedEnd`]: MessageParseError::UnexpectedEnd
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

    /// Parse all messages of six bytes length.
    /// Therefore the first byte specifying the message type is passed as `opc` and the
    /// other four message bytes are passed as `args`.
    ///
    /// # Errors
    ///
    /// - [`UnknownOpcode`]: If the message has an unknown opcode
    /// - [`UnexpectedEnd`]: If the buf not holds the complete message
    /// - [`InvalidFormat`]: If the message is in invalid format
    ///
    /// [`UnknownOpcode`]: MessageParseError::UnknownOpcode
    /// [`UnexpectedEnd`]: MessageParseError::UnexpectedEnd
    /// [`InvalidFormat`]: MessageParseError::InvalidFormat
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
                    )));
                }
                Ok(Self::UhliFun(
                    SlotArg::parse(args[1]),
                    FunctionArg::parse(args[2], args[3]),
                ))
            }
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    /// Parse all messages of variable length.
    /// Therefore the first byte specifying the message type is passed as `opc` and the
    /// other message bytes are passed as `args`.
    ///
    /// # Errors
    ///
    /// - [`UnknownOpcode`]: If the message has an unknown opcode
    /// - [`UnexpectedEnd`]: If the buf not holds the complete message
    /// - [`InvalidFormat`]: If the message is in invalid format
    ///
    /// [`UnknownOpcode`]: MessageParseError::UnknownOpcode
    /// [`UnexpectedEnd`]: MessageParseError::UnexpectedEnd
    /// [`InvalidFormat`]: MessageParseError::InvalidFormat
    fn parse_var(opc: u8, args: &[u8]) -> Result<Self, MessageParseError> {
        if args.len() + 2 != args[0] as usize {
            return Err(MessageParseError::UnexpectedEnd)
        }
        match opc {
            0xED => {
                if args[1] != 0x7F {
                    return Err(
                        MessageParseError::InvalidFormat(
                            format!("The check byte of the received message was invalid. \
                            Expected 0x7F got {:02x}", args[1])
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
            0xE6 => {
                Ok(Message::ProgrammingAborted(ProgrammingAbortedArg::parse(args[0], &args[1..])))
            },
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

    /// Validates the `msg` by xor-ing all bytes and checking for the result to be 0xFF.
    fn validate(msg: &[u8]) -> bool {
        return msg.iter().fold(0, |acc, &b| acc ^ b) == 0xFF;
    }

    /// Parses the given [`Message`] to a [`Vec<u8>`] using the `LocoNet` protocol.
    pub fn to_message(self) -> Vec<u8> {
        // Parses the message
        let mut message = match self {
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
                multi_sense.m_high(),
                multi_sense.zas(),
                address.adr2(),
                address.adr1(),
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
            Message::ProgrammingAborted(args) => args.to_message(),
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

        // Appending checksum to the created message
        message.push(Self::check_sum(&message));

        message
    }

    /// Calculates the check sum for the given `msg`.
    fn check_sum(msg: &[u8]) -> u8 {
        0xFF - msg.iter().fold(0, |acc, &b| acc ^ b)
    }

    /// # Returns
    ///
    /// The op code for the specified message
    pub fn opc(&self) -> u8 {
        match *self {
            Message::Idle => 0x85,
            Message::GpOn => 0x83,
            Message::GpOff => 0x82,
            Message::Busy => 0x81,
            Message::LocoAdr(..) => 0xBF,
            Message::SwAck(..) => 0xBD,
            Message::SwState(..) => 0xBC,
            Message::RqSlData(..) => 0xBB,
            Message::MoveSlots(..) => 0xBA,
            Message::LinkSlots(..) => 0xB9,
            Message::UnlinkSlots(..) => 0xB8,
            Message::ConsistFunc(..) => 0xB6,
            Message::SlotStat1(..) => 0xB5,
            Message::LongAck(..) => 0xB4,
            Message::InputRep(..) => 0xB2,
            Message::SwRep(..) => 0xB1,
            Message::SwReq(..) => 0xB0,
            Message::LocoSnd(..) => 0xA2,
            Message::LocoDirf(..) => 0xA1,
            Message::LocoSpd(..) => 0xA0,
            Message::MultiSense(..) => 0xD0,
            Message::UhliFun(..) => 0xD4,
            Message::WrSlData(..) => 0xEF,
            Message::SlRdData(..) => 0xE7,
            Message::ProgrammingAborted(..) => 0xE6,
            Message::PeerXfer(..) => 0xE5,
            Message::Rep(..) => 0xE4,
            Message::ImmPacket(..) => 0xED,
        }
    }

    /// Checks whether this message expects a long acknowledgment message to follow.
    pub fn answer_follows(&self) -> bool {
        0x01 & self.opc() == 0x01
    }

    /// Indicates if a request with the specified slot
    /// data was awaited after that message.
    pub fn await_slot_data(&self) -> bool {
        matches!(
            self,
            Message::LocoAdr(..) |
            Message::RqSlData(..) |
            Message::MoveSlots(..) |
            Message::LinkSlots(..) |
            Message::UnlinkSlots(..)
        )
    }
}
