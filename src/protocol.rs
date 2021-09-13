use crate::error::MessageParseError;
use crate::protocol::args::*;

mod args {
    #[derive(Debug, Copy, Clone)]
    pub struct AddressArg {
        address: u16,
    }

    impl AddressArg {
        pub fn parse(adr2: u8, adr: u8) -> AddressArg {
            let mut address = adr as u16;
            address |= (adr2 as u16) << 7;
            AddressArg { address }
        }

        pub fn address(&self) -> u16 {
            self.address
        }

        pub fn set_address(&mut self, address: u16) {
            assert_eq!(
                address & 0x3FFF,
                0,
                "address must only use the 14 least significant bits"
            );
            self.address = address;
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
    pub struct SlotArg {
        number: u8,
    }

    impl SlotArg {
        pub fn parse(slot: u8) -> SlotArg {
            SlotArg {
                number: slot & 0x7F,
            }
        }

        pub fn number(&self) -> u8 {
            self.number
        }

        pub fn set_number(&mut self, number: u8) {
            assert_eq!(
                number & 0x7F,
                0,
                "number must only use the 7 least significant bits"
            )
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
}

#[repr(u8)]
#[derive(Debug)]
pub enum Message {
    Idle = 0x85,
    GpOn = 0x83,
    GpOff = 0x82,
    Busy = 0x81,

    LocoAdr(AddressArg) = 0xBF,
    SwState(SwitchArg) = 0xBC,
    LocoSpd(SlotArg, SpeedArg) = 0xA0,
}

impl Message {
    pub fn parse(msg: &[u8]) -> Result<Message, MessageParseError> {
        if msg.len() < 2 {
            return Err(MessageParseError::InvalidLength(msg.len()));
        }

        if !Self::validate(msg) {
            return Err(MessageParseError::InvalidChecksum);
        }

        let opc = msg[0];
        if (0x80..=0x8F).contains(&opc) {
            if msg.len() != 2 {
                return Err(MessageParseError::InvalidLength(msg.len()));
            }
            Self::parse2(opc)
        } else if (0xA0..=0xBF).contains(&opc) {
            if msg.len() != 4 {
                return Err(MessageParseError::InvalidLength(msg.len()));
            }
            Self::parse4(opc, &msg[1..3])
        } else if (0xD0..=0xDF).contains(&opc) {
            if msg.len() != 6 {
                return Err(MessageParseError::InvalidLength(msg.len()));
            }
            Self::parse6(opc, &msg[1..5])
        } else {
            let count = msg[1] as usize;
            Self::parse_var(opc, &msg[2..count - 1])
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
            0xBC => Ok(Self::SwState(SwitchArg::parse(args[0], args[1]))),
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

    fn parse_var(opc: u8, args: &[u8]) -> Result<Message, MessageParseError> {
        Err(MessageParseError::UnknownOpcode(opc))
    }

    fn validate(msg: &[u8]) -> bool {
        return msg.iter().fold(0, |acc, &b| acc ^ b) == 0xFF;
    }
}
