use crate::error::MessageParseError;
use crate::protocol::args::*;

mod args {
    use std::fmt::{Debug, Formatter};
    use std::time::Duration;

    #[derive(Debug, Copy, Clone)]
    pub struct AddressArg(u16);

    impl AddressArg {
        pub fn parse(adr2: u8, adr: u8) -> Self {
            let mut address = adr as u16;
            address |= (adr2 as u16) << 7;
            Self(address)
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

        pub fn adr1(&self) -> u8 {
            (self.0 & 0x007F) as u8
        }

        pub fn adr2(&self) -> u8 {
            ((self.0 >> 7) & 0x007F) as u8
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
        pub fn parse(sw1: u8, sw2: u8) -> Self {
            let mut address = sw1 as u16;
            address |= (sw2 as u16 & 0x0F) << 7;

            let direction = if sw2 & 0x20 == 0 {
                SwitchDirection::Curved
            } else {
                SwitchDirection::Straight
            };

            let state = (sw2 & 0x10) != 0;
            Self {
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
                address & 0x07FF,
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

        pub fn sw1(&self) -> u8 {
            (self.address & 0x007F) as u8
        }

        pub fn sw2(&self) -> u8 {
            let mut sw2 = ((self.address >> 7) & 0x000F) as u8;

            sw2 |= match self.direction {
                SwitchDirection::Curved => 0x20,
                SwitchDirection::Straight => 0x00
            };

            if self.state {
                sw2 |= 0x10;
            }

            sw2
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct SlotArg(u8);

    impl SlotArg {
        pub fn parse(slot: u8) -> Self {
            Self(slot & 0x7F)
        }

        pub fn slot(&self) -> u8 {
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
        pub fn parse(spd: u8) -> Self {
            match spd {
                0x00 => Self::Stop,
                0x01 => Self::EmergencyStop,
                _ => Self::Drive(spd - 1),
            }
        }

        pub fn spd(&self) -> u8 {
            match *self {
                SpeedArg::Stop => 0x00,
                SpeedArg::EmergencyStop => 0x01,
                SpeedArg::Drive(spd) => spd
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

        pub fn dirf(&self) -> u8 {
            self.0
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
    pub struct TrkArg {
        power: bool,
        idle: bool,
        mlok1: bool,
        prog_busy: bool,
    }

    impl TrkArg {
        pub fn parse(trk_arg: u8) -> Self {
            let power = trk_arg & 0x01 == 0x01;
            let idle = trk_arg & 0x02 == 0x00;
            let mlok1 = trk_arg & 0x04 == 0x04;
            let prog_busy = trk_arg & 0x08 == 0x08;
            TrkArg {
                power,
                idle,
                mlok1,
                prog_busy,
            }
        }

        pub fn power_on(&self) -> bool {
            self.power
        }

        pub fn track_idle(&self) -> bool {
            self.idle
        }

        pub fn mlok1(&self) -> bool {
            self.mlok1
        }

        pub fn prog_busy(&self) -> bool {
            self.prog_busy
        }

        pub fn trk_arg(&self) -> u8 {
            let mut trk_arg = if self.power { 0x01 } else { 0x00 };
            if self.idle {
                trk_arg |= 0x02;
            }
            if self.mlok1 {
                trk_arg |= 0x04;
            }
            if self.prog_busy {
                trk_arg |= 0x08;
            }
            trk_arg
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

        pub fn snd(&self) -> u8 {
            self.0
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Stat1Arg {
        spurge: bool,
        consist: Consist,
        state: State,
        decoder_type: DecoderType,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum Consist {
        LogicalMid,
        LogicalTop,
        LogicalSubMember,
        Free,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum State {
        InUse,
        Idle,
        Common,
        Free,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum DecoderType {
        Dcc28,
        Dcc128,
        Regular28,
        AdrMobile28,
        Step14,
        Speed128,
    }

    impl Stat1Arg {
        pub fn parse(stat1: u8) -> Self {
            let spurge = stat1 & 0x80 != 0;

            let consist = match stat1 & 0x48 {
                0x48 => Consist::LogicalMid,
                0x08 => Consist::LogicalTop,
                0x40 => Consist::LogicalSubMember,
                0x00 => Consist::Free,
                _ => panic!("No valid consist is given!"),
            };

            let state = match stat1 & 0x30 {
                0x30 => State::InUse,
                0x20 => State::Idle,
                0x10 => State::Common,
                0x00 => State::Free,
                _ => panic!("No valid state is given!"),
            };

            let decoder_type = match stat1 & 0x07 {
                0x02 => DecoderType::Step14,
                0x01 => DecoderType::AdrMobile28,
                0x00 => DecoderType::Regular28,
                0x03 => DecoderType::Speed128,
                0x07 => DecoderType::Dcc128,
                0x04 => DecoderType::Dcc28,
                _ => panic!("The given decoder type was invalid!"),
            };

            Stat1Arg {
                spurge,
                consist,
                state,
                decoder_type,
            }
        }

        pub fn spurge(&self) -> bool {
            self.spurge
        }

        pub fn consist(&self) -> Consist {
            self.consist
        }

        pub fn state(&self) -> State {
            self.state
        }

        pub fn decoder_type(&self) -> DecoderType {
            self.decoder_type
        }

        pub fn stat1(&self) -> u8 {
            let mut stat1: u8 = if self.spurge { 0x80 } else { 0x00 };

            stat1 |= match self.consist {
                Consist::LogicalMid => 0x48,
                Consist::LogicalTop => 0x08,
                Consist::LogicalSubMember => 0x40,
                Consist::Free => 0x00
            };

            stat1 |= match self.state {
                State::InUse => 0x30,
                State::Idle => 0x20,
                State::Common => 0x10,
                State::Free => 0x00
            };

            stat1 |= match self.decoder_type {
                DecoderType::Dcc28 => 0x04,
                DecoderType::Dcc128 => 0x07,
                DecoderType::Regular28 => 0x00,
                DecoderType::AdrMobile28 => 0x01,
                DecoderType::Step14 => 0x02,
                DecoderType::Speed128 => 0x03
            };

            stat1
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Stat2Arg {
        has_adv: bool,
        no_id_usage: bool,
        id_encoded_alias: bool,
    }

    impl Stat2Arg {
        pub fn parse(stat2: u8) -> Self {
            let has_adv = stat2 & 0x01 != 0;

            let no_id_usage = stat2 & 0x04 != 0;

            let id_encoded_alias = stat2 & 0x08 != 0;

            Stat2Arg {
                has_adv,
                no_id_usage,
                id_encoded_alias,
            }
        }

        pub fn has_adv(&self) -> bool {
            self.has_adv
        }

        pub fn no_id_usage(&self) -> bool {
            self.no_id_usage
        }

        pub fn id_encoded_alias(&self) -> bool {
            self.id_encoded_alias
        }

        pub fn stat2(&self) -> u8 {
            let mut stat2 = if self.has_adv { 0x01 } else { 0x00 };
            if self.no_id_usage {
                stat2 |= 0x04;
            }
            if self.id_encoded_alias {
                stat2 |= 0x08;
            }
            stat2
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

        pub fn ack1(&self) -> u8 {
            self.0
        }

        pub fn success(&self) -> bool {
            self.0 != 0
        }

        pub fn is_busy(&self) -> bool {
            self.0 == 0
        }

        pub fn accepted(&self) -> bool {
            self.0 == 1
        }

        pub fn accepted_blind(&self) -> bool {
            self.0 == 0x40
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
            Self {
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
                address & 0x07FF,
                0,
                "address must only use the 11 least significant bits"
            );
            self.address = address;
        }

        pub fn set_address_ds54(&mut self, address_ds54: u16) {
            assert_eq!(
                self.address & 0x0FFF,
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

        pub fn in1(&self) -> u8 {
            self.address as u8 & 0x7F
        }

        pub fn in2(&self) -> u8 {
            let mut in2 = ((self.address >> 7) as u8) & 0x0F;
            in2 |= match self.source_type {
                SourceType::Aux => 0x00,
                SourceType::Switch => 0x20
            };
            if self.state {
                in2 |= 0x10;
            }
            in2
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

    #[derive(Debug, Copy, Clone)]
    pub struct SnArg {
        address: u16,
        format: bool,
        c: bool,
        t: bool,
    }

    impl SnArg {
        pub fn parse(sn1: u8, sn2: u8) -> Self {
            let mut address = sn1 as u16;
            address |= (sn2 as u16 & 0x0F) << 7;

            let format = sn2 & 0x20 == 0x20;

            let c = sn2 & 0x40 == 0x40;
            let t = sn2 & 0x80 == 0x80;

            SnArg {
                address,
                format,
                c,
                t,
            }
        }

        pub fn sn1(&self) -> u8 {
            (self.address as u8) & 0x7F
        }

        pub fn sn2(&self) -> u8 {
            let mut sn2 = (self.address >> 7) as u8 & 0x0F;
            if self.c {
                sn2 |= 0x40;
            }
            if self.t {
                sn2 |= 0x80;
            }
            sn2
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct IdArg(u8, u8);

    impl IdArg {
        pub fn parse(id1: u8, id2: u8) -> Self {
            IdArg(id1 & 0xF7, id2 & 0xF7)
        }

        pub fn id1(&self) -> u8 {
            self.0
        }

        pub fn id2(&self) -> u8 {
            self.1
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct MTypeArg(u8);

    impl MTypeArg {
        pub fn parse(m_type: u8) -> Self {
            MTypeArg(m_type)
        }

        pub fn m_type(&self) -> u8 {
            self.0
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct ZasArg(u8);

    impl ZasArg {
        pub fn parse(zone_and_section: u8) -> Self {
            ZasArg(zone_and_section)
        }

        pub fn zas(&self) -> u8 {
            self.0
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct SenseAddrArg(u16);

    impl SenseAddrArg {
        pub fn parse(addr1: u8, addr2: u8) -> Self {
            let mut address = addr1 as u16;
            address |= (addr2 as u16) << 7;

            SenseAddrArg(address)
        }

        pub fn addr1(&self) -> u8 {
            self.0 as u8 & 0x7F
        }

        pub fn addr2(&self) -> u8 {
            (self.0 >> 7) as u8
        }
    }

    #[derive(Copy, Clone)]
    pub struct FunctionArg(u8, u8);

    impl FunctionArg {
        pub fn parse(group: u8, function: u8) -> Self {
            FunctionArg(group, function)
        }

        pub fn f(&self, f_num: u8) -> bool {
            if f_num > 8 && f_num < 12 && self.0 == 0x07 {
                (self.1 >> (f_num - 9) & 1) != 0
            } else if (f_num == 12 || f_num == 20 || f_num == 28) && self.0 == 0x05 {
                (self.1
                    >> (if f_num == 12 {
                        0
                    } else if f_num == 20 {
                        1
                    } else {
                        2
                    })
                    & 1)
                    != 0
            } else if f_num > 12 && f_num < 20 && self.0 == 0x08 {
                (self.1 >> (f_num - 13) & 1) != 0
            } else if f_num > 20 && f_num < 28 && self.0 == 0x09 {
                (self.1 >> (f_num - 21) & 1) != 0
            } else {
                false
            }
        }

        pub fn set_f(&mut self, f_num: u8, value: bool) {
            assert!(f_num <= 4, "f must be lower than or equal to 4");
            let mask = 1 << (f_num - 9);
            if value {
                self.1 |= mask;
            } else {
                self.1 &= !mask;
            }
        }

        pub fn group(&self) -> u8 {
            self.0
        }

        pub fn function(&self) -> u8 {
            self.1
        }
    }

    impl Debug for FunctionArg {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "DirfArg(f9: {}, f10: {}, f11: {}, f12: {}, f13: {}, f14: {}, f15: {}, f16: {}, f17: {}, f18: {}, f19: {}, f20: {}, f21: {}, f22: {}, f23: {}, f24: {}, f25: {}, f26: {}, f27: {}, f28: {})",
                self.f(9),
                self.f(10),
                self.f(11),
                self.f(12),
                self.f(13),
                self.f(14),
                self.f(15),
                self.f(16),
                self.f(17),
                self.f(18),
                self.f(19),
                self.f(20),
                self.f(21),
                self.f(22),
                self.f(23),
                self.f(24),
                self.f(25),
                self.f(26),
                self.f(27),
                self.f(28),
            )
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Pcmd {
        write: bool,
        byte_mode: bool,
        ops_mode: bool,
        ty1: bool, // Programing type select bit
        ty2: bool, // prog type select bit
    }

    impl Pcmd {
        pub fn parse(pcmd: u8) -> Self {
            let write = pcmd & 0x20 == 0x20;
            let byte_mode = pcmd & 0x40 == 0x40;
            let ops_mode = pcmd & 0x02 == 0x02;
            let ty1 = pcmd & 0x80 == 0x80;
            let ty2 = pcmd & 0x01 == 0x01;

            Pcmd {
                write,
                byte_mode,
                ops_mode,
                ty1,
                ty2,
            }
        }

        pub fn pcmd(&self) -> u8 {
            let mut pcmd = if self.write { 0x20 } else { 0x00 };
            if self.byte_mode {
                pcmd |= 0x40;
            }
            if self.ops_mode {
                pcmd |= 0x02;
            }
            if self.ty1 {
                pcmd |= 0x80;
            }
            if self.ty2 {
                pcmd |= 0x01;
            }
            pcmd
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct PStat {
        user_aborted: bool,
        no_read_ack: bool,
        no_write_ack: bool,
        programming_track_empty: bool,
    }

    impl PStat {
        pub fn parse(stat: u8) -> Self {
            let user_aborted = stat & 0x01 == 0x01;
            let no_read_ack = stat & 0x02 == 0x02;
            let no_write_ack = stat & 0x04 == 0x04;
            let programming_track_empty = stat & 0x08 == 0x08;

            PStat {
                user_aborted,
                no_read_ack,
                no_write_ack,
                programming_track_empty,
            }
        }

        pub fn stat(&self) -> u8 {
            let mut stat = if self.user_aborted { 0x01 } else { 0x00 };
            if self.no_read_ack {
                stat |= 0x02;
            }
            if self.no_write_ack {
                stat |= 0x04;
            }
            if self.programming_track_empty {
                stat |= 0x08;
            }
            stat
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Hopsa(u8);

    impl Hopsa {
        pub fn parse(o_mode: u8) -> Self {
            Hopsa(o_mode & 0x7F)
        }

        pub fn service_mode(&self) -> bool {
            self.0 == 0
        }

        pub fn o_mode(&self) -> u8 {
            self.0
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Lopsa(u8);

    impl Lopsa {
        pub fn parse(o_mode: u8) -> Self {
            Lopsa(o_mode & 0x7F)
        }

        pub fn service_mode(&self) -> bool {
            self.0 == 0
        }

        pub fn o_mode(&self) -> u8 {
            self.0
        }
    }

    #[derive(Copy, Clone)]
    pub struct CvArg(u16);

    impl CvArg {
        pub fn parse(cvh: u8, cvl: u8) -> Self {
            let mut cv_arg = cvl as u16;

            let data_arg = (cvh & 0x02) >> 1;
            let mut high_cv_arg = cvh & 0x01;
            high_cv_arg |= (cvh & 0x30) >> 3;
            high_cv_arg |= (data_arg) << 3;

            cv_arg |= (high_cv_arg as u16) << 7;

            CvArg(cv_arg)
        }

        pub fn data7(&self) -> bool {
            self.0 & 0x0800 != 0
        }

        pub fn cv(&self, cv_num: u8) -> bool {
            assert!(cv_num <= 9, "cv must be lower than or equal to 9");
            self.0 >> cv_num & 1 != 0
        }

        pub fn set_data7(&mut self, value: bool) {
            if value {
                self.0 |= 0x0800;
            } else {
                self.0 &= !0x0800;
            }
        }

        pub fn set_cv(&mut self, cv_num: u8, value: bool) {
            assert!(cv_num <= 9, "cv must be lower than or equal to 9");
            let mask = 1 << cv_num;
            if value {
                self.0 |= mask;
            } else {
                self.0 &= !mask;
            }
        }

        pub fn cvh(&self) -> u8 {
            let mut cvh = (self.0 >> 7) as u8;
            let high_cv = cvh & 0x06 << 3;
            cvh &= 0x01;
            cvh |= high_cv;
            if self.data7() {
                cvh |= 0x02;
            }
            cvh
        }

        pub fn cvl(&self) -> u8 {
            self.0 as u8 & 0x7F
        }
    }

    impl Debug for CvArg {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "CvArg(data7: {}, cv0: {}, cv1: {}, cv2: {}, cv3: {}, cv4: {}, cv5: {}, cv6: {}, cv7: {}, cv8: {})",
                self.data7(),
                self.cv(0),
                self.cv(1),
                self.cv(2),
                self.cv(3),
                self.cv(4),
                self.cv(5),
                self.cv(6),
                self.cv(7),
                self.cv(8),
            )
        }
    }

    #[derive(Copy, Clone)]
    pub struct DataArg(u8);

    impl DataArg {
        pub fn parse(data: u8) -> Self {
            DataArg(data)
        }

        pub fn d(&self, d_num: u8) -> bool {
            assert!(d_num <= 6, "d must be lower than or equal to 6");
            self.0 >> d_num & 1 != 0
        }

        pub fn set_d(&mut self, d_num: u8, value: bool) {
            assert!(d_num <= 6, "d must be lower than or equal to 6");
            let mask = 1 << d_num;
            if value {
                self.0 |= mask;
            } else {
                self.0 &= !mask;
            }
        }

        pub fn data(&self) -> u8 {
            self.0
        }
    }

    impl Debug for DataArg {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "DataArg(d0: {}, d1: {}, d2: {}, d3: {}, d4: {}, d5: {}, d6: {})",
                self.d(0),
                self.d(1),
                self.d(2),
                self.d(3),
                self.d(4),
                self.d(5),
                self.d(6),
            )
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct ClkRateArg(u8);

    impl ClkRateArg {
        pub fn parse(clk_rate: u8) -> Self {
            ClkRateArg(clk_rate & 0x7F)
        }

        pub fn set_rate(&mut self, clk_rate: u8) {
            assert!(
                clk_rate > 0x7F,
                "Clock rate {:?} is to high. Only values up to 0x7F are allowed",
                clk_rate
            );

            self.0 = clk_rate & 0x7F;
        }

        pub fn clk_rate(&self) -> u8 {
            self.0
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct FastClock {
        clk_rate: u8,
        frac_minsl: u8,
        frac_minsh: u8,
        duration: Duration,
        clk_cntrl: u8,
    }

    impl FastClock {
        pub fn parse(clk_rate: u8, frac_minsl: u8, frac_minsh: u8, mins: u8, hours: u8, days: u8, clk_cntrl: u8) -> Self {
            let min = mins % 60 - 0xFF;
            let hour = hours % 60 - 0xFF;

            let secs: u64 = min as u64 * 60 + hour as u64 * 60 * 60 + days as u64 * 24 * 60 * 60;

            let duration = Duration::new(secs, 0);

            FastClock {
                clk_rate: clk_rate & 0x7F,
                frac_minsl,
                frac_minsh,
                duration,
                clk_cntrl,
            }
        }

        pub fn clk_rate(&self) -> u8 {
            self.clk_rate
        }

        pub fn frac_minsl(&self) -> u8 {
            self.frac_minsl
        }

        pub fn frac_minsh(&self) -> u8 {
            self.frac_minsh
        }

        pub fn mins(&self) -> u8 {
            0xFF - (self.duration.as_secs() % 60) as u8
        }

        pub fn hours(&self) -> u8 {
            0xFF - (self.duration.as_secs() / 60 % 60) as u8
        }

        pub fn days(&self) -> u8 {
            0xFF - (self.duration.as_secs() / 60 / 60 % 24) as u8
        }

        pub fn clk_cntrl(&self) -> u8 {
            self.clk_cntrl
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct ImArg{
        reps: u8,
        dhi: u8,
        address: u16,
        function_type: u8,
        function_bits: u8,
        im5: u8
    }

    impl ImArg {
        pub fn parse(check_byte: u8, reps: u8, dhi: u8, im1: u8, im2 : u8, im3: u8, im4: u8, im5: u8) -> ImArg {
            assert_eq!(check_byte, 0x7F, "Checkbyte of ImmPacket is not 0x7F");

            if reps == 0x34 {
                let address = ((im2 as u16) << 8) | im1 as u16;

                let function_type = if im3 == 0x5E { 0x5E } else if im3 == 0x5F { 0x5F } else { 0x20 };
                let mut function_bits = if function_type == 0x5E || function_type == 0x5F { im4 } else { im3 };

                function_bits = function_bits & 0x7F;

                Self {reps, dhi, address, function_type, function_bits, im5}
            } else {
                let address = im1 as u16;

                let function_type = if im3 == 0x5E { 0x5E } else if im3 == 0x5F { 0x5F } else { 0x20 };
                let mut function_bits = if function_type == 0x5E || function_type == 0x5F { im3 } else { im2 & 0xDF };

                function_bits = function_bits & 0x7F;

                Self {reps, dhi, address, function_type, function_bits, im5}
            }
        }

        pub fn check_byte() -> u8 {
            0x7F
        }

        pub fn reps(&self) -> u8 {
            self.reps
        }

        pub fn dhi(&self) -> u8 {
            self.dhi
        }

        pub fn im1(&self) -> u8 {
            self.address as u8
        }

        pub fn im2(&self) -> u8 {
            return if self.reps == 0x34 {
                (self.address >> 8) as u8
            } else {
                if self.function_type == 0x20 {
                    self.function_bits | 0x20
                } else {
                    self.function_type
                }
            }
        }

        pub fn im3(&self) -> u8 {
            return if self.reps() == 0x34 {
                if self.function_type == 0x20 {
                    self.function_bits | 0x20
                } else {
                    self.function_type
                }
            } else {
                if self.function_type == 0x20 {
                    0x00
                } else {
                    self.function_bits
                }
            }
        }

        pub fn im4(&self) -> u8 {
            if self.reps() == 0x34 {
                if self.function_type != 0x20 {
                    return self.function_bits;
                }
            }
            0x00
        }

        pub fn im5(&self) -> u8 {
            self.im5
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct WrSlDataTime(FastClock, TrkArg, IdArg);

    impl WrSlDataTime {
        pub fn parse(clk_rate: u8, frac_minsh: u8, frac_minsl: u8, mins: u8, trk: u8, hours: u8, days: u8, clk_cntr: u8, id1: u8, id2: u8) -> Self {
            WrSlDataTime(FastClock::parse(clk_rate, frac_minsl, frac_minsh, mins, hours, days, clk_cntr),
                         TrkArg::parse(trk),
                         IdArg::parse(id1, id2))
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct WrSlDataPt(Pcmd, Hopsa, Lopsa, TrkArg, CvArg, DataArg);

    impl WrSlDataPt {
        pub fn parse(pcmd: u8, arg3: u8, hopsa: u8, lopsa: u8, trk: u8, cvh: u8, cvl: u8, data7: u8, arg10: u8, arg11: u8) -> Self {
            WrSlDataPt(Pcmd::parse(pcmd), Hopsa::parse(hopsa), Lopsa::parse(lopsa), TrkArg::parse(trk), CvArg::parse(cvh, cvl), DataArg::parse(data7))
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct WrSlDataGeneral(SlotArg, Stat1Arg, Stat2Arg, AddressArg, SpeedArg, DirfArg, TrkArg, SndArg, IdArg);

    impl WrSlDataGeneral {
        pub fn parse(slot: u8, stat1: u8, adr: u8, spd: u8, dirf: u8, trk: u8, stat2: u8, adr2: u8, snd: u8, id1: u8, id2: u8) -> Self {
            WrSlDataGeneral(SlotArg::parse(slot),
                            Stat1Arg::parse(stat1),
                            Stat2Arg::parse(stat2),
                            AddressArg::parse(adr2, adr),
                            SpeedArg::parse(spd),
                            DirfArg::parse(dirf),
                            TrkArg::parse(trk),
                            SndArg::parse(snd),
                            IdArg::parse(id1, id2))
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct WrSlDataStructure {
        slot_type: u8,
        time_slot: WrSlDataTime,
        pt_slot: WrSlDataPt,
        general_slot: WrSlDataGeneral
    }

    impl WrSlDataStructure {
        pub fn parse(arg1: u8, arg2: u8, arg3: u8, arg4: u8, arg5: u8, arg6: u8, arg7: u8, arg8: u8, arg9: u8, arg10: u8, arg11: u8) -> Self {
            let slot_type = if arg1 == 0x7C { 0x7C } else if arg1 == 0x7B { 0x7B } else { 0x00 };

            let time_slot = WrSlDataTime::parse(arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11);
            let pt_slot = WrSlDataPt::parse(arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11);
            let general_slot = WrSlDataGeneral::parse(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11);

            WrSlDataStructure{ slot_type, time_slot, pt_slot, general_slot }
        }

        pub fn to_message(&self) -> Vec<u8> {
            if self.slot_type == 0x7C {
                return vec![0xEF as u8, 0x0E as u8, 0x7C as u8, self.pt_slot.0.pcmd(), 0x00 as u8, self.pt_slot.1.o_mode(), self.pt_slot.2.o_mode(), self.pt_slot.3.trk_arg(), self.pt_slot.4.cvh(), self.pt_slot.4.cvl(), self.pt_slot.5.data(), 0x00 as u8, 0x00 as u8];
            } else if self.slot_type == 0x7B {
                return vec![0xEF as u8, 0x0E as u8, 0x7B as u8, self.time_slot.0.clk_rate(), self.time_slot.0.frac_minsl(), self.time_slot.0.frac_minsh(), self.time_slot.0.mins(), self.time_slot.1.trk_arg(), self.time_slot.0.hours(), self.time_slot.0.days(), self.time_slot.0.clk_cntrl(), self.time_slot.2.id1(), self.time_slot.2.id2()];
            }
            return vec![0xEF as u8, 0x0E as u8, self.general_slot.0.slot(), self.general_slot.1.stat1(), self.general_slot.3.adr1(), self.general_slot.4.spd(), self.general_slot.5.dirf(), self.general_slot.6.trk_arg(), self.general_slot.2.stat2(), self.general_slot.3.adr2(), self.general_slot.7.snd(), self.general_slot.8.id1(), self.general_slot.8.id2()];
        }
    }

}

#[repr(u8)]
#[derive(Debug)]
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
    MultiSense(MTypeArg, ZasArg, SenseAddrArg),
    UhliFun(SlotArg, FunctionArg),
    WrSlData(WrSlDataStructure),
    SlRdData(SlotArg, Stat1Arg, AddressArg, SpeedArg, DirfArg, TrkArg, Stat2Arg, SndArg, IdArg),
    ImmPacket(ImArg),
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
    pub fn parse<I: Iterator<Item = u8>>(stream: &mut I) -> Result<Self, MessageParseError> {
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
        assert_eq!(args.len(), 4, "length of args mut be 4");
        match opc {
            0xD0 => Ok(Self::MultiSense(
                MTypeArg::parse(args[0]),
                ZasArg::parse(args[1]),
                SenseAddrArg::parse(args[2], args[3]),
            )),
            0xD4 => {
                assert_eq!(0x20, args[0], "Value of arg0 can only be {:?}", 0x20);
                Ok(Self::UhliFun(
                    SlotArg::parse(args[1]),
                    FunctionArg::parse(args[2], args[3]),
                ))
            }
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    #[allow(unused_variables)] // TODO: remove allowance when parse_var is implemented
    fn parse_var(opc: u8, args: &[u8]) -> Result<Self, MessageParseError> {
        assert_eq!(
            args.len() as u8 + 2,
            args[0],
            "length of args mut be {:?}",
            args[0]
        );
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
            0xED => Ok(Self::ImmPacket(
                ImArg::parse(args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8])
            )),
            0xEF => Ok(Self::WrSlData(
                WrSlDataStructure::parse(args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8], args[9], args[10], args[11])
            )),
            _ => Err(MessageParseError::UnknownOpcode(opc)),
        }
    }

    fn validate(msg: &[u8]) -> bool {
        return msg.iter().fold(0, |acc, &b| acc ^ b) == 0xFF;
    }

    pub fn to_message(&self) -> Vec<u8> {
        let mut message = match *self {
            Message::Idle => vec![0x85 as u8],
            Message::GpOn => vec![0x83 as u8],
            Message::GpOff => vec![0x82 as u8],
            Message::Busy => vec![0x81 as u8],
            Message::LocoAdr(adr_arg) => vec![0xBF as u8, adr_arg.adr2(), adr_arg.adr1()],
            Message::SwAck(switch_arg) => vec![0xBD as u8, switch_arg.sw1(), switch_arg.sw2()],
            Message::SwState(switch_arg) => vec![0xBC as u8, switch_arg.sw1(), switch_arg.sw2()],
            Message::RqSlData(slot_arg) => vec![0xBB as u8, slot_arg.slot(), 0x00 as u8],
            Message::MoveSlots(src, dst) => vec![0xBA as u8, src.slot(), dst.slot()],
            Message::LinkSlots(sl1, sl2) => vec![0xB9 as u8, sl1.slot(), sl2.slot()],
            Message::UnlinkSlots(sl1, sl2) => vec![0xB8 as u8, sl1.slot(), sl2.slot()],
            Message::ConsistFunc(slot, dirf) => vec![0xB6 as u8, slot.slot(), dirf.dirf()],
            Message::SlotStat1(slot, stat1) => vec![0xB5 as u8, slot.slot(), stat1.stat1()],
            Message::LongAck(lopc, ack1) => vec![0xB4 as u8, lopc.lopc(), ack1.ack1()],
            Message::InputRep(input) => vec![0xB2 as u8, input.in1(), input.in2()],
            Message::SwRep(sn_arg) => vec![0xB1 as u8, sn_arg.sn1(), sn_arg.sn2()],
            Message::SwReq(sw) => vec![0xB0 as u8, sw.sw1(), sw.sw2()],
            Message::LocoSnd(slot, snd) => vec![0xA2 as u8, slot.slot(), snd.snd()],
            Message::LocoDirf(slot, dirf) => vec![0xA1 as u8, slot.slot(), dirf.dirf()],
            Message::LocoSpd(slot, spd) => vec![0xA0 as u8, slot.slot(), spd.spd()],
            Message::MultiSense(m_type, zas, sense_adr) => vec![0xD0 as u8, m_type.m_type(), zas.zas(), sense_adr.addr1(), sense_adr.addr2()],
            Message::UhliFun(slot, function) => vec![0xD4 as u8, 0x20 as u8, slot.slot(), function.group(), function.function()],
            Message::WrSlData(wr_slot_data_arg) => wr_slot_data_arg.to_message(),
            Message::SlRdData(slot, stat1, adr, spd,
                              dirf, trk, stat2, snd, id) =>
                vec![0xE7 as u8, 0x0E as u8, slot.slot(), stat1.stat1(), adr.adr1(), spd.spd(), dirf.dirf(), trk.trk_arg(),
                    stat2.stat2(), adr.adr2(), snd.snd(), id.id1(), id.id2()],
            Message::ImmPacket(im) => vec![0xED as u8, 0x0B as u8, 0x7F as u8, im.reps(), im.dhi(), im.im1(), im.im2(), im.im3(), im.im4(), im.im5()]
        };

        message.append(&mut vec![Self::check_sum(&message)]);

        return message;
    }

    fn check_sum(msg: &[u8]) -> u8 {
        msg.iter().fold(0, |acc, &b| acc ^ b)
    }
}
