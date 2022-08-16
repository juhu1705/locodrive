#![allow(clippy::too_many_arguments)]

use std::fmt::{Debug, Display, Formatter};
use crate::error::MessageParseError;
use crate::protocol::Message;

/// Represents a trains address of 14 byte length.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AddressArg(u16);

impl AddressArg {
    /// Creates a new address.
    ///
    /// Please consider keeping in range between 0 and 16383.
    /// Higher values may not be supported by this address implementation.
    pub fn new(adr: u16) -> Self {
        Self(adr)
    }

    /// Parses the message bytes from a model railroads message into an `AddressArg`
    ///
    /// # Parameters
    ///
    /// - `adr`: seven least significant loco address bits
    /// - `adr2`: seven most significant loco address bits
    pub(crate) fn parse(adr2: u8, adr: u8) -> Self {
        let mut address = adr as u16;
        address |= (adr2 as u16) << 7;
        Self(address)
    }

    /// # Returns
    ///
    /// The address hold by this arg
    pub fn address(&self) -> u16 {
        self.0
    }

    /// Sets the address hold by this [`AddressArg`]
    ///
    /// Please consider keeping in range between 0 and 16383.
    /// Higher values may not be supported by this address implementation.
    pub fn set_address(&mut self, address: u16) {
        self.0 = address;
    }

    /// # Returns
    ///
    /// seven least significant loco address bits
    pub(crate) fn adr1(&self) -> u8 {
        (self.0 & 0x007F) as u8
    }

    /// # Returns
    ///
    /// seven most significant loco address bits
    pub(crate) fn adr2(&self) -> u8 {
        ((self.0 >> 7) & 0x007F) as u8
    }
}

/// Which direction state a switch is orientated to
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SwitchDirection {
    Straight,
    Curved,
}

/// Holds switch state information to be read or write
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SwitchArg {
    /// The address of the switch (0 - 2047)
    address: u16,
    /// The switches direction state
    direction: SwitchDirection,
    /// The activation state of the switch (`true = ON, false = OFF`)
    state: bool,
}

impl SwitchArg {
    /// Creates a new switch information block that can be send to update a switch in a
    /// model railroad system using the corresponding [`crate::protocol::Message::SwReq`] message.
    ///
    /// # Parameters
    ///
    /// - `address`: The address of the switch you want to change state (0 to 2047)
    /// - `direction`: The direction the switch should switch to
    /// - `state`: The activation state of the switch (`true = ON`, `false = OFF`)
    pub fn new(address: u16, direction: SwitchDirection, state: bool) -> Self {
        Self {
            address,
            direction,
            state,
        }
    }

    /// Parses the arguments of an incoming model railroads message to a [`SwitchArg`].
    ///
    /// # Parameters
    ///
    /// - `sw1`: Seven least significant switch address bits
    /// - `sw2`: four most significant switch address bits,
    ///          1 bit for direction and
    ///          1 bit for activation state
    pub(crate) fn parse(sw1: u8, sw2: u8) -> Self {
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

    /// # Returns
    ///
    /// The address of the switch
    pub fn address(&self) -> u16 {
        self.address
    }
    /// # Returns
    ///
    /// The switches direction state
    pub fn direction(&self) -> SwitchDirection {
        self.direction
    }
    /// # Returns
    ///
    /// The switches activation status (`true = ON, false = OFF`)
    pub fn state(&self) -> bool {
        self.state
    }

    /// Sets the address of the switch to use.
    ///
    /// # Parameters
    ///
    /// - `address`: The switches address (0 - 2047)
    pub fn set_address(&mut self, address: u16) {
        self.address = address;
    }
    /// Sets the direction to switch to.
    ///
    /// # Parameters
    ///
    /// - `direction`: The switches direction
    pub fn set_direction(&mut self, direction: SwitchDirection) {
        self.direction = direction;
    }
    /// Sets the activation state of the switch.
    ///
    /// # Parameters
    ///
    /// - `state`: The switches activation state to set (`true = ON, false = OFF`)
    pub fn set_state(&mut self, state: bool) {
        self.state = state;
    }

    /// # Returns
    ///
    /// The seven least significant address bits.
    pub(crate) fn sw1(&self) -> u8 {
        (self.address & 0x007F) as u8
    }

    /// # Returns
    ///
    /// The four most significant address bits combined with a direction state and activation state.
    pub(crate) fn sw2(&self) -> u8 {
        let mut sw2 = ((self.address >> 7) & 0x000F) as u8;

        sw2 |= match self.direction {
            SwitchDirection::Curved => 0x00,
            SwitchDirection::Straight => 0x20,
        };

        if self.state {
            sw2 |= 0x10;
        }

        sw2
    }
}

/// Represents one slots address between 0 to 127.
///
/// Note that some slots are special handled slots and therefore can not be used (read/write) as normal slots.
///
/// # Slots
///
/// | Nr.     | Function                           |
/// |---------|------------------------------------|
/// | 0       | dispatch                           |
/// | 1-119   | active locs (normal slots)         |
/// | 120-127 | reserved (system / master control) |
/// | - 123   | fast clock                         |
/// | - 124   | programming track                  |
/// | - 127   | command station options            |
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SlotArg(u8);

impl SlotArg {
    /// Creates a new slots address in range of 0 to 127.
    ///
    /// Please consider that the special slots (0, 120 - 127) may not work
    /// as you expect other slots to do.
    ///
    /// # Parameter
    ///
    /// - `slot`: The slots address to set
    pub fn new(slot: u8) -> Self {
        Self(slot & 0x7F)
    }

    /// Parses an incoming slot message from a model railroads message.
    ///
    /// # Parameter
    ///
    /// - `slot`: The slots address to set
    pub(crate) fn parse(slot: u8) -> Self {
        Self(slot & 0x7F)
    }

    /// # Returns
    ///
    /// The slot hold by the struct
    pub fn slot(&self) -> u8 {
        self.0
    }
}

/// Represents the speed set to a [`SlotArg`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SpeedArg {
    /// Performs a normal stop. Trains may stop smoothly when they receive a message force them to stop.
    Stop,
    /// Performs an immediate stop action. Trains do stop immediately.
    EmergencyStop,
    /// Sets the slots speed to a given value. If you want a slot speed to set to 0
    /// use [`SpeedArg::Stop`] or create your [`SpeedArg`] using [`SpeedArg::new()`].
    ///
    /// The maximum speed is 126. Higher values may create unexpected behaviour.
    Drive(u8),
}

impl SpeedArg {
    /// Creates a new [`SpeedArg`] from the given value.
    /// This means returning [`SpeedArg::Stop`] if the given `spd` is set to 0 and
    /// returning [`SpeedArg::Drive`] with the given `spd` set as speed otherwise.
    ///
    /// # Parameters
    ///
    /// - `spd`: The speed to create the `SpeedArg` for.
    ///          The maximum speed is 126. Higher values may create unexpected behaviour.
    pub fn new(spd: u8) -> Self {
        match spd {
            0x00 => Self::Stop,
            _ => Self::Drive(spd as u8),
        }
    }

    /// Parses the speed from a model railroads send speed.
    ///
    /// # Parameters
    ///
    /// - `spd`: The model railroad messages speed
    pub(crate) fn parse(spd: u8) -> Self {
        match spd {
            0x00 => Self::Stop,
            0x01 => Self::EmergencyStop,
            _ => Self::Drive(spd - 1),
        }
    }

    /// # Returns
    ///
    /// The model railroad interpreted speed of this arg.
    pub(crate) fn spd(&self) -> u8 {
        match *self {
            SpeedArg::Stop => 0x00,
            SpeedArg::EmergencyStop => 0x01,
            SpeedArg::Drive(spd) => (spd + 1) & 0x7F,
        }
    }

    /// # Returns
    ///
    /// A `u8` interpreted value of the given [`SpeedArg`].
    ///
    /// Please note that [`SpeedArg::Stop`] and [`SpeedArg::EmergencyStop`] are both cast to 0
    /// as they both indicates that the slots speed is 0 and only differ in how
    /// immediate this state is reached by the connected device.
    pub fn get_spd(&self) -> u8 {
        match *self {
            SpeedArg::Stop => 0x00,
            SpeedArg::EmergencyStop => 0x00,
            SpeedArg::Drive(spd) => spd,
        }
    }
}

/// Represents the direction and first five function bits of a slot.
///
/// Function bit 0 may control a trains light
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DirfArg(u8);

impl DirfArg {
    /// Creates a new dirf arg with all possible functions set
    ///
    /// # Parameter
    ///
    /// - `dir`: The direction to set (`true` = forwards, `false` = backwards)
    /// - `f0`: Function bit 0 (train light control)
    /// - `f1`: Function bit 1
    /// - `f2`: Function bit 2
    /// - `f3`: Function bit 3
    /// - `f4`: Function bit 4
    pub fn new(dir: bool, f0: bool, f1: bool, f2: bool, f3: bool, f4: bool) -> Self {
        let mut dirf = if dir { 0x20 } else { 0x00 };
        if f0 {
            dirf |= 0x10
        }
        if f1 {
            dirf |= 0x01
        }
        if f2 {
            dirf |= 0x02
        }
        if f3 {
            dirf |= 0x04
        }
        if f4 {
            dirf |= 0x08
        }
        Self(dirf)
    }

    /// Parses the direction from a model railroad message.
    pub(crate) fn parse(dirf: u8) -> Self {
        Self(dirf & 0x3F)
    }

    /// # Returns
    ///
    /// The direction represented by this [`DirfArg`].
    /// `true` means forward, `false` means backwards.
    pub fn dir(&self) -> bool {
        self.0 & 0x20 != 0
    }

    /// # Returns
    ///
    /// The value of the requested f-flag.
    /// As there are only for f-flags are hold by one [`DirfArg`] only values from
    /// 0 to 4 are calculated other inputs may ever return `false`.
    pub fn f(&self, f_num: u8) -> bool {
        if f_num <= 4 {
            self.0 >> (if f_num == 0 { 4 } else { f_num - 1 }) & 1 != 0
        } else {
            false
        }
    }

    /// Sets the direction hold by this arg to the requested value
    ///
    /// # Parameters
    ///
    /// - `value`: The direction to set (`true` = forward, `false` = backward)
    pub fn set_dir(&mut self, value: bool) {
        if value {
            self.0 |= 0x20;
        } else {
            self.0 &= !0x20
        }
    }

    /// Sets the value of the requested f-flag.
    ///
    /// # Parameters
    ///
    /// - `f_num`: The f-flag to set. (Only values in range of 0 to 4 may create an effect).
    ///            Other inputs will be ignored.
    /// - `value`: The value to set the requested flag to.
    pub fn set_f(&mut self, f_num: u8, value: bool) {
        if f_num <= 4 {
            let mask = 1 << if f_num == 0 { 4 } else { f_num - 1 };
            if value {
                self.0 |= mask;
            } else {
                self.0 &= !mask;
            }
        }
    }

    /// Parses this [`DirfArg`] in the corresponding model railroad message format.
    ///
    /// # Returns
    ///
    /// The to this arg corresponding model railroad message value.
    pub(crate) fn dirf(&self) -> u8 {
        self.0
    }
}

/// Overriding the [`Debug`] trait, to show only the corresponding arg states
impl Debug for DirfArg {
    /// Prints the direction and all f-flags from 0 to 4 to the formatter
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "dirf: (dir: {}, f0: {}, f1: {}, f2: {}, f3: {}, f4: {})",
            self.dir(),
            self.f(0),
            self.f(1),
            self.f(2),
            self.f(3),
            self.f(4)
        )
    }
}

/// Holds the track information
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TrkArg {
    /// The tracks power state (`ON`/`OFF`).
    power: bool,
    /// The tracks idle state.
    idle: bool,
    /// `true`: This master implements this protocol capability.
    /// `false`: This master is `DT200`.
    mlok1: bool,
    /// Indicates that masters programming track is busy.
    prog_busy: bool,
}

impl TrkArg {
    /// Creates a new arg representing the tracks status
    ///
    /// # Parameters
    ///
    /// - `power`: The tracks power state (`On`/`OFF`)
    /// - `idle`: The tracks idle state
    /// - `mlok1`: The protocol Version to use. 0 = `DT200`, 1 = this protocol
    /// - `prog_busy`: Busy status for programming track (Slot 124)
    pub fn new(power: bool, idle: bool, mlok1: bool, prog_busy: bool) -> Self {
        TrkArg {
            power,
            idle,
            mlok1,
            prog_busy,
        }
    }

    /// Parses a model railroad messages trk arg to this struct by extracting the required values.
    ///
    /// # Parameters
    ///
    /// - `trk_arg`: The track message to parse
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

    /// # Returns
    ///
    /// The power state of the track.
    pub fn power_on(&self) -> bool {
        self.power
    }

    /// # Returns
    ///
    /// The tracks master idle status.
    pub fn track_idle(&self) -> bool {
        self.idle
    }

    /// # Returns
    ///
    /// The available protocol version by the master.
    ///
    /// - `true` = this protocol is fully supported
    /// - `false` = `DT200`
    pub fn mlok1(&self) -> bool {
        self.mlok1
    }

    /// # Returns
    ///
    /// The programing tracks busy status.
    pub fn prog_busy(&self) -> bool {
        self.prog_busy
    }

    /// Parses this arg to a valid model railroad track message byte.
    ///
    /// # Returns
    ///
    /// The model railroad trk message byte matching this [`TrkArg`].
    pub(crate) fn trk_arg(&self) -> u8 {
        let mut trk_arg = if self.power { 0x01 } else { 0x00 };
        if !self.idle {
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

/// Holds the function flags 5 to 8.
///
/// This function flags may be used for train sound management if available.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SndArg(u8);

impl SndArg {
    /// Creates a new [`SndArg`] with the function flags set.
    ///
    /// # Parameters
    ///
    /// - `f5`: Function flag 5
    /// - `f6`: Function flag 6
    /// - `f7`: Function flag 7
    /// - `f8`: Function flag 8
    pub fn new(f5: bool, f6: bool, f7: bool, f8: bool) -> Self {
        let mut snd = if f5 { 0x01 } else { 0x00 } as u8;
        if f6 {
            snd |= 0x02
        }
        if f7 {
            snd |= 0x04
        }
        if f8 {
            snd |= 0x08
        }
        Self(snd)
    }

    /// Parses a model railroad based function message byte to this arg.
    ///
    /// # Parameters
    ///
    /// - `snd`: A model railroad formatted snd byte
    pub(crate) fn parse(snd: u8) -> Self {
        Self(snd & 0x0F)
    }

    /// # Parameters
    ///
    /// - `f_num`: Which flag to look up
    ///
    /// # Returns
    ///
    /// The value of the `f_num`s function flag. Only values between 5 and 8 are allowed.
    pub fn f(&self, f_num: u8) -> bool {
        if (5..=8).contains(&f_num) {
            self.0 & 1 << (f_num - 5) != 0
        } else {
            false
        }
    }

    /// Sets the value of the `f_num`s function flag to `value`.
    ///
    /// # Parameters
    ///
    /// - `f_num`: The function flags index
    /// - `value`: Which value to set the function bit to
    pub fn set_f(&mut self, f_num: u8, value: bool) {
        if (5..=8).contains(&f_num) {
            let mask = 1 << (f_num - 5);
            if value {
                self.0 |= mask;
            } else {
                self.0 &= !mask;
            }
        }
    }

    /// Parses this [`SndArg`] to a model railroad snd message byte
    pub(crate) fn snd(&self) -> u8 {
        self.0
    }
}

/// Overrides the [`Debug`] trait to show only the corresponding function bits
impl Debug for SndArg {
    /// Prints the f flags from 5 to 8 to the formatter
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "snd: (f5: {}, f6: {}, f7: {}, f8: {})",
            self.f(5),
            self.f(6),
            self.f(7),
            self.f(8)
        )
    }
}

/// Represents the link status of a slot
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Consist {
    /// Slot is linked up and down
    LogicalMid,
    /// Slot is only linked down
    LogicalTop,
    /// Slot is only linked up
    LogicalSubMember,
    /// Slot is not linked
    Free,
}

/// Represents the usage status of a slot
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum State {
    /// Indicates that this slot is in use by some device. The slot holds a loc address and is refreshed.
    ///
    /// If you want to mark your slot as [`State::InUse`] simply perform a `NULL`-Move on this slot. (Move message with equal source and destination)
    InUse,
    /// A loco adr is in the slot but the slot was not refreshed.
    Idle,
    /// This slot holds some loc address and is refreshed.
    Common,
    /// No valid data in this slot, this slot is not refreshed.
    Free,
}

/// Represents the decoders speed control message format used
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DecoderType {
    /// 28 step decoder with advanced DCC allowed
    Dcc28,
    /// 128 step decoder with advanced DVV allowed
    Dcc128,
    /// 28 step mode in 3 byte PKT regular mode
    Regular28,
    /// 28 step mode. Generates trinary packets for mobile address.
    AdrMobile28,
    /// 14 step speed mode (Speed will match values from 0 to 14)
    Step14,
    /// 128 speed mode packets
    Speed128,
}

/// Holds general slot status information.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Stat1Arg {
    /// The slots purge status.
    s_purge: bool,
    /// The slots link status.
    consist: Consist,
    /// The slots usage status.
    state: State,
    /// The decoder type used by the slot.
    decoder_type: DecoderType,
}

impl Stat1Arg {
    /// Creates new slot status information
    ///
    /// # Parameters
    ///
    /// - `s_purge`: The slots purge status
    /// - `consist`: The slots link status
    /// - `state`: The slots usage status
    /// - `decoder_type`: The decoder type used to generate loc messages for this slot
    pub fn new(s_purge: bool, consist: Consist, state: State, decoder_type: DecoderType) -> Self {
        Stat1Arg {
            s_purge,
            consist,
            state,
            decoder_type,
        }
    }

    /// Parses a model railroad formatted `stat1` byte into this arg
    ///
    /// # Parameters
    ///
    /// - `stat1`: The status byte to parse
    pub(crate) fn parse(stat1: u8) -> Self {
        let s_purge = stat1 & 0x80 != 0;

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
            s_purge,
            consist,
            state,
            decoder_type,
        }
    }

    /// # Returns
    ///
    /// The slots purge status
    pub fn s_purge(&self) -> bool {
        self.s_purge
    }

    /// # Returns
    ///
    /// The slots linking state
    pub fn consist(&self) -> Consist {
        self.consist
    }

    /// # Returns
    ///
    /// The usage state of the slot
    pub fn state(&self) -> State {
        self.state
    }

    /// # Returns
    ///
    /// The decoder type to use for this slot
    pub fn decoder_type(&self) -> DecoderType {
        self.decoder_type
    }

    /// Parses this arg to a model railroad defined stat1 message byte
    pub(crate) fn stat1(&self) -> u8 {
        let mut stat1: u8 = if self.s_purge { 0x80 } else { 0x00 };

        stat1 |= match self.consist {
            Consist::LogicalMid => 0x48,
            Consist::LogicalTop => 0x08,
            Consist::LogicalSubMember => 0x40,
            Consist::Free => 0x00,
        };

        stat1 |= match self.state {
            State::InUse => 0x30,
            State::Idle => 0x20,
            State::Common => 0x10,
            State::Free => 0x00,
        };

        stat1 |= match self.decoder_type {
            DecoderType::Dcc28 => 0x04,
            DecoderType::Dcc128 => 0x07,
            DecoderType::Regular28 => 0x00,
            DecoderType::AdrMobile28 => 0x01,
            DecoderType::Step14 => 0x02,
            DecoderType::Speed128 => 0x03,
        };

        stat1
    }
}

/// Extension part for the slot status holding some additional slot information
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Stat2Arg {
    /// If slots ADV consist is suppressed
    has_adv: bool,
    /// ID1/2 is not used for ID
    no_id_usage: bool,
    /// If this ID is no encoded alias
    id_encoded_alias: bool,
}

impl Stat2Arg {
    /// Creates a new status argument
    ///
    /// # Parameters
    ///
    /// - `has_adv`: If this slot has suppressed ADV consist
    /// - `no_id_usage`: If this slots ID1/2 values are not used to represent the ID
    /// - `id_encoded_alias`: If this ID is no encoded alias
    pub fn new(has_adv: bool, no_id_usage: bool, id_encoded_alias: bool) -> Self {
        Stat2Arg {
            has_adv,
            no_id_usage,
            id_encoded_alias,
        }
    }

    /// Parses a received `stat2` byte by the model railroad to this struct
    pub(crate) fn parse(stat2: u8) -> Self {
        let has_adv = stat2 & 0x01 != 0;

        let no_id_usage = stat2 & 0x04 != 0;

        let id_encoded_alias = stat2 & 0x08 != 0;

        Stat2Arg {
            has_adv,
            no_id_usage,
            id_encoded_alias,
        }
    }

    /// # Returns
    ///
    /// If this slot has suppressed advanced control v
    pub fn has_adv(&self) -> bool {
        self.has_adv
    }

    /// # Returns
    ///
    /// If this slot has suppressed adv
    pub fn no_id_usage(&self) -> bool {
        self.no_id_usage
    }

    /// # Returns
    ///
    /// If this messages id is no encoded alias
    pub fn id_encoded_alias(&self) -> bool {
        self.id_encoded_alias
    }

    /// Parses the values hold by this argument to one byte
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LopcArg(u8);

impl LopcArg {
    pub fn new(opc: u8) -> Self {
        LopcArg::parse(opc)
    }

    pub(crate) fn parse(lopc: u8) -> Self {
        Self(lopc & !0x80)
    }

    pub(crate) fn lopc(&self) -> u8 {
        self.0
    }

    pub fn set_opc(&mut self, opc: u8) {
        self.0 = opc & !0x80
    }

    pub fn check_opc(&self, message: &Message) -> bool {
        message.opc() & !0x80 == self.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ack1Arg(u8);

impl Ack1Arg {
    pub fn new(code: u8) -> Self {
        Self(code)
    }

    pub(crate) fn parse(ack1: u8) -> Self {
        Self(ack1)
    }

    pub fn ack1(&self) -> u8 {
        self.0
    }

    pub fn success(&self) -> bool {
        self.0 == 0x7F
    }

    pub fn limited_success(&self) -> bool {
        self.0 != 0x00
    }

    pub fn failed(&self) -> bool {
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

impl Display for Ack1Arg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.failed() {
            write!(f, "ack1: (failed, ack: {})", self.0,)
        } else if self.accepted() {
            write!(f, "ack1: (accepted, ack: {})", self.0,)
        } else if self.accepted_blind() {
            write!(f, "ack1: (accepted_blind, ack: {})", self.0,)
        } else {
            write!(f, "ack1: (success, ack: {})", self.0,)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SourceType {
    Ds54Aux,
    Switch,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SensorLevel {
    High,
    Low,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct InArg {
    address: u16,
    input_source: SourceType,
    sensor_level: SensorLevel,
    control_bit: bool,
}

impl InArg {
    pub fn new(
        address: u16,
        input_source: SourceType,
        sensor_level: SensorLevel,
        control_bit: bool,
    ) -> Self {
        InArg {
            address: address & 0x07FF,
            input_source,
            sensor_level,
            control_bit,
        }
    }

    pub fn parse(in1: u8, in2: u8) -> Self {
        let mut address = in1 as u16;
        address |= (in2 as u16 & 0x0F) << 7;

        let input_source = if in2 & 0x20 == 0 {
            SourceType::Ds54Aux
        } else {
            SourceType::Switch
        };

        let sensor_level = if (in2 & 0x10) != 0 {
            SensorLevel::High
        } else {
            SensorLevel::Low
        };
        let control_bit = (in2 & 0x40) != 0;
        Self {
            address,
            input_source,
            sensor_level,
            control_bit,
        }
    }

    pub fn address(&self) -> u16 {
        self.address
    }

    pub fn address_ds54(&self) -> u16 {
        (self.address << 1)
            | if self.input_source == SourceType::Switch {
                1
            } else {
                0
            }
    }

    pub fn input_source(&self) -> SourceType {
        self.input_source
    }

    pub fn sensor_level(&self) -> SensorLevel {
        self.sensor_level
    }

    pub fn control_bit(&self) -> bool {
        self.control_bit
    }

    pub fn set_address(&mut self, address: u16) {
        debug_assert_eq!(
            address & 0x07FF,
            0,
            "address must only use the 11 least significant bits"
        );
        self.address = address;
    }

    pub fn set_address_ds54(&mut self, address_ds54: u16) {
        debug_assert_eq!(
            self.address & 0x0FFF,
            0,
            "address must only use the 12 least significant bits"
        );
        self.input_source = if address_ds54 & 1 == 0 {
            SourceType::Ds54Aux
        } else {
            SourceType::Switch
        };
        self.set_address(address_ds54 >> 1);
    }

    pub fn set_input_source(&mut self, input_source: SourceType) {
        self.input_source = input_source;
    }

    pub fn set_sensor_level(&mut self, sensor_level: SensorLevel) {
        self.sensor_level = sensor_level;
    }

    pub fn set_control_bit(&mut self, control_bit: bool) {
        self.control_bit = control_bit;
    }

    pub fn in1(&self) -> u8 {
        self.address as u8 & 0x7F
    }

    pub fn in2(&self) -> u8 {
        let mut in2 = ((self.address >> 7) as u8) & 0x0F;
        in2 |= match self.input_source {
            SourceType::Ds54Aux => 0x00,
            SourceType::Switch => 0x20,
        };
        in2 |= match self.sensor_level {
            SensorLevel::High => 0x10,
            SensorLevel::Low => 0x00,
        };
        if self.control_bit {
            in2 |= 0x40;
        }
        in2
    }
}

#[derive(Copy, Clone, Eq)]
pub struct SnArg {
    address: u16,
    format: bool,
    c: bool,
    t: bool,
    input_source: SourceType,
    sensor_level: SensorLevel,
}

impl SnArg {
    pub fn new_c_t(address: u16, c: bool, t: bool) -> Self {
        let input_source = if c {
            SourceType::Switch
        } else {
            SourceType::Ds54Aux
        };

        let sensor_level = if t {
            SensorLevel::High
        } else {
            SensorLevel::Low
        };

        SnArg {
            address: address & 0x07FF,
            format: false,
            c,
            t,
            input_source,
            sensor_level,
        }
    }

    pub fn new_f(address: u16, input_source: SourceType, sensor_level: SensorLevel) -> Self {
        SnArg {
            address: address & 0x07FF,
            format: true,
            c: false,
            t: false,
            input_source,
            sensor_level,
        }
    }

    pub fn parse(sn1: u8, sn2: u8) -> Self {
        let mut address = sn1 as u16;
        address |= (sn2 as u16 & 0x0F) << 7;

        let format = sn2 & 0x40 == 0x40;

        let t = sn2 & 0x10 == 0x10;
        let c = sn2 & 0x20 == 0x20;

        let input_source = if c {
            SourceType::Switch
        } else {
            SourceType::Ds54Aux
        };

        let sensor_level = if t {
            SensorLevel::High
        } else {
            SensorLevel::Low
        };

        SnArg {
            address,
            format,
            c,
            t,
            input_source,
            sensor_level,
        }
    }

    pub fn address(&self) -> u16 {
        self.address
    }

    pub fn format(&self) -> bool {
        self.format
    }

    pub fn c_u8(&self) -> Result<u8, String> {
        if !self.format {
            Ok(self.c as u8)
        } else {
            Err("Wrong sn format".to_owned())
        }
    }

    pub fn t_u8(&self) -> Result<u8, String> {
        if !self.format {
            Ok(self.t as u8)
        } else {
            Err("Wrong sn format".to_owned())
        }
    }

    pub fn input_source(&self) -> Result<SourceType, String> {
        if self.format {
            Ok(self.input_source)
        } else {
            Err("Wrong sn format".to_owned())
        }
    }

    pub fn sensor_level(&self) -> Result<SensorLevel, String> {
        if self.format {
            Ok(self.sensor_level)
        } else {
            Err("Wrong sn format".to_owned())
        }
    }

    pub fn set_address(&mut self, address: u16) {
        self.address = address & 0x07FF;
    }

    pub fn set_format(&mut self, format: bool) {
        self.format = format;
    }

    pub fn set_c(&mut self, c: bool) -> Result<(), String> {
        if self.format {
            return Err("Wrong sn format".to_owned());
        }

        self.c = c;
        Ok(())
    }

    pub fn set_t(&mut self, t: bool) -> Result<(), String> {
        if self.format {
            return Err("Wrong sn format".to_owned());
        }

        self.t = t;
        Ok(())
    }

    pub fn set_input_source(&mut self, input_source: SourceType) -> Result<(), String> {
        if !self.format {
            return Err("Wrong sn format".to_owned());
        }

        self.input_source = input_source;
        Ok(())
    }

    pub fn set_sensor_level(&mut self, sensor_level: SensorLevel) -> Result<(), String> {
        if !self.format {
            return Err("Wrong sn format".to_owned());
        }

        self.sensor_level = sensor_level;
        Ok(())
    }

    pub fn sn1(&self) -> u8 {
        (self.address as u8) & 0x7F
    }

    pub fn sn2(&self) -> u8 {
        let mut sn2 = (self.address >> 7) as u8 & 0x0F;
        if self.format {
            sn2 |=
                0x40 | match self.input_source {
                    SourceType::Ds54Aux => 0x00,
                    SourceType::Switch => 0x20,
                } | match self.sensor_level {
                    SensorLevel::High => 0x10,
                    SensorLevel::Low => 0x00,
                }
        } else {
            if self.c {
                sn2 |= 0x20;
            }
            if self.t {
                sn2 |= 0x10;
            }
        }

        sn2
    }
}

impl PartialEq for SnArg {
    fn eq(&self, other: &Self) -> bool {
        if !(self.format == other.format && self.address == other.address) {
            return false;
        }

        if self.format {
            self.input_source == other.input_source && self.sensor_level == other.sensor_level
        } else {
            self.t == other.t && self.c == other.c
        }
    }
}

impl Debug for SnArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.format {
            write!(
                f,
                "sn: (address: {}, input_source: {:?}, sensor_level: {:?})",
                self.address, self.input_source, self.sensor_level,
            )
        } else {
            write!(
                f,
                "sn: (address: {}, c: {:?}, t: {:?})",
                self.address, self.c, self.t,
            )
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct IdArg(u16);

impl IdArg {
    pub fn new(id: u16) -> Self {
        IdArg(id & 0x3FFF)
    }

    pub fn parse(id1: u8, id2: u8) -> Self {
        IdArg((((id2 & 0x7F) as u16) << 7) | ((id1 & 0x7F) as u16))
    }

    pub fn id(&self) -> u16 {
        self.0
    }

    pub fn id1(&self) -> u8 {
        self.0 as u8 & 0x7F
    }

    pub fn id2(&self) -> u8 {
        (self.0 >> 7) as u8 & 0x7F
    }

    pub fn set_id(&mut self, id: u16) {
        self.0 = id & 0x3FFF
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MultiSenseArg {
    m_type: u8,
    present: bool,
    board_address: u8,
    zone: u8,
}

impl MultiSenseArg {
    pub fn new(m_type: u8, present: bool, board_address: u8, zone: u8) -> Self {
        Self {
            m_type: m_type & 0x07,
            present,
            board_address,
            zone,
        }
    }

    pub fn parse(m_high: u8, zas: u8) -> Self {
        let m_type = (0xE0 & m_high) >> 5;
        let present = 0x10 & m_high == 0x10;
        let board_address = ((0x0F & m_high) << 4) | ((zas & 0xF0) >> 4);
        let zone = 0x0F & zas;

        MultiSenseArg {
            m_type,
            present,
            board_address,
            zone,
        }
    }

    pub fn m_type(&self) -> u8 {
        self.m_type
    }

    pub fn present(&self) -> bool {
        self.present
    }

    pub fn board_address(&self) -> u8 {
        self.board_address
    }

    pub fn zone(&self) -> u8 {
        self.zone
    }

    pub fn zas(&self) -> u8 {
        self.zone | ((self.board_address & 0x0F) << 4)
    }

    pub fn m_high(&self) -> u8 {
        ((self.board_address & 0xF0) >> 4)
            | ((self.m_type & 0x07) << 5)
            | if self.present { 0x10 } else { 0x00 }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FunctionArg(u8, u8);

impl FunctionArg {
    pub fn new(group: u8) -> Self {
        FunctionArg(group, 0)
    }

    pub fn c_new(group: u16) -> Self {
        FunctionArg(group as u8, 0)
    }

    pub fn c_parse(group: u16, function: u16) -> Self {
        FunctionArg(group as u8, function as u8)
    }

    pub fn parse(group: u8, function: u8) -> Self {
        FunctionArg(group, function)
    }

    pub fn f(&self, f_num: u8) -> bool {
        if f_num > 8 && f_num < 12 && self.0 == 0x07 {
            (self.1 >> (f_num - 5)) & 1 != 0
        } else if (f_num == 12 || f_num == 20 || f_num == 28) && self.0 == 0x05 {
            (self.1
                >> (if f_num == 12 {
                    4
                } else if f_num == 20 {
                    5
                } else {
                    6
                }))
                & 1
                != 0
        } else if f_num > 12 && f_num < 20 && self.0 == 0x08 {
            (self.1 >> (f_num - 13)) & 1 != 0
        } else if f_num > 20 && f_num < 28 && self.0 == 0x09 {
            (self.1 >> (f_num - 21)) & 1 != 0
        } else {
            false
        }
    }

    pub fn set_f(&mut self, f_num: u8, value: bool) {
        let mask = if f_num > 8 && f_num < 12 && self.0 == 0x07 {
            1 << (f_num - 5)
        } else if (f_num == 12 || f_num == 20 || f_num == 28) && self.0 == 0x05 {
            1 << (if f_num == 12 {
                0
            } else if f_num == 20 {
                1
            } else {
                2
            })
        } else if f_num > 12 && f_num < 20 && self.0 == 0x08 {
            1 << (f_num - 13)
        } else if f_num > 20 && f_num < 28 && self.0 == 0x09 {
            1 << (f_num - 21)
        } else {
            0x00
        };

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
            "function_arg: (f9: {}, f10: {}, f11: {}, f12: {}, f13: {}, f14: {}, f15: {}, f16: {}, f17: {}, f18: {}, f19: {}, f20: {}, f21: {}, f22: {}, f23: {}, f24: {}, f25: {}, f26: {}, f27: {}, f28: {})",
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Pcmd {
    write: bool,
    byte_mode: bool,
    ops_mode: bool,
    ty0: bool, // Programing type select bit
    ty1: bool, // prog type select bit
}

impl Pcmd {
    pub fn new(write: bool, byte_mode: bool, ops_mode: bool, ty0: bool, ty1: bool) -> Self {
        Pcmd {
            write,
            byte_mode,
            ops_mode,
            ty0,
            ty1,
        }
    }

    pub fn parse(pcmd: u8) -> Self {
        let write = pcmd & 0x20 == 0x20;
        let byte_mode = pcmd & 0x40 == 0x40;
        let ops_mode = pcmd & 0x02 == 0x02;
        let ty0 = pcmd & 0x80 == 0x80;
        let ty1 = pcmd & 0x01 == 0x01;

        Pcmd {
            write,
            byte_mode,
            ops_mode,
            ty0,
            ty1,
        }
    }

    pub fn write(&self) -> bool {
        self.write
    }

    pub fn byte_mode(&self) -> bool {
        self.byte_mode
    }

    pub fn ops_mode(&self) -> bool {
        self.ops_mode
    }

    pub fn ty0(&self) -> bool {
        self.ty0
    }

    pub fn ty1(&self) -> bool {
        self.ty1
    }

    pub fn set_write(&mut self, write: bool) {
        self.write = write
    }

    pub fn set_byte_mode(&mut self, byte_mode: bool) {
        self.byte_mode = byte_mode
    }

    pub fn set_ops_mode(&mut self, ops_mode: bool) {
        self.ops_mode = ops_mode
    }

    pub fn set_ty0(&mut self, ty0: bool) {
        self.ty0 = ty0
    }

    pub fn set_ty1(&mut self, ty1: bool) {
        self.ty1 = ty1
    }

    pub fn pcmd(&self) -> u8 {
        let mut pcmd = if self.write { 0x20 } else { 0x00 };
        if self.byte_mode {
            pcmd |= 0x40;
        }
        if self.ops_mode {
            pcmd |= 0x02;
        }
        if self.ty0 {
            pcmd |= 0x80;
        }
        if self.ty1 {
            pcmd |= 0x01;
        }
        pcmd
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PStat {
    user_aborted: bool,
    no_read_ack: bool,
    no_write_ack: bool,
    programming_track_empty: bool,
}

impl PStat {
    pub fn new(
        user_aborted: bool,
        no_read_ack: bool,
        no_write_ack: bool,
        programming_track_empty: bool,
    ) -> Self {
        PStat {
            user_aborted,
            no_read_ack,
            no_write_ack,
            programming_track_empty,
        }
    }

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

    pub fn user_aborted(&self) -> bool {
        self.user_aborted
    }

    pub fn no_read_ack(&self) -> bool {
        self.no_read_ack
    }

    pub fn no_write_ack(&self) -> bool {
        self.no_write_ack
    }

    pub fn programming_track_empty(&self) -> bool {
        self.programming_track_empty
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

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct CvDataArg(u16, u8);

impl CvDataArg {
    pub fn new() -> CvDataArg {
        CvDataArg(0, 0)
    }

    pub(crate) fn parse(cvh: u8, cvl: u8, data7: u8) -> Self {
        let mut cv_arg = cvl as u16;
        let data = ((cvh & 0x02) << 6) | data7;

        let mut high_cv_arg = cvh & 0x01;
        high_cv_arg |= (cvh & 0x30) >> 3;

        cv_arg |= (high_cv_arg as u16) << 7;

        CvDataArg(cv_arg, data)
    }

    pub fn data(&self, d_num: u8) -> bool {
        (self.1 >> d_num) & 0x01 != 0
    }

    pub fn cv(&self, cv_num: u8) -> bool {
        self.0 >> cv_num & 1 != 0
    }

    pub fn set_data(&mut self, d_num: u8, value: bool) -> &mut Self {
        let mask = 1 << d_num;

        if value {
            self.1 |= mask;
        } else {
            self.1 &= !mask;
        }

        self
    }

    pub fn set_cv(&mut self, cv_num: u8, value: bool) -> &mut Self {
        let mask = (1 << cv_num) & 0x03FF;

        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }

        self
    }

    pub fn cvh(&self) -> u8 {
        let mut cvh = (self.0 >> 7) as u8;
        let high_cv = cvh & 0x06 << 3;
        cvh &= 0x01;
        cvh |= high_cv;
        if self.data(7) {
            cvh |= 0x02;
        }
        cvh
    }

    pub fn cvl(&self) -> u8 {
        self.0 as u8 & 0x7F
    }

    pub fn data7(&self) -> u8 {
        self.1 & 0x7F
    }
}

impl Debug for CvDataArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cv_data_arg: (data: (d0: {}, d1: {}, d2: {}, d3: {}, d4: {}, d5: {}, d6: {}, d7: {}), cv:(cv0: {}, cv1: {}, cv2: {}, cv3: {}, cv4: {}, cv5: {}, cv6: {}, cv7: {}, cv8: {}, cv9: {}))",
            self.data(0),
            self.data(1),
            self.data(2),
            self.data(3),
            self.data(4),
            self.data(5),
            self.data(6),
            self.data(7),
            self.cv(0),
            self.cv(1),
            self.cv(2),
            self.cv(3),
            self.cv(4),
            self.cv(5),
            self.cv(6),
            self.cv(7),
            self.cv(8),
            self.cv(9)
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FastClock {
    clk_rate: u8,
    frac_minsl: u8,
    frac_minsh: u8,
    mins: u8,
    hours: u8,
    days: u8,
    clk_cntrl: u8,
}

impl FastClock {
    pub fn new(
        clk_rate: u8,
        frac_minsl: u8,
        frac_minsh: u8,
        mins: u8,
        hours: u8,
        days: u8,
        clk_cntrl: u8
    ) -> Self {
        FastClock::parse(
            clk_rate,
            frac_minsl,
            frac_minsh,
            mins,
            hours,
            days,
            clk_cntrl
        )
    }

    pub(crate) fn parse(
        clk_rate: u8,
        frac_minsl: u8,
        frac_minsh: u8,
        mins: u8,
        hours: u8,
        days: u8,
        clk_cntrl: u8,
    ) -> Self {
        FastClock {
            clk_rate: clk_rate & 0x7F,
            frac_minsl,
            frac_minsh,
            mins,
            hours,
            days,
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

    fn mins(&self) -> u8 {
        self.mins
    }

    fn hours(&self) -> u8 {
        self.hours
    }

    fn days(&self) -> u8 {
        self.days
    }

    pub fn clk_cntrl(&self) -> u8 {
        self.clk_cntrl
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ImFunctionType {
    F9to12,
    F13to20,
    F21to28
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ImAddress {
    Short(u8),
    Long(u16)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ImArg {
    dhi: u8,
    address: ImAddress,
    function_type: ImFunctionType,
    function_bits: u8,
    im5: u8,
}

impl ImArg {
    pub fn new(
        dhi: u8,
        address: ImAddress,
        function_type: ImFunctionType,
        im5: u8,
    ) -> Self {
        ImArg {
            dhi,
            address,
            function_type,
            function_bits: 0x00,
            im5,
        }
    }

    pub(crate) fn parse(
        _: u8,
        reps: u8,
        dhi: u8,
        im1: u8,
        im2: u8,
        im3: u8,
        im4: u8,
        im5: u8,
    ) -> ImArg {
        if reps == 0x44 || (reps == 0x34 && (im3 & 0x20) == 0x20) {
            let address = ImAddress::Long(((im2 as u16) << 8) | im1 as u16);

            let function_type = if im3 == 0x5E {
                ImFunctionType::F13to20
            } else if im3 == 0x5F {
                ImFunctionType::F21to28
            } else {
                ImFunctionType::F9to12
            };
            let mut function_bits = match function_type {
                ImFunctionType::F21to28 => im4,
                ImFunctionType::F13to20 => im4,
                ImFunctionType::F9to12 => im3 & !0x20,
            };

            function_bits &= 0x7F;

            Self {
                dhi,
                address,
                function_type,
                function_bits,
                im5,
            }
        } else {
            let address = ImAddress::Short(im1);

            let function_type = if im2 == 0x5E {
                ImFunctionType::F13to20
            } else if im2 == 0x5F {
                ImFunctionType::F21to28
            } else {
                ImFunctionType::F9to12
            };
            let mut function_bits = match function_type {
                ImFunctionType::F13to20 => im3,
                ImFunctionType::F21to28 => im3,
                ImFunctionType::F9to12 => im2 & !0x2F,
            };

            function_bits &= 0x7F;

            Self {
                dhi,
                address,
                function_type,
                function_bits,
                im5,
            }
        }
    }

    pub fn check_byte(&self) -> u8 {
        0x7F
    }

    pub fn reps(&self) -> u8 {
        match self.address {
            ImAddress::Short(_) => match self.function_type {
                ImFunctionType::F9to12 => 0x24,
                ImFunctionType::F13to20 => 0x34,
                ImFunctionType::F21to28 => 0x34,
            }
            ImAddress::Long(_) => match self.function_type {
                ImFunctionType::F9to12 => 0x34,
                ImFunctionType::F13to20 => 0x44,
                ImFunctionType::F21to28 => 0x44,
            }
        }
    }

    pub fn dhi(&self) -> u8 {
        self.dhi
    }

    pub fn address(&self) -> ImAddress {
        self.address
    }

    pub fn function_type(&self) -> ImFunctionType {
        self.function_type
    }

    pub fn function_bits(&self) -> u8 {
        self.function_bits
    }

    pub fn f(&self, f_num: u8) -> bool {
        let dist = match self.function_type {
            ImFunctionType::F13to20 => 21,
            ImFunctionType::F21to28 => 13,
            ImFunctionType::F9to12 => 9,
        };

        (self.function_bits >> (f_num - dist)) & 0x01 == 0x01
    }

    pub fn set_f(&mut self, f_num: u8, f: bool) {
        let dist = match self.function_type {
            ImFunctionType::F13to20 => 21,
            ImFunctionType::F21to28 => 13,
            ImFunctionType::F9to12 => 9,
        };

        let mask = 0x01 << (f_num - dist);

        if f {
            self.function_bits |= mask;
        } else {
            self.function_bits &= !mask;
        }
    }

    pub fn im1(&self) -> u8 {
        match self.address {
            ImAddress::Short(adr) => adr,
            ImAddress::Long(adr) => adr as u8,
        }
    }

    pub fn im2(&self) -> u8 {
        match self.address {
            ImAddress::Short(_) => {
                match self.function_type {
                    ImFunctionType::F9to12 => (self.function_bits & 0x7F) | 0x20,
                    ImFunctionType::F13to20 => 0x5E,
                    ImFunctionType::F21to28 => 0x5F,
                }
            },
            ImAddress::Long(adr) => {
                (adr >> 8) as u8
            }
        }
    }

    pub fn im3(&self) -> u8 {
        match self.address {
            ImAddress::Short(_) => {
                if self.function_type == ImFunctionType::F9to12 {
                    0x00
                } else {
                    self.function_bits
                }
            },
            ImAddress::Long(_) => {
                match self.function_type {
                    ImFunctionType::F9to12 => (self.function_bits & 0x7F) | 0x20,
                    ImFunctionType::F13to20 => 0x5E,
                    ImFunctionType::F21to28 => 0x5F,
                }
            }
        }
    }

    pub fn im4(&self) -> u8 {
        if self.reps() == 0x34 && self.function_type != ImFunctionType::F9to12 {
            return self.function_bits;
        }
        0x00
    }

    pub fn im5(&self) -> u8 {
        self.im5
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WrSlDataTime(FastClock, TrkArg, IdArg);

impl WrSlDataTime {
    pub fn new(fast_clock: FastClock, trk_arg: TrkArg, id_arg: IdArg) -> Self {
        WrSlDataTime(fast_clock, trk_arg, id_arg)
    }

    pub fn parse(
        clk_rate: u8,
        frac_minsl: u8,
        frac_minsh: u8,
        mins: u8,
        trk: u8,
        hours: u8,
        days: u8,
        clk_cntr: u8,
        id1: u8,
        id2: u8,
    ) -> Self {
        WrSlDataTime(
            FastClock::parse(
                clk_rate, frac_minsl, frac_minsh, mins, hours, days, clk_cntr,
            ),
            TrkArg::parse(trk),
            IdArg::parse(id1, id2),
        )
    }

    pub fn fast_clock(&self) -> FastClock {
        self.0
    }

    pub fn trk_arg(&self) -> TrkArg {
        self.1
    }

    pub fn id_arg(&self) -> IdArg {
        self.2
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WrSlDataPt(Pcmd, AddressArg, TrkArg, CvDataArg);

impl WrSlDataPt {
    pub fn new(pcmd: Pcmd, opsa: AddressArg, trk_arg: TrkArg, cv_data_arg: CvDataArg) -> Self {
        WrSlDataPt(pcmd, opsa, trk_arg, cv_data_arg)
    }

    pub fn parse(
        pcmd: u8,
        _arg3: u8,
        hopsa: u8,
        lopsa: u8,
        trk: u8,
        cvh: u8,
        cvl: u8,
        data7: u8,
        _arg10: u8,
        _arg11: u8,
    ) -> Self {
        WrSlDataPt(
            Pcmd::parse(pcmd),
            AddressArg::parse(hopsa, lopsa),
            TrkArg::parse(trk),
            CvDataArg::parse(cvh, cvl, data7),
        )
    }

    pub fn pcmd(&self) -> Pcmd {
        self.0
    }

    pub fn opsa(&self) -> AddressArg {
        self.1
    }

    pub fn trk_arg(&self) -> TrkArg {
        self.2
    }

    pub fn cv_data_arg(&self) -> CvDataArg {
        self.3
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WrSlDataGeneral(
    SlotArg,
    Stat1Arg,
    Stat2Arg,
    AddressArg,
    SpeedArg,
    DirfArg,
    TrkArg,
    SndArg,
    IdArg,
);

impl WrSlDataGeneral {
    pub fn new(
        slot_arg: SlotArg,
        stat1_arg: Stat1Arg,
        stat2_arg: Stat2Arg,
        address_arg: AddressArg,
        speed_arg: SpeedArg,
        dirf_arg: DirfArg,
        trk_arg: TrkArg,
        snd_arg: SndArg,
        id_arg: IdArg,
    ) -> Self {
        WrSlDataGeneral(
            slot_arg,
            stat1_arg,
            stat2_arg,
            address_arg,
            speed_arg,
            dirf_arg,
            trk_arg,
            snd_arg,
            id_arg,
        )
    }

    pub fn parse(
        slot: u8,
        stat1: u8,
        adr: u8,
        spd: u8,
        dirf: u8,
        trk: u8,
        stat2: u8,
        adr2: u8,
        snd: u8,
        id1: u8,
        id2: u8,
    ) -> Self {
        WrSlDataGeneral(
            SlotArg::parse(slot),
            Stat1Arg::parse(stat1),
            Stat2Arg::parse(stat2),
            AddressArg::parse(adr2, adr),
            SpeedArg::parse(spd),
            DirfArg::parse(dirf),
            TrkArg::parse(trk),
            SndArg::parse(snd),
            IdArg::parse(id1, id2),
        )
    }

    pub fn slot_arg(&self) -> SlotArg {
        self.0
    }

    pub fn stat1_arg(&self) -> Stat1Arg {
        self.1
    }

    pub fn stat2_arg(&self) -> Stat2Arg {
        self.2
    }

    pub fn address_arg(&self) -> AddressArg {
        self.3
    }

    pub fn speed_arg(&self) -> SpeedArg {
        self.4
    }

    pub fn dirf_arg(&self) -> DirfArg {
        self.5
    }

    pub fn trk_arg(&self) -> TrkArg {
        self.6
    }

    pub fn snd_arg(&self) -> SndArg {
        self.7
    }

    pub fn id_arg(&self) -> IdArg {
        self.8
    }

    pub fn set_snd_arg(&mut self, snd_arg: SndArg) {
        self.7 = snd_arg;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WrSlDataStructure {
    DataTime(WrSlDataTime),
    /// Used for train programming
    DataPt(WrSlDataPt),
    DataGeneral(WrSlDataGeneral),
}

impl WrSlDataStructure {

    pub(crate) fn parse(
        arg1: u8,
        arg2: u8,
        arg3: u8,
        arg4: u8,
        arg5: u8,
        arg6: u8,
        arg7: u8,
        arg8: u8,
        arg9: u8,
        arg10: u8,
        arg11: u8,
    ) -> Self {
        if arg1 == 0x7C {
            WrSlDataStructure::DataPt(
                WrSlDataPt::parse(arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11)
            )
        } else if arg1 == 0x7B {
            WrSlDataStructure::DataTime(
                WrSlDataTime::parse(arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11)
            )
        } else {
            WrSlDataStructure::DataGeneral(
                WrSlDataGeneral::parse(
                    arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
                )
            )
        }
    }

    pub fn slot_type(&self) -> u8 {
        match self {
            WrSlDataStructure::DataPt(..) => 0x7C,
            WrSlDataStructure::DataTime(..) => 0x7B,
            WrSlDataStructure::DataGeneral(data) => data.slot_arg().slot()
        }
    }

    pub(crate) fn to_message(self) -> Vec<u8> {
        match self {
            WrSlDataStructure::DataPt(pt_slot) => {
                vec![
                    0xEF,
                    0x0E,
                    0x7C,
                    pt_slot.0.pcmd(),
                    0x00,
                    pt_slot.1.adr2(),
                    pt_slot.1.adr1(),
                    pt_slot.2.trk_arg(),
                    pt_slot.3.cvh(),
                    pt_slot.3.cvl(),
                    pt_slot.3.data7(),
                    0x00,
                    0x00,
                ]
            },
            WrSlDataStructure::DataTime(time_slot) => {
                vec![
                    0xEF,
                    0x0E,
                    0x7B,
                    time_slot.0.clk_rate(),
                    time_slot.0.frac_minsl(),
                    time_slot.0.frac_minsh(),
                    time_slot.0.mins(),
                    time_slot.1.trk_arg(),
                    time_slot.0.hours(),
                    time_slot.0.days(),
                    time_slot.0.clk_cntrl(),
                    time_slot.2.id1(),
                    time_slot.2.id2(),
                ]
            },
            WrSlDataStructure::DataGeneral(general_slot) => {
                vec![
                    0xEF,
                    0x0E,
                    general_slot.0.slot(),
                    general_slot.1.stat1(),
                    general_slot.3.adr1(),
                    general_slot.4.spd(),
                    general_slot.5.dirf(),
                    general_slot.6.trk_arg(),
                    general_slot.2.stat2(),
                    general_slot.3.adr2(),
                    general_slot.7.snd(),
                    general_slot.8.id1(),
                    general_slot.8.id2(),
                ]
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LissyIrReport {
    arg1: u8,
    dir: bool,
    unit: u16,
    address: u16,
}

impl LissyIrReport {
    pub fn new(dir: bool, unit: u16, address: u16) -> Self {
        LissyIrReport {
            arg1: 0x00,
            dir,
            unit,
            address,
        }
    }

    fn parse(
        arg1: u8,
        high_unit: u8,
        low_unit: u8,
        high_adr: u8,
        low_adr: u8,
    ) -> Self {

        let dir = high_unit & 0x40 == 0x40;
        let unit = (((high_unit & 0x3F) as u16) << 7) | (low_unit as u16);
        let address = (((high_adr & 0x7F) as u16) << 7) | (low_adr as u16);

        LissyIrReport {
            arg1,
            dir,
            unit,
            address,
        }
    }

    pub fn to_message(&self) -> Vec<u8> {
        let mut high_unit = ((self.unit >> 7) as u8) & 0x3F;
        if self.dir {
            high_unit |= 0x40;
        }
        let low_unit = self.unit as u8 & 0x7F;
        let high_adr = ((self.address >> 7) as u8) & 0x7F;
        let low_adr = self.address as u8 & 0x7F;
        vec![
            0xE4, 0x08, self.arg1, high_unit, low_unit, high_adr, low_adr,
        ]
    }

    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    pub fn dir(&self) -> bool {
        self.dir
    }

    pub fn unit(&self) -> u16 {
        self.unit
    }

    pub fn address(&self) -> u16 {
        self.address
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RFID5Report {
    arg1: u8,
    address: u16,
    rfid0: u8,
    rfid1: u8,
    rfid2: u8,
    rfid3: u8,
    rfid4: u8,
    rfid_hi: u8,
}

impl RFID5Report {
    pub fn new(
        address: u16,
        rfid0: u8,
        rfid1: u8,
        rfid2: u8,
        rfid3: u8,
        rfid4: u8,
        rfid_hi: u8,
    ) -> Self {
        RFID5Report {
            arg1: 0x41,
            address,
            rfid0,
            rfid1,
            rfid2,
            rfid3,
            rfid4,
            rfid_hi,
        }
    }

    fn parse(
        arg1: u8,
        high_adr: u8,
        low_adr: u8,
        rfid0: u8,
        rfid1: u8,
        rfid2: u8,
        rfid3: u8,
        rfid4: u8,
        rfid_hi: u8,
    ) -> Self {
        let address = (((high_adr & 0x7F) as u16) << 7) | (low_adr as u16);
        RFID5Report {
            arg1,
            address,
            rfid0,
            rfid1,
            rfid2,
            rfid3,
            rfid4,
            rfid_hi,
        }
    }

    pub fn to_message(&self) -> Vec<u8> {
        let high_adr = ((self.address >> 7) as u8) & 0x7F;
        let low_adr = (self.address as u8) & 0x7F;
        vec![
            0xE4,
            0x0C,
            self.arg1,
            high_adr,
            low_adr,
            self.rfid0,
            self.rfid1,
            self.rfid2,
            self.rfid3,
            self.rfid4,
            self.rfid_hi,
        ]
    }

    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    pub fn address(&self) -> u16 {
        self.address
    }

    pub fn rfid0(&self) -> u8 {
        self.rfid0
    }

    pub fn rfid1(&self) -> u8 {
        self.rfid1
    }

    pub fn rfid2(&self) -> u8 {
        self.rfid2
    }

    pub fn rfid3(&self) -> u8 {
        self.rfid3
    }

    pub fn rfid4(&self) -> u8 {
        self.rfid4
    }

    pub fn rfid_hi(&self) -> u8 {
        self.rfid_hi
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RFID7Report {
    arg1: u8,
    address: u16,
    rfid0: u8,
    rfid1: u8,
    rfid2: u8,
    rfid3: u8,
    rfid4: u8,
    rfid5: u8,
    rfid6: u8,
    rfid_hi: u8,
}

impl RFID7Report {
    pub fn new(
        address: u16,
        rfid0: u8,
        rfid1: u8,
        rfid2: u8,
        rfid3: u8,
        rfid4: u8,
        rfid5: u8,
        rfid6: u8,
        rfid_hi: u8,
    ) -> Self {
        RFID7Report {
            arg1: 0x41,
            address,
            rfid0,
            rfid1,
            rfid2,
            rfid3,
            rfid4,
            rfid5,
            rfid6,
            rfid_hi,
        }
    }

    fn parse(
        arg1: u8,
        high_adr: u8,
        low_adr: u8,
        rfid0: u8,
        rfid1: u8,
        rfid2: u8,
        rfid3: u8,
        rfid4: u8,
        rfid5: u8,
        rfid6: u8,
        rfid_hi: u8,
    ) -> Self {
        let address = (((high_adr & 0x7F) as u16) << 7) | (low_adr as u16);
        RFID7Report {
            arg1,
            address,
            rfid0,
            rfid1,
            rfid2,
            rfid3,
            rfid4,
            rfid5,
            rfid6,
            rfid_hi,
        }
    }

    pub fn to_message(&self) -> Vec<u8> {
        let high_adr = ((self.address >> 7) as u8) & 0x7F;
        let low_adr = (self.address as u8) & 0x7F;
        vec![
            0xE4,
            0x0E,
            self.arg1,
            high_adr,
            low_adr,
            self.rfid0,
            self.rfid1,
            self.rfid2,
            self.rfid3,
            self.rfid4,
            self.rfid5,
            self.rfid6,
            self.rfid_hi,
        ]
    }

    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    pub fn address(&self) -> u16 {
        self.address
    }

    pub fn rfid0(&self) -> u8 {
        self.rfid0
    }

    pub fn rfid1(&self) -> u8 {
        self.rfid1
    }

    pub fn rfid2(&self) -> u8 {
        self.rfid2
    }

    pub fn rfid3(&self) -> u8 {
        self.rfid3
    }

    pub fn rfid4(&self) -> u8 {
        self.rfid4
    }

    pub fn rfid5(&self) -> u8 {
        self.rfid5
    }

    pub fn rfid6(&self) -> u8 {
        self.rfid6
    }

    pub fn rfid_hi(&self) -> u8 {
        self.rfid_hi
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WheelcntReport {
    arg1: u8,
    unit: u16,
    direction: bool,
    count: u16,
}

impl WheelcntReport {
    pub fn new(unit: u16, direction: bool, count: u16) -> Self {
        WheelcntReport {
            arg1: 0x40,
            unit,
            direction,
            count,
        }
    }

    fn parse(
        arg1: u8,
        high_unit: u8,
        low_unit: u8,
        high_count: u8,
        low_count: u8,
    ) -> Self {
        let count = ((high_count as u16) << 7) | (low_count as u16);
        let direction = high_unit & 0x40 == 0x40;
        let unit = (((high_unit & 0x3F) as u16) << 7) | (low_unit as u16);
        WheelcntReport {
            arg1,
            unit,
            direction,
            count,
        }
    }

    pub fn to_message(&self) -> Vec<u8> {
        let mut high_unit = ((self.unit >> 7) as u8) & 0x3F;
        if self.direction {
            high_unit |= 0x40;
        }
        let low_unit = self.unit as u8 & 0x7F;
        let high_count = ((self.count >> 7) as u8) & 0x7F;
        let low_count = self.count as u8 & 0x7F;
        vec![
            0xE4, 0x08, self.arg1, high_unit, low_unit, high_count, low_count,
        ]
    }

    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    pub fn unit(&self) -> u16 {
        self.unit
    }

    pub fn count(&self) -> u16 {
        self.count
    }

    pub fn direction(&self) -> bool {
        self.direction
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RepStructure {
    LissyIrReport(LissyIrReport),
    RFID5Report(RFID5Report),
    RFID7Report(RFID7Report),
    WheelcntReport(WheelcntReport),
}

impl RepStructure {
    pub fn new_lissy_ir(rep: LissyIrReport) -> Self {
        RepStructure::LissyIrReport(rep)
    }

    pub fn new_rfid_5(rep: RFID5Report) -> Self {
        RepStructure::RFID5Report(rep)
    }

    pub fn new_rfid_7(rep: RFID7Report) -> Self {
        RepStructure::RFID7Report(rep)
    }

    pub fn new_wheelcnt(rep: WheelcntReport) -> Self {
        RepStructure::WheelcntReport(rep)
    }

    pub fn parse(count: u8, args: &[u8]) -> Result<Self, MessageParseError> {
        if args[0] == 0x00 {
            Ok(Self::LissyIrReport(LissyIrReport::parse(
                args[0], args[1], args[2], args[3], args[4],
            )))
        } else if args[0] == 0x40 {
            Ok(Self::WheelcntReport(WheelcntReport::parse(
                args[0], args[1], args[2], args[3], args[4],
            )))
        } else if args[0] == 0x41 && count == 0x0C {
            Ok(Self::RFID5Report(RFID5Report::parse(
                args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8],
            )))
        } else if args[0] == 0x41 && count == 0x0E {
            Ok(Self::RFID7Report(RFID7Report::parse(
                args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8],
                args[9], args[10],
            )))
        } else {
            Err(MessageParseError::InvalidFormat("The report message was in invalid format!".into()))
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DstArg(u16);

impl DstArg {
    pub fn new(dst: u16) -> Self {
        DstArg(dst)
    }

    pub fn parse(dst_low: u8, dst_high: u8) -> Self {
        let dst = ((dst_high as u16) << 7) | (dst_low as u16);
        DstArg(dst)
    }

    pub fn dst(&self) -> u16 {
        self.0
    }

    pub fn dst_low(&self) -> u8 {
        self.0 as u8 & 0x7F
    }

    pub fn dst_high(&self) -> u8 {
        (self.0 >> 7) as u8 & 0x7F
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PxctData {
    pxc: u8,
    d1: u8,
    d2: u8,
    d3: u8,
    d4: u8,
    d5: u8,
    d6: u8,
    d7: u8,
    d8: u8,
}

impl PxctData {
    pub fn new(pxc: u8, d1: u8, d2: u8, d3: u8, d4: u8, d5: u8, d6: u8, d7: u8, d8: u8) -> Self {
        PxctData {
            pxc,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,
            d8,
        }
    }

    pub fn parse(
        pxct1: u8,
        d1: u8,
        d2: u8,
        d3: u8,
        d4: u8,
        pxct2: u8,
        d5: u8,
        d6: u8,
        d7: u8,
        d8: u8,
    ) -> Self {
        let pxc = ((pxct1 & 0x70) >> 4) | ((pxct2 & 0x70) >> 1);

        PxctData {
            pxc,
            d1: d1 | ((pxct1 & 0x01) << 6),
            d2: d2 | ((pxct1 & 0x02) << 5),
            d3: d3 | ((pxct1 & 0x04) << 4),
            d4: d4 | ((pxct1 & 0x08) << 3),
            d5: d5 | ((pxct2 & 0x01) << 6),
            d6: d6 | ((pxct2 & 0x02) << 5),
            d7: d7 | ((pxct2 & 0x04) << 4),
            d8: d8 | ((pxct2 & 0x08) << 3),
        }
    }

    pub fn pxc(&self) -> u8 {
        self.pxc
    }

    pub fn pxct1(&self) -> u8 {
        let mut pxct1 = (self.pxc & 0x07) << 4;

        if self.d1 & 0x40 == 0x40 {
            pxct1 |= 0x01;
        }
        if self.d2 & 0x40 == 0x40 {
            pxct1 |= 0x02;
        }
        if self.d3 & 0x40 == 0x40 {
            pxct1 |= 0x04;
        }
        if self.d4 & 0x40 == 0x40 {
            pxct1 |= 0x08;
        }

        pxct1
    }

    pub fn pxct2(&self) -> u8 {
        let mut pxct2 = (self.pxc & 0x78) << 1;

        if self.d5 & 0x40 == 0x40 {
            pxct2 |= 0x01;
        }
        if self.d6 & 0x40 == 0x40 {
            pxct2 |= 0x02;
        }
        if self.d7 & 0x40 == 0x40 {
            pxct2 |= 0x04;
        }
        if self.d8 & 0x40 == 0x40 {
            pxct2 |= 0x08;
        }

        pxct2
    }

    pub fn d1(&self) -> u8 {
        self.d1 & 0x3F
    }

    pub fn d2(&self) -> u8 {
        self.d2 & 0x3F
    }

    pub fn d3(&self) -> u8 {
        self.d3 & 0x3F
    }

    pub fn d4(&self) -> u8 {
        self.d4 & 0x3F
    }

    pub fn d5(&self) -> u8 {
        self.d5 & 0x3F
    }

    pub fn d6(&self) -> u8 {
        self.d6 & 0x3F
    }

    pub fn d7(&self) -> u8 {
        self.d7 & 0x3F
    }

    pub fn d8(&self) -> u8 {
        self.d8 & 0x3F
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ProgrammingAbortedArg {
    pub arg_len: u8,
    pub arg1: u8,
    pub arg2: u8,
    pub arg3: u8,
    pub arg4: u8,
    pub arg5: u8,
    pub arg6: u8,
    pub arg7: u8,
    pub arg8: u8,
    pub arg9: u8,
    pub arg10: u8,
    pub arg11: u8,
    pub arg12: u8,
    pub arg13: u8,
    pub arg14: u8,
    pub arg15: u8,
    pub arg16: u8,
    pub arg17: u8,
    pub arg18: u8,
}

impl ProgrammingAbortedArg {
    pub fn new(len: u8, args: &[u8]) -> Self {
        ProgrammingAbortedArg::parse(len, args)
    }

    pub(crate) fn parse(len: u8, args: &[u8]) -> Self {
        match len {
            0x10 => {
                ProgrammingAbortedArg {
                    arg_len: len,
                    arg1: args[0],
                    arg2: args[1],
                    arg3: args[2],
                    arg4: args[3],
                    arg5: args[4],
                    arg6: args[5],
                    arg7: args[6],
                    arg8: args[7],
                    arg9: args[8],
                    arg10: args[9],
                    arg11: args[10],
                    arg12: args[11],
                    arg13: args[12],
                    arg14: 0,
                    arg15: 0,
                    arg16: 0,
                    arg17: 0,
                    arg18: 0,
                }
            },
            len => {
                ProgrammingAbortedArg {
                    arg_len: len,
                    arg1: args[0],
                    arg2: args[1],
                    arg3: args[2],
                    arg4: args[3],
                    arg5: args[4],
                    arg6: args[5],
                    arg7: args[6],
                    arg8: args[7],
                    arg9: args[8],
                    arg10: args[9],
                    arg11: args[10],
                    arg12: args[11],
                    arg13: args[12],
                    arg14: args[13],
                    arg15: args[14],
                    arg16: args[15],
                    arg17: args[16],
                    arg18: args[17],
                }
            }
        }
    }

    pub(crate) fn to_message(self) -> Vec<u8> {
        match self.arg_len {
            0x10 => vec![
                0xE6,
                0x10,
                self.arg1,
                self.arg2,
                self.arg3,
                self.arg4,
                self.arg5,
                self.arg6,
                self.arg7,
                self.arg8,
                self.arg9,
                self.arg10,
                self.arg11,
                self.arg12,
                self.arg13
            ],
            _ => vec![
                0xE6,
                0x15,
                self.arg1,
                self.arg2,
                self.arg3,
                self.arg4,
                self.arg5,
                self.arg6,
                self.arg7,
                self.arg8,
                self.arg9,
                self.arg10,
                self.arg11,
                self.arg12,
                self.arg13,
                self.arg14,
                self.arg15,
                self.arg16,
                self.arg17,
                self.arg18
            ],
        }
    }
}
