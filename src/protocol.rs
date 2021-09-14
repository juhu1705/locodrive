use crate::error::MessageParseError;
use crate::protocol::args::*;

mod args {
    use std::fmt::{Debug, Formatter};

    #[derive(Debug, Copy, Clone)]
    pub struct AddressArg(u16);

    impl AddressArg {
        pub fn parse(adr2: u8, adr: u8) -> AddressArg {
            let mut address = adr as u16;
            address |= (adr2 as u16) << 7;
            AddressArg(address)
        }

        pub fn address(&self) -> u16 {
            self.0
        }

        pub fn set_address(&mut self, address: u16) {
            assert_eq!(
                address & 0x3FFF,
                0,
                "address must only use the 14 least significant bits"
            );
            self.0 = address;
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub enum SwitchDirection {
        Straight,
        Curved,
    }

    #[derive(Debug, Copy, Clone)]
    pub struct SwitchArg {
        address: u16,
        direction: SwitchDirection,
        state: bool,
    }

    impl SwitchArg {
        pub fn parse(sw1: u8, sw2: u8) -> SwitchArg {
            let mut address = sw1 as u16;
            address |= (sw2 as u16 & 0x0F) << 7;

            let direction = if sw2 & 0x20 == 0 {
                SwitchDirection::Curved
            } else {
                SwitchDirection::Straight
            };

            let state = (sw2 & 0x10) != 0;
            SwitchArg {
                address,
                direction,
                state,
            }
        }

        pub fn address(&self) -> u16 {
            self.address
        }
        pub fn direction(&self) -> SwitchDirection {
            self.direction
        }
        pub fn state(&self) -> bool {
            self.state
        }

        pub fn set_address(&mut self, address: u16) {
            assert_eq!(
                address & 0x03FF,
                0,
                "address must only use the 11 least significant bits"
            );
            self.address = address;
        }
        pub fn set_direction(&mut self, direction: SwitchDirection) {
            self.direction = direction;
        }
        pub fn set_state(&mut self, state: bool) {
            self.state = state;
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct SlotArg(u8);

    impl SlotArg {
        pub fn new(number: u8) -> Self {
            SlotArg(number)
        }

        pub fn parse(slot: u8) -> SlotArg {
            SlotArg(slot & 0x7F)
        }

        pub fn number(&self) -> u8 {
            self.0
        }

        pub fn set_number(&mut self, number: u8) {
            assert_eq!(
                number & 0x7F,
                0,
                "number must only use the 7 least significant bits"
            );
            self.0 = number;
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub enum SpeedArg {
        Stop,
        EmergencyStop,
        Drive(u8),
    }

    impl SpeedArg {
        pub fn parse(spd: u8) -> SpeedArg {
            match spd {
                0x00 => SpeedArg::Stop,
                0x01 => SpeedArg::EmergencyStop,
                _ => SpeedArg::Drive(spd - 1),
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct DirfArg(u8);

    impl DirfArg {
        pub fn parse(dirf: u8) -> Self {
            Self(dirf & 0x3F)
        }

        pub fn dir(&self) -> bool {
            self.0 & 0x20 != 0
        }

        pub fn f(&self, f_num: u8) -> bool {
            assert!(f_num <= 4, "f must be lower than or equal to 4");
            self.0 >> (if f_num == 0 { 4 } else { f_num - 1 }) & 1 != 0
        }

        pub fn set_dir(&mut self, value: bool) {
            if value {
                self.0 |= 0x20;
            } else {
                self.0 &= !0x20
            }
        }

        pub fn set_f(&mut self, f_num: u8, value: bool) {
            assert!(f_num <= 4, "f must be lower than or equal to 4");
            let mask = 1 << if f_num == 0 { 4 } else { f_num - 1 };
            if value {
                self.0 |= mask;
            } else {
                self.0 &= !mask;
            }
        }
    }

    impl Debug for DirfArg {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "DirfArg(dir: {}, f0: {}, f1: {}, f2: {}, f3: {}, f4: {})",
                self.dir(),
                self.f(0),
                self.f(1),
                self.f(2),
                self.f(3),
                self.f(4)
            )
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct SndArg(u8);

    impl SndArg {
        pub fn parse(snd: u8) -> Self {
            Self(snd & 0x0F)
        }

        pub fn f(&self, f_num: u8) -> bool {
            assert!(
                (5..=8).contains(&f_num),
                "f_num must be within 5 and 8 (inclusive)"
            );
            self.0 & 1 << (f_num - 5) != 0
        }

        pub fn set_f(&mut self, f_num: u8, value: bool) {
            assert!(
                (5..=8).contains(&f_num),
                "f_num must be within 5 and 8 (inclusive)"
            );
            let mask = 1 << (f_num - 5);
            if value {
                self.0 |= mask;
            } else {
                self.0 &= !mask;
            }
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct LopcArg(u8);

    impl LopcArg {
        pub fn parse(lopc: u8) -> Self {
            Self(lopc & !0x40)
        }

        pub fn lopc(&self) -> u8 {
            self.0
        }

        pub fn set_lopc(&mut self, lopc: u8) {
            assert_eq!(lopc & 0x40, 0, "7th least significant bit must be 0");
            self.0 = lopc
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Ack1Arg(u8);

    impl Ack1Arg {
        pub fn parse(ack1: u8) -> Self {
            Self(ack1)
        }

        pub fn code(&self) -> u8 {
            self.0
        }

        pub fn success(&self) -> bool {
            self.0 != 0
        }

        pub fn set_code(&mut self, code: u8) {
            self.0 = code
        }
    }

    #[derive(Copy, Clone)]
    pub struct InArg {
        address: u16,
        source_type: SourceType,
        state: bool,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum SourceType {
        Aux,
        Switch,
    }

    impl InArg {
        pub fn parse(in1: u8, in2: u8) -> Self {
            let mut address = in1 as u16;
            address |= (in2 as u16 & 0x0F) << 7;

            let source_type = if in2 & 0x20 == 0 {
                SourceType::Aux
            } else {
                SourceType::Switch
            };

            let state = (in2 & 0x10) != 0;
            InArg {
                address,
                source_type,
                state,
            }
        }

        pub fn address(&self) -> u16 {
            self.address
        }
        pub fn address_ds54(&self) -> u16 {
            self.address << 1
                | if self.source_type() == SourceType::Switch {
                    1
                } else {
                    0
                }
        }
        pub fn source_type(&self) -> SourceType {
            self.source_type
        }
        pub fn state(&self) -> bool {
            self.state
        }

        pub fn set_address(&mut self, address: u16) {
            assert_eq!(
                address & 0x03FF,
                0,
                "address must only use the 11 least significant bits"
            );
            self.address = address;
        }

        pub fn set_address_ds54(&mut self, address_ds54: u16) {
            assert_eq!(
                self.address & 0x07FF,
                0,
                "address must only use the 12 least significant bits"
            );
            self.set_source_type(if address_ds54 & 1 == 0 {
                SourceType::Aux
            } else {
                SourceType::Switch
            });
            self.set_address(address_ds54 >> 1);
        }

        pub fn set_source_type(&mut self, source_type: SourceType) {
            self.source_type = source_type;
        }
        pub fn set_state(&mut self, state: bool) {
            self.state = state;
        }
    }

    impl Debug for InArg {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "InArg {{ address: {:?} (DS54: {:?}), source_type: {:?}, state: {:?} }}",
                self.address(),
                self.address_ds54(),
                self.source_type(),
                self.state()
            )
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum Message {
    Idle = 0x85,
    GpOn = 0x83,
    GpOff = 0x82,
    Busy = 0x81,

    LocoAdr(AddressArg) = 0xBF,
    SwAck(SwitchArg) = 0xBD,
    SwState(SwitchArg) = 0xBC,
    RqSlData(SlotArg) = 0xBB,
    MoveSlots(SlotArg, SlotArg) = 0xBA,
    LinkSlots(SlotArg, SlotArg) = 0xB9,
    UnlinkSlots(SlotArg, SlotArg) = 0xB8,
    ConsistFunc(SlotArg, DirfArg) = 0xB6,
    // TODO: SlotStat1 0xB5
    LongAck(LopcArg, Ack1Arg) = 0xB4,
    InputRep(InArg) = 0xB2,
    // TODO: SwRep 0xB1
    SwReq(SwitchArg) = 0xB0,
    LocoSnd(SlotArg, SndArg) = 0xA2,
    LocoDirf(SlotArg, DirfArg) = 0xA1,
    LocoSpd(SlotArg, SpeedArg) = 0xA0,
}

impl Message {
    /// Reads and Parses the next message from `stream`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the message could not be parsed:
    ///
    /// * [`UnknownOpcode`] if the message has an unknown opcode
    /// * [`UnexpectedEnd`] if `stream` unexpectedly yields [`None`]
    /// * [`InvalidChecksum`] if the checksum is invalid
    ///
    /// [`UnknownOpcode`]: MessageParseError::UnknownOpcode
    /// [`UnexpectedEnd`]: MessageParseError::UnexpectedEnd
    /// [`InvalidChecksum`]: MessageParseError::InvalidChecksum
    pub fn parse<I: Iterator<Item = u8>>(stream: &mut I) -> Result<Message, MessageParseError> {
        // create the buffer (a message can be at most 256 bytes long)
        // and map the iterator to store all read bytes in the buffer
        let mut buf = [0u8; 256];
        let mut stream = stream.enumerate().map(|(i, b)| {
            buf[i] = b;
            b
        });

        // get first two bytes from stream
        let (opc, byte1) = match stream.next().zip(stream.next()) {
            Some(bytes) => bytes,
            None => return Err(MessageParseError::UnexpectedEnd),
        };

        // determine length of the message by comparing the ms 3 bytes
        let len = match opc & 0xE0 {
            0x80 => 2,
            0xA0 => 4,
            0xC0 => 6,
            0xE0 => byte1 as usize,
            _ => return Err(MessageParseError::UnknownOpcode(opc)),
        };

        // advance iterator by len - 2 to read full message into buf
        // TODO: replace with `advance_by(len - 2)` when available
        if len > 2 && stream.nth(len - 3) == None {
            return Err(MessageParseError::UnexpectedEnd);
        }

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

    fn parse2(opc: u8) -> Result<Message, MessageParseError> {
        match opc {
            0x85 => Ok(Self::Idle),
            0x83 => Ok(Self::GpOn),
            0x82 => Ok(Self::GpOff),
            0x81 => Ok(Self::Busy),
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    fn parse4(opc: u8, args: &[u8]) -> Result<Message, MessageParseError> {
        assert_eq!(args.len(), 2, "length of args mut be 2");
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
            // TODO: 0xB5 => Ok(Self::SlotStat1(...))
            0xB4 => Ok(Self::LongAck(
                LopcArg::parse(args[0]),
                Ack1Arg::parse(args[1]),
            )),
            0xB2 => Ok(Self::InputRep(InArg::parse(args[0], args[1]))),
            // TODO: 0xB1 => Ok(Self::SwRep(...))
            0xA2 => Ok(Self::LocoSnd(
                SlotArg::parse(args[0]),
                SndArg::parse(args[1]),
            )),
            0xB0 => Ok(Self::SwReq(SwitchArg::parse(args[0], args[1]))),
            0xA1 => Ok(Self::LocoDirf(
                SlotArg::new(args[0]),
                DirfArg::parse(args[1]),
            )),
            0xA0 => Ok(Self::LocoSpd(
                SlotArg::parse(args[0]),
                SpeedArg::parse(args[1]),
            )),
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    fn parse6(opc: u8, args: &[u8]) -> Result<Message, MessageParseError> {
        assert_eq!(args.len(), 4, "length of args mut be 4");
        Err(MessageParseError::UnknownOpcode(opc))
    }

    #[allow(unused_variables)] // TODO: remove allowance when parse_var is implemented
    fn parse_var(opc: u8, args: &[u8]) -> Result<Message, MessageParseError> {
        Err(MessageParseError::UnknownOpcode(opc))
    }

    fn validate(msg: &[u8]) -> bool {
        return msg.iter().fold(0, |acc, &b| acc ^ b) == 0xFF;
    }
}
