#![allow(clippy::too_many_arguments)]

use crate::error::MessageParseError;
use crate::protocol::Message;
use std::fmt::{Debug, Display, Formatter};

/// Represents a trains address of 14 byte length.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SwitchDirection {
    Straight,
    Curved,
}

impl std::ops::Not for SwitchDirection {
    type Output = SwitchDirection;

    fn not(self) -> Self::Output {
        match self {
            SwitchDirection::Straight => SwitchDirection::Curved,
            SwitchDirection::Curved => SwitchDirection::Straight,
        }
    }
}

/// Holds switch state information to be read or write
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SwitchArg {
    /// The address of the switch (0 - 2047)
    address: u16,
    /// The switches direction state, which direction the switch points
    direction: SwitchDirection,
    /// If the switch is not in the requested direction.
    /// Use true if you want the switch to go to the direction.
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
    /// - `state`: The activation state of the switch (If the switch is in the requested state)
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
    /// The switches activation status. False if the switch has switched to the requested state.
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
    pub(crate) fn parse(trk_arg: u8) -> Self {
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
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum State {
    /// Indicates that this slot is in use by some device. The slot holds a loc address and is refreshed.
    ///
    /// If you want to mark your slot as [`State::InUse`] simply perform a `NULL`-Move on this slot. (Move message with eq, Hashual source and destination)
    InUse,
    /// A loco adr is in the slot but the slot was not refreshed.
    Idle,
    /// This slot holds some loc address and is refreshed.
    Common,
    /// No valid data in this slot, this slot is not refreshed.
    Free,
}

/// Represents the decoders speed control message format used
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
            _ => Consist::Free,
        };

        let state = match stat1 & 0x30 {
            0x30 => State::InUse,
            0x20 => State::Idle,
            0x10 => State::Common,
            0x00 => State::Free,
            _ => State::Free,
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
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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

    /// # Returns
    ///
    /// The values hold by this argument as one byte
    pub(crate) fn stat2(&self) -> u8 {
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

/// Represents a copy of the operation code with the highest bit erased
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct LopcArg(u8);

impl LopcArg {
    /// Creates a new operation code copy with the highest bit erased from the given op code byte.
    ///
    /// To get a messages operation code you can use: [Message::opc()]
    pub fn new(opc: u8) -> Self {
        LopcArg::parse(opc & 0x7F)
    }

    /// Parses a new operation code copy from an incoming byte
    pub(crate) fn parse(lopc: u8) -> Self {
        Self(lopc & 0x7F)
    }

    /// # Returns
    ///
    /// The operation code copy argument
    pub(crate) fn lopc(&self) -> u8 {
        self.0
    }

    /// Checks whether an messages operation code matches the operation code held by this argument
    ///
    /// # Parameter
    ///
    /// - `message`: The message to check operation code matching for
    ///
    /// # Returns
    ///
    /// If the messages operation code matches the operation code hold by this argument
    pub fn check_opc(&self, message: &Message) -> bool {
        message.opc() & 0x7F == self.0
    }
}

/// Holds a response code for a before received message
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct Ack1Arg(u8);

impl Ack1Arg {
    /// Creates a new acknowledgment answer
    ///
    /// # Parameter
    ///
    /// - `success`: If this acknowledgment indicates that the request was successfully
    pub fn new(success: bool) -> Self {
        Self(if success { 0x7F } else { 0x00 })
    }

    /// This creates a new acknowledgment answer with only the `code` to send as answer.
    ///
    /// `0x7F` means the request succeeded and `0x00` means the request was denied.
    ///
    /// If you want to mark that you accepted the message use `0x01` and when you want to indicate a blind acceptance use `0x40`
    pub fn new_advanced(code: u8) -> Self {
        Self(code & 0x7F)
    }

    /// Parses the acknowledgment type from a byte
    pub(crate) fn parse(ack1: u8) -> Self {
        Self(ack1)
    }

    /// # Returns
    ///
    /// The acknowledgment parsed to a byte
    pub fn ack1(&self) -> u8 {
        self.0
    }

    /// # Returns
    ///
    /// If this message indicates the operation succeeded
    pub fn success(&self) -> bool {
        self.0 == 0x7F
    }

    /// # Returns
    ///
    /// If the message has not failed
    pub fn limited_success(&self) -> bool {
        self.0 != 0x00
    }

    /// # Returns
    ///
    /// If this message indicates the operation failure
    pub fn failed(&self) -> bool {
        self.0 == 0x00
    }

    /// # Returns
    ///
    /// If this message indicates the operation was accepted but not succeeded yet
    pub fn accepted(&self) -> bool {
        self.0 == 0x01
    }

    /// # Returns
    ///
    /// If this message indicates the operation was accepted without checks, but not succeeded yet
    pub fn accepted_blind(&self) -> bool {
        self.0 == 0x40
    }
}

impl Display for Ack1Arg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.failed() {
            write!(f, "ack1: (failed)")
        } else if self.accepted() {
            write!(f, "ack1: (accepted)")
        } else if self.accepted_blind() {
            write!(f, "ack1: (accepted_blind)")
        } else {
            write!(f, "ack1: (success, ack: {})", self.0,)
        }
    }
}

/// Indicates which source type the input came from
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SourceType {
    /// Switch is connected over a DS54 port
    Ds54Aux,
    /// Switch is directly accessible
    Switch,
}

/// A sensors detection state
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SensorLevel {
    /// The sensor detects some energy flow (sensor on)
    High,
    /// The sensor detects no energy flow (sensor off)
    Low,
}

impl std::ops::Not for SensorLevel {
    type Output = SensorLevel;

    fn not(self) -> Self::Output {
        match self {
            SensorLevel::High => SensorLevel::Low,
            SensorLevel::Low => SensorLevel::High,
        }
    }
}

/// Represents an sensor input argument
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct InArg {
    /// The sensors argument
    address: u16,
    /// The sensors source type
    input_source: SourceType,
    /// The sensors detection state
    sensor_level: SensorLevel,
    /// The sensors control bit that is reserved to future use
    control_bit: bool,
}

impl InArg {
    /// Creates a new sensors input argument
    ///
    /// # Parameters
    ///
    /// - `address`: The sensors address (0 - 2047)
    /// - `input_source`: The sensors input source type
    /// - `sensor_level`: The sensors state (High = On, Low = Off)
    /// - `control_bit`: Control bit that is reserved for future use.
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

    /// Parses the sensors information from two bytes `in1` and `in2`
    pub(crate) fn parse(in1: u8, in2: u8) -> Self {
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

    /// # Returns
    ///
    /// The address of this sensor
    pub fn address(&self) -> u16 {
        self.address
    }

    /// # Returns
    ///
    /// The address with the sensors source type set as least significant bit
    pub fn address_ds54(&self) -> u16 {
        (self.address << 1)
            | match self.input_source {
                SourceType::Switch => 1,
                SourceType::Ds54Aux => 0,
            }
    }

    /// # Returns
    ///
    /// The sensors source type
    pub fn input_source(&self) -> SourceType {
        self.input_source
    }

    /// # Returns
    ///
    /// The sensors state (High = On, Low = Off)
    pub fn sensor_level(&self) -> SensorLevel {
        self.sensor_level
    }

    /// # Returns
    ///
    /// The sensors control bit
    pub fn control_bit(&self) -> bool {
        self.control_bit
    }

    /// Sets the address of this sensor argument
    ///
    /// # Parameters
    ///
    /// - `address`: The address to set (0 - 2047)
    pub fn set_address(&mut self, address: u16) {
        if address <= 0x07FF {
            self.address = address;
        }
    }

    /// Sets the address with the sensors source type as least significant bit
    ///
    /// # Parameters
    ///
    /// - `address_ds54`: The address and as least significant the source type
    pub fn set_address_ds54(&mut self, address_ds54: u16) {
        if address_ds54 <= 0x0FFF {
            self.input_source = if address_ds54 & 1 == 0 {
                SourceType::Ds54Aux
            } else {
                SourceType::Switch
            };
            self.set_address(address_ds54 >> 1);
        }
    }

    /// Sets the sensors input source type
    ///
    /// # Parameters
    ///
    /// - `input_source`: The input source the sensor used
    pub fn set_input_source(&mut self, input_source: SourceType) {
        self.input_source = input_source;
    }

    /// Sets the sensors activation state
    ///
    /// # Parameters
    ///
    /// - `sensor_level`: The activation state to use (High = ON, Low = OFF)
    pub fn set_sensor_level(&mut self, sensor_level: SensorLevel) {
        self.sensor_level = sensor_level;
    }

    /// Sets the control bit of this sensor arg to the given value.
    ///
    /// # Parameters
    ///
    /// - `control_bit`: The bit to set
    pub fn set_control_bit(&mut self, control_bit: bool) {
        self.control_bit = control_bit;
    }

    /// Parses this sensors least significant address bit in one byte
    pub(crate) fn in1(&self) -> u8 {
        self.address as u8 & 0x7F
    }

    /// Parses this sensors most significant address bit and its input source type
    /// as well as the sensor activation state and control bit in one byte,
    pub(crate) fn in2(&self) -> u8 {
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

/// Metainformation for a device
#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub enum SnArg {
    /// The devices meta information by device type
    /// - 0: Device address
    /// - 1: If this device is a switch
    /// - 2: If this device is active
    SwitchType(u16, bool, bool),
    /// The devices meta information by output
    /// - 0: Device address
    /// - 1: The activation state of the straight switch part
    /// - 2: The activation state of the curved switch part
    SwitchDirectionStatus(u16, SensorLevel, SensorLevel),
}

impl SnArg {
    /// Parses the sensors information from two bytes `sn1` and `sn2`
    pub(crate) fn parse(sn1: u8, sn2: u8) -> Self {
        let mut address = sn1 as u16;
        address |= (sn2 as u16 & 0x0F) << 7;

        let format = sn2 & 0x40 == 0x40;

        let t = sn2 & 0x10 == 0x10;
        let c = sn2 & 0x20 == 0x20;

        if format {
            SnArg::SwitchType(address, c, t)
        } else {
            SnArg::SwitchDirectionStatus(
                address,
                if c {
                    SensorLevel::High
                } else {
                    SensorLevel::Low
                },
                if t {
                    SensorLevel::High
                } else {
                    SensorLevel::Low
                },
            )
        }
    }

    /// # Returns
    ///
    /// The device address
    pub fn address(&self) -> u16 {
        match *self {
            SnArg::SwitchType(address, ..) => address,
            SnArg::SwitchDirectionStatus(address, ..) => address,
        }
    }

    /// # Returns
    ///
    /// Parses this low address bits in a writeable byte
    pub(crate) fn sn1(&self) -> u8 {
        (match *self {
            SnArg::SwitchDirectionStatus(address, ..) => address,
            SnArg::SwitchType(address, ..) => address,
        } as u8)
            & 0x7F
    }

    /// # Returns
    ///
    /// Parses the status information and the high address bits into a writeable byte
    pub(crate) fn sn2(&self) -> u8 {
        match *self {
            SnArg::SwitchType(address, is_switch, state) => {
                let mut sn2 = ((address >> 7) as u8 & 0x0F) | 0x40;

                sn2 |= if is_switch { 0x20 } else { 0x00 };
                sn2 | if state { 0x10 } else { 0x00 }
            }
            SnArg::SwitchDirectionStatus(address, straight_status, curved_status) => {
                let mut sn2 = (address >> 7) as u8 & 0x0F;

                sn2 |= match straight_status {
                    SensorLevel::High => 0x20,
                    SensorLevel::Low => 0x00,
                };
                sn2 | match curved_status {
                    SensorLevel::High => 0x10,
                    SensorLevel::Low => 0x00,
                }
            }
        }
    }
}

/// Id of the slot controlling device
///
/// - 0: No ID being used
/// - 00/80 - 3F/81: ID shows PC usage
/// - 00/02 - 3F/83: System reserved
/// - 00/04 - 3F/FE: normal throttle range
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct IdArg(u16);

impl IdArg {
    /// Creates a new device id
    ///
    /// # Parameters
    ///
    /// - `id`: A fourteen bit device address
    pub fn new(id: u16) -> Self {
        IdArg(id & 0x3FFF)
    }

    /// Parses the device id from two bytes `id1` and `id2`
    pub(crate) fn parse(id1: u8, id2: u8) -> Self {
        IdArg((((id2 & 0x7F) as u16) << 7) | ((id1 & 0x7F) as u16))
    }

    /// # Returns
    ///
    /// The device `id`
    pub fn id(&self) -> u16 {
        self.0
    }

    /// # Returns
    ///
    /// The seven least significant address bits
    pub(crate) fn id1(&self) -> u8 {
        self.0 as u8 & 0x7F
    }

    /// # Returns
    ///
    /// The seven most significant address bits
    pub(crate) fn id2(&self) -> u8 {
        (self.0 >> 7) as u8 & 0x7F
    }
}

/// Represents power information for a specific railway sector
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct MultiSenseArg {
    /// This messages three bit represented type
    m_type: u8,
    /// The present state
    present: bool,
    /// The board address corresponding to this message
    board_address: u8,
    /// The zone corresponding to this message
    zone: u8,
}

impl MultiSenseArg {
    /// Creates new power information for a specified railway sector
    ///
    /// # Parameters
    ///
    /// - `m_type`: The messages type
    /// - `present`: The present state of the sender
    /// - `board_address`: The board address
    /// - `zone`: The zone address
    pub fn new(m_type: u8, present: bool, board_address: u8, zone: u8) -> Self {
        Self {
            m_type: m_type & 0x07,
            present,
            board_address,
            zone,
        }
    }

    /// Parses the power information id from two bytes `m_high` and `zas`
    pub(crate) fn parse(m_high: u8, zas: u8) -> Self {
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

    /// # Returns
    ///
    /// The three bit message type
    pub fn m_type(&self) -> u8 {
        self.m_type
    }

    /// # Returns
    ///
    /// The senders present status
    pub fn present(&self) -> bool {
        self.present
    }

    /// # Returns
    ///
    /// The sections board address
    pub fn board_address(&self) -> u8 {
        self.board_address
    }

    /// # Returns
    ///
    /// The sections zone
    pub fn zone(&self) -> u8 {
        self.zone
    }

    /// # Returns
    ///
    /// One byte holding the least significant board address and zone bits
    pub(crate) fn zas(&self) -> u8 {
        self.zone | ((self.board_address & 0x0F) << 4)
    }

    /// # Returns
    ///
    /// The low address bits as well as the messages type and present status as one byte
    pub(crate) fn m_high(&self) -> u8 {
        ((self.board_address & 0xF0) >> 4)
            | ((self.m_type & 0x07) << 5)
            | if self.present { 0x10 } else { 0x00 }
    }
}

/// The functions group
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum FunctionGroup {
    /// Function bits 9, 10 and 11 are available
    F9TO11,
    /// Function bits 13 to 19 are available
    F13TO19,
    /// Function bits 12, 20 and 28 are available
    F12F20F28,
    /// Function bit 21 to 27 are available
    F21TO27,
}

/// Represents the function bits of one function group.
///
/// - 0: The functions group type
/// - 1: The functions bits set
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct FunctionArg(u8, u8);

impl FunctionArg {
    /// Creates a new function arg for a given group.
    ///
    /// # Parameters
    ///
    /// - `group`: The functions group to set the values to.
    pub fn new(group: FunctionGroup) -> Self {
        FunctionArg(
            match group {
                FunctionGroup::F9TO11 => 0x07,
                FunctionGroup::F13TO19 => 0x08,
                FunctionGroup::F12F20F28 => 0x05,
                FunctionGroup::F21TO27 => 0x09,
            },
            0,
        )
    }

    /// Parses the group and function bits from two bits.
    pub(crate) fn parse(group: u8, function: u8) -> Self {
        FunctionArg(group, function)
    }

    /// # Returns
    ///
    /// The value of the `f_num`s function bit value if this bit is contained in
    /// this args function group.
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

    /// Sets the `f_num` function bits value, if it is present in this args function group.
    ///
    /// # Parameters
    ///
    /// - `f_num`: The bit to set
    /// - `value`: The bits value
    ///
    /// # Returns
    ///
    /// A mutable reference of this struct instance.
    pub fn set_f(&mut self, f_num: u8, value: bool) -> &mut Self {
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

        self
    }

    /// # Returns
    ///
    /// The function group specifying which function values may be set.
    pub fn function_group(&self) -> FunctionGroup {
        match self.0 {
            0x07 => FunctionGroup::F9TO11,
            0x05 => FunctionGroup::F12F20F28,
            0x08 => FunctionGroup::F13TO19,
            0x09 => FunctionGroup::F21TO27,
            _ => FunctionGroup::F9TO11,
        }
    }

    /// # Returns
    ///
    /// The functions group represented as one byte.
    pub(crate) fn group(&self) -> u8 {
        self.0
    }

    /// # Returns
    ///
    /// The function bits represented as one byte.
    pub(crate) fn function(&self) -> u8 {
        self.1
    }
}

/// Overriding debug to only display the relevant function bits.
impl Debug for FunctionArg {
    /// Prints the group corresponding function bit values.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.function_group() {
            FunctionGroup::F9TO11 => {
                write!(
                    f,
                    "function_arg: (group: {:?}, f9: {}, f10: {}, f11: {})",
                    FunctionGroup::F9TO11,
                    self.f(9),
                    self.f(10),
                    self.f(11)
                )
            }
            FunctionGroup::F13TO19 => {
                write!(f,
                       "function_arg: (group: {:?}, f13: {}, f14: {}, f15: {}, f16: {}, f17: {}, f18: {}, f19: {})",
                       FunctionGroup::F13TO19,
                       self.f(13),
                       self.f(14),
                       self.f(15),
                       self.f(16),
                       self.f(17),
                       self.f(18),
                       self.f(19),
                )
            }
            FunctionGroup::F12F20F28 => {
                write!(
                    f,
                    "function_arg: (group: {:?}, f12: {}, f20: {}, f28: {})",
                    FunctionGroup::F12F20F28,
                    self.f(12),
                    self.f(20),
                    self.f(28)
                )
            }
            FunctionGroup::F21TO27 => {
                write!(f,
                       "function_arg: (group: {:?}, f21: {}, f22: {}, f23: {}, f24: {}, f25: {}, f26: {}, f27: {})",
                       FunctionGroup::F21TO27,
                       self.f(21),
                       self.f(22),
                       self.f(23),
                       self.f(24),
                       self.f(25),
                       self.f(26),
                       self.f(27)
                )
            }
        }
    }
}

/// Representing the command mode used to write to the programming track
///
/// # Type Codes Table
///
/// | [Pcmd::byte_mode] | [Pcmd::ops_mode] | [Pcmd::ty0] | [Pcmd::ty1] | Mode                            |
/// |-------------------|------------------|-------------|-------------|---------------------------------|
/// | 0                 | 0                | 0           | 0           | Abort operation                 |
/// | 1                 | 0                | 0           | 0           | Paged mode                      |
/// | x                 | 0                | 0           | 1           | Direct mode                     |
/// | x                 | 0                | 1           | 0           | Physical register               |
/// | x                 | 0                | 1           | 1           | service track reserved function |
/// | x                 | 1                | 0           | 0           | no feedback                     |
/// | x                 | 1                | 0           | 0           | feedback                        |
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct Pcmd {
    /// Whether to write or if `false` read
    write: bool,
    /// Whether to use byte or bitwise operation mode
    byte_mode: bool,
    /// Whether to use the main track or the programming track
    ops_mode: bool,
    /// First programing type select bit
    ty0: bool,
    /// Second programming type select bit
    ty1: bool,
}

impl Pcmd {
    /// Creates a new programm control argument
    ///
    /// For near information on `ty0` and `ty1` see [Pcmd].
    ///
    /// # Parameters
    ///
    /// - `write`: Whether to write or read
    /// - `byte_mode`: Whether to use bytewise or bitwise mode
    /// - `ops_mode`: Whether to use the main track or the programming track
    /// - `ty0`: See [Pcmd]
    /// - `ty1`: See [Pcmd]
    pub fn new(write: bool, byte_mode: bool, ops_mode: bool, ty0: bool, ty1: bool) -> Self {
        Pcmd {
            write,
            byte_mode,
            ops_mode,
            ty0,
            ty1,
        }
    }

    /// Reads the programming control information from one byte
    pub(crate) fn parse(pcmd: u8) -> Self {
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

    /// # Returns
    ///
    /// Whether to write or read
    pub fn write(&self) -> bool {
        self.write
    }

    /// # Returns
    ///
    /// Whether to use byte or bit mode
    pub fn byte_mode(&self) -> bool {
        self.byte_mode
    }

    /// # Returns
    ///
    /// Whether to use the main or programming track
    pub fn ops_mode(&self) -> bool {
        self.ops_mode
    }

    /// See [Pcmd]
    pub fn ty0(&self) -> bool {
        self.ty0
    }

    /// See [Pcmd]
    pub fn ty1(&self) -> bool {
        self.ty1
    }

    /// Sets the write argument
    ///
    /// # Parameters
    ///
    /// - `write`: Whether to write or read
    pub fn set_write(&mut self, write: bool) {
        self.write = write
    }

    /// Sets the byte_mode argument
    ///
    /// # Parameters
    ///
    /// - `byte_mode`: Whether to use byte or bit mode
    pub fn set_byte_mode(&mut self, byte_mode: bool) {
        self.byte_mode = byte_mode
    }

    /// Sets the ops_mode argument
    ///
    /// # Parameters
    ///
    /// - `ops_mode`: Whether to use the main or programming track
    pub fn set_ops_mode(&mut self, ops_mode: bool) {
        self.ops_mode = ops_mode
    }

    /// See [Pcmd]
    pub fn set_ty0(&mut self, ty0: bool) {
        self.ty0 = ty0
    }

    /// See [Pcmd]
    pub fn set_ty1(&mut self, ty1: bool) {
        self.ty1 = ty1
    }

    /// # Returns
    ///
    /// Parses the programming information data into one representing byte
    pub(crate) fn pcmd(&self) -> u8 {
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

/// Holding programming error flags
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct PStat {
    /// User canceled operation
    user_aborted: bool,
    /// No read acknowledgment
    no_read_ack: bool,
    /// No write acknowledgment
    no_write_ack: bool,
    /// No train on the programming track to programm
    programming_track_empty: bool,
}

impl PStat {
    /// Creates new programming error information
    ///
    /// # Parameters
    ///
    /// - `user_aborted`: If an user canceled the programming operation
    /// - `no_read_ack`: No read acknowledgment received
    /// - `no_write_ack`: No write acknowledgment received
    /// - `programming_track_empty`: No train is on the programming track
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

    /// Parses the error flags from one byte
    pub(crate) fn parse(stat: u8) -> Self {
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

    /// # Returns
    ///
    /// If the operation was aborted by a user
    pub fn user_aborted(&self) -> bool {
        self.user_aborted
    }

    /// # Returns
    ///
    /// If the operation was canceled by a missing read acknowledgment
    pub fn no_read_ack(&self) -> bool {
        self.no_read_ack
    }

    /// # Returns
    ///
    /// If the operation was canceled by a missing write acknowledgment
    pub fn no_write_ack(&self) -> bool {
        self.no_write_ack
    }

    /// # Returns
    ///
    /// If no train was found to programm
    pub fn programming_track_empty(&self) -> bool {
        self.programming_track_empty
    }

    /// # Returns
    ///
    /// A byte representing all found error states
    pub(crate) fn stat(&self) -> u8 {
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

/// Holds control variables and data arguments.
#[derive(Copy, Clone, Eq, Hash, PartialEq, Default)]
pub struct CvDataArg(u16, u8);

impl CvDataArg {
    /// Creates a new empty arg.
    pub fn new() -> CvDataArg {
        CvDataArg(0, 0)
    }

    /// Parses cv and data from three byte
    pub(crate) fn parse(cvh: u8, cvl: u8, data7: u8) -> Self {
        let mut cv_arg = cvl as u16;
        let data = ((cvh & 0x02) << 6) | data7;

        let mut high_cv_arg = cvh & 0x01;
        high_cv_arg |= (cvh & 0x30) >> 3;

        cv_arg |= (high_cv_arg as u16) << 7;

        CvDataArg(cv_arg, data)
    }

    /// # Parameters
    ///
    /// - `d_num`: Wich data bit to return (Value must be between 0 and 7 (inclusive))
    ///
    /// # Returns
    ///
    /// The data bit specified by `d_num`
    pub fn data(&self, d_num: u8) -> bool {
        (self.1 >> d_num) & 0x01 != 0
    }

    /// # Parameters
    ///
    /// - `cv_num`: Wich cv bit to return (Value must be between 0 and 9 (inclusive))
    ///
    /// # Returns
    ///
    /// The cv bit specified by `cv_num`
    pub fn cv(&self, cv_num: u8) -> bool {
        self.0 >> cv_num & 1 != 0
    }

    /// Sets the specified data bit to the given state
    ///
    /// # Parameters
    ///
    /// - `d_num`: Wich data bit to set
    /// - `value`: The value to set the data bit to
    pub fn set_data(&mut self, d_num: u8, value: bool) -> &mut Self {
        let mask = 1 << d_num;

        if value {
            self.1 |= mask;
        } else {
            self.1 &= !mask;
        }

        self
    }

    /// Sets the specified cv bit to the given state
    ///
    /// # Parameters
    ///
    /// - `cv_num`: Wich cv bit to set
    /// - `value`: The value to set the cv bit to
    pub fn set_cv(&mut self, cv_num: u8, value: bool) -> &mut Self {
        let mask = (1 << cv_num) & 0x03FF;

        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }

        self
    }

    /// # Returns
    ///
    /// The high part of the cv values and the seventh data bit as one byte
    pub(crate) fn cvh(&self) -> u8 {
        let mut cvh = (self.0 >> 7) as u8;
        let high_cv = cvh & 0x06 << 3;
        cvh &= 0x01;
        cvh |= high_cv;
        if self.data(7) {
            cvh |= 0x02;
        }
        cvh
    }

    /// # Returns
    ///
    /// The low part of the cv values as one byte
    pub(crate) fn cvl(&self) -> u8 {
        self.0 as u8 & 0x7F
    }

    /// # Returns
    ///
    /// The data bits from 0 to 6 (inclusive) as one byte
    pub(crate) fn data7(&self) -> u8 {
        self.1 & 0x7F
    }
}

/// Overridden for precise value orientated output
impl Debug for CvDataArg {
    /// Writes all args and cv values to the formatter
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cv_data_arg: (data: (d0: {}, d1: {}, d2: {}, d3: {}, d4: {}, d5: {}, d6: {}, d7: {}), cv: (cv0: {}, cv1: {}, cv2: {}, cv3: {}, cv4: {}, cv5: {}, cv6: {}, cv7: {}, cv8: {}, cv9: {}))",
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

/// Holding the clocks information
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct FastClock {
    /// The clocks tick rate. (0 = Frozen), (x = x to 1 rate),
    clk_rate: u8,
    /// Intern subminute counter
    frac_mins: u16,
    /// The clocks minutes
    mins: u8,
    /// The clocks set hours
    hours: u8,
    /// The clocks set days
    days: u8,
    /// The clock control
    clk_cntrl: u8,
}

impl FastClock {
    /// Creates a new clock synchronise information
    ///
    /// # Parameters
    ///
    /// - `clock_rate`: The clocks tick rate. (0 = Frozen), (x = x to 1 rate)
    /// - `frac_mins`: The internal subminute counter
    /// - `mins`: The clock mins calculated by 256-MINS%60
    /// - `hours`: The clocks hours calculated by 256-HRS%24
    /// - `days`: The number of 24 hour cycles passed
    /// - `clk_cntrl`: Clock control information. third bit must be true to mark this clock data valid.
    pub fn new(clk_rate: u8, frac_mins: u16, mins: u8, hours: u8, days: u8, clk_cntrl: u8) -> Self {
        FastClock {
            clk_rate,
            frac_mins,
            mins,
            hours,
            days,
            clk_cntrl,
        }
    }

    /// Calculates the clock information from 7 bytes
    ///
    /// # Parameters
    ///
    /// - `clock_rate`: The clocks tick rate. (0 = Frozen), (x = x to 1 rate)
    /// - `frac_minsl`: The least significant part of the internal subminute counter
    /// - `frac_minsh`: The most significant part of the internal subminute counter
    /// - `mins`: The clock mins calculated by 256-MINS%60
    /// - `hours`: The clocks hours calculated by 256-HRS%24
    /// - `days`: The number of 24 hour cycles passed
    /// - `clk_cntrl`: Clock control information. third bit must be true to mark this clock data valid.
    fn parse(
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
            frac_mins: (frac_minsl as u16) | ((frac_minsh as u16) << 8),
            mins,
            hours,
            days,
            clk_cntrl,
        }
    }

    /// # Returns
    ///
    /// The clocks rate
    pub fn clk_rate(&self) -> u8 {
        self.clk_rate
    }

    /// # Returns
    ///
    /// The clocks least significant internal counter part
    fn frac_minsl(&self) -> u8 {
        self.frac_mins as u8
    }

    /// # Returns
    ///
    /// The clocks most significant internal counter part
    fn frac_minsh(&self) -> u8 {
        (self.frac_mins >> 8) as u8
    }

    /// # Returns
    ///
    /// The internal clock counter
    pub fn frac_mins(&self) -> u16 {
        self.frac_mins
    }

    /// # Returns
    ///
    /// The clocks minutes. Represented by (256-MINS%60)
    pub fn mins(&self) -> u8 {
        self.mins
    }

    /// # Returns
    ///
    /// The clocks hours. Represented by (256-HRS%24)
    pub fn hours(&self) -> u8 {
        self.hours
    }

    /// # Retuns
    ///
    /// The count of 24 hour cycles passed
    pub fn days(&self) -> u8 {
        self.days
    }

    /// # Returns
    ///
    /// General clock control information.
    ///
    /// The third bit represents the valid state of this message (0 = invalid)
    pub fn clk_cntrl(&self) -> u8 {
        self.clk_cntrl
    }
}

/// The function bits accessible by the corresponding [ImArg]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum ImFunctionType {
    /// Functions 9 to 12 (inclusive) are accessible
    F9to12,
    /// Functions 13 to 20 (inclusive) are accessible
    F13to20,
    /// Functions 21 to 28 (inclusive) are accessible
    F21to28,
}

/// The address in the right format used by the corresponding [ImArg]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum ImAddress {
    /// A short 8 bit address
    Short(u8),
    /// A long 16 bit address
    Long(u16),
}

/// This arg hold function bit information
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ImArg {
    /// I don't get the concrete meaning and functionality of this arg
    dhi: u8,
    /// This is the address to set the function bits to
    address: ImAddress,
    /// This is the functions settable by this arg
    function_type: ImFunctionType,
    /// This holds the function bits
    function_bits: u8,
    /// Unused for now, do what you want
    im5: u8,
}

impl ImArg {
    /// Creates a new function arg
    ///
    /// # Parameters
    ///
    /// - `dhi`: I don't get the concrete meaning and functionality of this arg
    /// - `address`: The address to set the function bits for
    /// - `function_type`: Wich functions should be settable
    /// - `im5`: Unused parameter
    pub fn new(dhi: u8, address: ImAddress, function_type: ImFunctionType, im5: u8) -> Self {
        ImArg {
            dhi,
            address,
            function_type,
            function_bits: 0x00,
            im5,
        }
    }

    /// Calculates the information of one im arg from eight bytes
    ///
    /// # Parameters
    ///
    /// - `_`: Not used, as it was always the same value
    /// - `reps`: The function bits range type
    /// - `dhi`: Not understood by me
    /// - `im1-5`: The address and function bits
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

    /// # Returns
    ///
    /// The type of this function arg as one byte
    pub(crate) fn reps(&self) -> u8 {
        match self.address {
            ImAddress::Short(_) => match self.function_type {
                ImFunctionType::F9to12 => 0x24,
                ImFunctionType::F13to20 => 0x34,
                ImFunctionType::F21to28 => 0x34,
            },
            ImAddress::Long(_) => match self.function_type {
                ImFunctionType::F9to12 => 0x34,
                ImFunctionType::F13to20 => 0x44,
                ImFunctionType::F21to28 => 0x44,
            },
        }
    }

    /// # Returns
    ///
    /// The dhi byte, holding special address and bit information.
    pub fn dhi(&self) -> u8 {
        self.dhi
    }

    /// # Returns
    ///
    /// The address in long or short format
    pub fn address(&self) -> ImAddress {
        self.address
    }

    /// # Returns
    ///
    /// The type specifying wich function bits are settable
    pub fn function_type(&self) -> ImFunctionType {
        self.function_type
    }

    /// Calculates the `f_num`s function bit
    ///
    /// # Parameters
    ///
    /// - `f_num`: The function bits number to get
    ///
    /// # Returns
    ///
    /// The value of the `f_num`s function bit
    pub fn f(&self, f_num: u8) -> bool {
        let dist = match self.function_type {
            ImFunctionType::F13to20 => 21,
            ImFunctionType::F21to28 => 13,
            ImFunctionType::F9to12 => 9,
        };

        (self.function_bits >> (f_num - dist)) & 0x01 == 0x01
    }

    /// Sets the `f_num`s function bit to the given value `f`.
    ///
    /// # Parameters
    ///
    /// - `f_num`: The function bit to set
    /// - `f`: The value to set the function bit to
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

    /// # Returns
    ///
    /// The first function arg
    pub(crate) fn im1(&self) -> u8 {
        match self.address {
            ImAddress::Short(adr) => adr,
            ImAddress::Long(adr) => adr as u8,
        }
    }

    /// # Returns
    ///
    /// The second function arg
    pub(crate) fn im2(&self) -> u8 {
        match self.address {
            ImAddress::Short(_) => match self.function_type {
                ImFunctionType::F9to12 => (self.function_bits & 0x7F) | 0x20,
                ImFunctionType::F13to20 => 0x5E,
                ImFunctionType::F21to28 => 0x5F,
            },
            ImAddress::Long(adr) => (adr >> 8) as u8,
        }
    }

    /// # Returns
    ///
    /// The third function arg
    pub(crate) fn im3(&self) -> u8 {
        match self.address {
            ImAddress::Short(_) => {
                if self.function_type == ImFunctionType::F9to12 {
                    0x00
                } else {
                    self.function_bits
                }
            }
            ImAddress::Long(_) => match self.function_type {
                ImFunctionType::F9to12 => (self.function_bits & 0x7F) | 0x20,
                ImFunctionType::F13to20 => 0x5E,
                ImFunctionType::F21to28 => 0x5F,
            },
        }
    }

    /// # Returns
    ///
    /// The fourth function arg
    pub(crate) fn im4(&self) -> u8 {
        if self.reps() == 0x34 && self.function_type != ImFunctionType::F9to12 {
            return self.function_bits;
        }
        0x00
    }

    /// # Returns
    ///
    /// The fifth function arg
    pub(crate) fn im5(&self) -> u8 {
        self.im5
    }
}

/// Holds messages for writing data to slots
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum WrSlDataStructure {
    /// Represents clock sync information
    ///
    /// # Parameters
    ///
    /// - `FastClock`: The clock information
    /// - `TrkArg`: The track
    /// - `IdArg`: The ID of the slots user
    DataTime(FastClock, TrkArg, IdArg),
    /// Creates new data to write to the programming track
    ///
    /// # Parameters
    ///
    /// - `Pcmd`: The programming command to use
    /// - `AddressArg`: Operation mode programming bits as address
    /// - `TrkArg`: The current track information to set
    /// - `CvDataArg`: The command value and data bits to programm
    DataPt(Pcmd, AddressArg, TrkArg, CvDataArg),
    /// Represents a general message to write data to one specified slot
    ///
    /// # Parameters
    ///
    /// - `SlotArg`: The slot to write to
    /// - `Stat1Arg`: The slots general status information
    /// - `Stat2Arg`: Additional slot status information
    /// - `AddressArg`: The slots corresponding address
    /// - `SpeedArg`: The slots set speed
    /// - `DirfArg`: The direction and low function bits
    /// - `TrkArg`: The general track information
    /// - `SndArg`: Additional function bits
    /// - `IdArg`: The ID of the slots user
    DataGeneral(
        SlotArg,
        Stat1Arg,
        Stat2Arg,
        AddressArg,
        SpeedArg,
        DirfArg,
        TrkArg,
        SndArg,
        IdArg,
    ),
}

impl WrSlDataStructure {
    /// Parses eleven incoming bytes to one write slot data message
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
                Pcmd::parse(arg2),
                AddressArg::parse(arg4, arg5),
                TrkArg::parse(arg6),
                CvDataArg::parse(arg7, arg8, arg9),
            )
        } else if arg1 == 0x7B {
            WrSlDataStructure::DataTime(
                FastClock::parse(arg2, arg3, arg4, arg5, arg7, arg8, arg9),
                TrkArg::parse(arg6),
                IdArg::parse(arg10, arg11),
            )
        } else {
            WrSlDataStructure::DataGeneral(
                SlotArg::parse(arg1),
                Stat1Arg::parse(arg2),
                Stat2Arg::parse(arg7),
                AddressArg::parse(arg8, arg3),
                SpeedArg::parse(arg4),
                DirfArg::parse(arg5),
                TrkArg::parse(arg6),
                SndArg::parse(arg9),
                IdArg::parse(arg10, arg11),
            )
        }
    }

    /// # Returns
    ///
    /// The slot this message is written to
    pub fn slot_type(&self) -> u8 {
        match self {
            WrSlDataStructure::DataPt(..) => 0x7C,
            WrSlDataStructure::DataTime(..) => 0x7B,
            WrSlDataStructure::DataGeneral(slot, ..) => slot.slot(),
        }
    }

    /// # Returns
    ///
    /// This message as a sequence of 13 bytes
    pub(crate) fn to_message(self) -> Vec<u8> {
        match self {
            WrSlDataStructure::DataPt(pcmd, adr, trk, cv_data) => {
                vec![
                    0xEF,
                    0x0E,
                    0x7C,
                    pcmd.pcmd(),
                    0x00,
                    adr.adr2(),
                    adr.adr1(),
                    trk.trk_arg(),
                    cv_data.cvh(),
                    cv_data.cvl(),
                    cv_data.data7(),
                    0x00,
                    0x00,
                ]
            }
            WrSlDataStructure::DataTime(fast_clock, trk, id) => {
                vec![
                    0xEF,
                    0x0E,
                    0x7B,
                    fast_clock.clk_rate(),
                    fast_clock.frac_minsl(),
                    fast_clock.frac_minsh(),
                    fast_clock.mins(),
                    trk.trk_arg(),
                    fast_clock.hours(),
                    fast_clock.days(),
                    fast_clock.clk_cntrl(),
                    id.id1(),
                    id.id2(),
                ]
            }
            WrSlDataStructure::DataGeneral(
                slot,
                stat1,
                stat2,
                adr,
                speed,
                dirf,
                trk,
                sound,
                id,
            ) => {
                vec![
                    0xEF,
                    0x0E,
                    slot.slot(),
                    stat1.stat1(),
                    adr.adr1(),
                    speed.spd(),
                    dirf.dirf(),
                    trk.trk_arg(),
                    stat2.stat2(),
                    adr.adr2(),
                    sound.snd(),
                    id.id1(),
                    id.id2(),
                ]
            }
        }
    }
}

/// Lissy IR reports status information
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct LissyIrReport {
    arg1: u8,
    dir: bool,
    unit: u16,
    address: u16,
}

impl LissyIrReport {
    /// Creates a new report
    ///
    /// # Parameters
    ///
    /// - `dir`: The direction
    /// - `unit`: The reports unit
    /// - `address`: The reports address
    pub fn new(dir: bool, unit: u16, address: u16) -> Self {
        LissyIrReport {
            arg1: 0x00,
            dir,
            unit,
            address,
        }
    }

    /// Parses the report information from five bytes
    ///
    /// # Parameters
    ///
    /// - `arg1`: Specifies the report type
    /// - `high_unit`: The most significant unit bits and the direction
    /// - `low_unit`: The least significant unit bits
    /// - `high_adr`: The most significant address bits
    /// - `low_adr`: The least significant address bits
    pub(crate) fn parse(arg1: u8, high_unit: u8, low_unit: u8, high_adr: u8, low_adr: u8) -> Self {
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

    /// # Returns
    ///
    /// This message represented by a vector of seven bytes
    pub(crate) fn to_message(self) -> Vec<u8> {
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

    /// # Returns
    ///
    /// The messages type byte
    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    /// # Returns
    ///
    /// The direction
    pub fn dir(&self) -> bool {
        self.dir
    }

    /// # Returns
    ///
    /// The unit of this message
    pub fn unit(&self) -> u16 {
        self.unit
    }

    /// # Returns
    ///
    /// The messages address
    pub fn address(&self) -> u16 {
        self.address
    }
}

/// Holds report information of a rfid5 report message
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
    /// Creates new report information
    ///
    /// # Parameters
    ///
    /// - `address`: The reporters address
    /// - `rfid0` - `rfid4` and `rfid_hi`: The reported rfid values
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

    /// Parses this message from nine bytes
    ///
    /// # Parameters
    ///
    /// - `arg1`: This reports type byte
    /// - `high_adr`: This most significant address part
    /// - `low_adr`: This least significant address part
    /// - `rfid0` - `rfid4` and `rfid_hi`: The reported rfid values
    pub(crate) fn parse(
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

    /// # Returns
    ///
    /// This message parsed represented by 11 bytes
    pub(crate) fn to_message(self) -> Vec<u8> {
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

    /// # Returns
    ///
    /// The messages type byte
    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    /// # Returns
    ///
    /// The reporters address
    pub fn address(&self) -> u16 {
        self.address
    }

    /// # Returns
    ///
    /// The first reported rfid byte
    pub fn rfid0(&self) -> u8 {
        self.rfid0
    }

    /// # Returns
    ///
    /// The second reported rfid byte
    pub fn rfid1(&self) -> u8 {
        self.rfid1
    }

    /// # Returns
    ///
    /// The third reported rfid byte
    pub fn rfid2(&self) -> u8 {
        self.rfid2
    }

    /// # Returns
    ///
    /// The fourth reported rfid byte
    pub fn rfid3(&self) -> u8 {
        self.rfid3
    }

    /// # Returns
    ///
    /// The fifth reported rfid byte
    pub fn rfid4(&self) -> u8 {
        self.rfid4
    }

    /// # Returns
    ///
    /// The last reported rfid byte
    pub fn rfid_hi(&self) -> u8 {
        self.rfid_hi
    }
}

/// Holds report information of a rfid7 report message
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
    /// Creates new report information
    ///
    /// # Parameters
    ///
    /// - `address`: The reporters address
    /// - `rfid0` - `rfid6` and `rfid_hi`: The reported rfid values
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

    /// Parses this message from eleven bytes
    ///
    /// # Parameters
    ///
    /// - `arg1`: This reports type byte
    /// - `high_adr`: This most significant address part
    /// - `low_adr`: This least significant address part
    /// - `rfid0` - `rfid6` and `rfid_hi`: The reported rfid values
    pub(crate) fn parse(
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

    /// # Returns
    ///
    /// This message represented by 13 bytes
    pub(crate) fn to_message(self) -> Vec<u8> {
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

    /// # Returns
    ///
    /// The messages type byte
    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    /// # Returns
    ///
    /// The reporters address
    pub fn address(&self) -> u16 {
        self.address
    }

    /// # Returns
    ///
    /// The first reported rfid byte
    pub fn rfid0(&self) -> u8 {
        self.rfid0
    }

    /// # Returns
    ///
    /// The second reported rfid byte
    pub fn rfid1(&self) -> u8 {
        self.rfid1
    }

    /// # Returns
    ///
    /// The third reported rfid byte
    pub fn rfid2(&self) -> u8 {
        self.rfid2
    }

    /// # Returns
    ///
    /// The fourth reported rfid byte
    pub fn rfid3(&self) -> u8 {
        self.rfid3
    }

    /// # Returns
    ///
    /// The fifth reported rfid byte
    pub fn rfid4(&self) -> u8 {
        self.rfid4
    }

    /// # Returns
    ///
    /// The sixth reported rfid byte
    pub fn rfid5(&self) -> u8 {
        self.rfid5
    }

    /// # Returns
    ///
    /// The seventh reported rfid byte
    pub fn rfid6(&self) -> u8 {
        self.rfid6
    }

    /// # Returns
    ///
    /// The last reported rfid byte
    pub fn rfid_hi(&self) -> u8 {
        self.rfid_hi
    }
}

/// Holds wheel counter report information
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct WheelcntReport {
    arg1: u8,
    unit: u16,
    direction: bool,
    count: u16,
}

impl WheelcntReport {
    /// Creates new wheel counter report information
    ///
    /// # Parameters
    ///
    /// - `unit`: The reports unit
    /// - `direction`: The reports direction
    /// - `count`: The reports wheel count
    pub fn new(unit: u16, direction: bool, count: u16) -> Self {
        WheelcntReport {
            arg1: 0x40,
            unit,
            direction,
            count,
        }
    }

    /// Parses the wheel count information from five bytes
    ///
    /// # Parameters
    ///
    /// - `arg1`: The reports type byte
    /// - `high_unit`: The most significant unit bits and the direction
    /// - `low_unit`: The least significant unit bits
    /// - `high_count`: The most significant count bits
    /// - `low_count`: The least significant count bits
    pub(crate) fn parse(
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

    /// # Returns
    ///
    /// This message represented by seven bytes
    pub(crate) fn to_message(self) -> Vec<u8> {
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

    /// # Returns
    ///
    /// This reports type byte
    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    /// # Returns
    ///
    /// The unit of this report
    pub fn unit(&self) -> u16 {
        self.unit
    }

    /// # Returns
    ///
    /// The count hold by this message
    pub fn count(&self) -> u16 {
        self.count
    }

    /// # Returns
    ///
    /// This messages direction
    pub fn direction(&self) -> bool {
        self.direction
    }
}

/// Represents a report message
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum RepStructure {
    /// A Lissy IR report
    LissyIrReport(LissyIrReport),
    /// A rfid5 report
    RFID5Report(RFID5Report),
    /// A rfid7 report
    RFID7Report(RFID7Report),
    /// A wheel count report
    WheelcntReport(WheelcntReport),
}

impl RepStructure {
    /// Parses a report message from the given bytes
    ///
    /// # Parameters
    ///
    /// - `count`: The messages length
    /// - `args`: The messages arguments to parse
    pub(crate) fn parse(count: u8, args: &[u8]) -> Result<Self, MessageParseError> {
        if args[0] == 0x00 {
            if count != 0x08 {
                Err(MessageParseError::UnexpectedEnd(0xE4))
            } else {
                Ok(Self::LissyIrReport(LissyIrReport::parse(
                    args[0], args[1], args[2], args[3], args[4],
                )))
            }
        } else if args[0] == 0x40 {
            if count != 0x08 {
                Err(MessageParseError::UnexpectedEnd(0xE4))
            } else {
                Ok(Self::WheelcntReport(WheelcntReport::parse(
                    args[0], args[1], args[2], args[3], args[4],
                )))
            }
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
            Err(MessageParseError::InvalidFormat(
                "The report message (opcode: 0xE4) was in invalid format!".into(),
            ))
        }
    }
}

/// The destination slot to move data to
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DstArg(u16);

impl DstArg {
    /// Creates a new destination slot
    ///
    /// # Parameters
    ///
    /// - `dst`: The destination
    pub fn new(dst: u16) -> Self {
        DstArg(dst)
    }

    /// Parses the destination from two bytes
    ///
    /// # Parameters
    ///
    /// - `dst_low`: The seven least significant destination address bytes
    /// - `dst_high`: The seven most significant destination address bytes
    pub(crate) fn parse(dst_low: u8, dst_high: u8) -> Self {
        let dst = ((dst_high as u16) << 7) | (dst_low as u16);
        DstArg(dst)
    }

    /// # Returns
    ///
    /// The destination address of the slot move
    pub fn dst(&self) -> u16 {
        self.0
    }

    /// # Returns
    ///
    /// The seven least significant destination address bits
    pub(crate) fn dst_low(&self) -> u8 {
        self.0 as u8 & 0x7F
    }

    /// # Returns
    ///
    /// The seven most significant destination address bits
    pub(crate) fn dst_high(&self) -> u8 {
        (self.0 >> 7) as u8 & 0x7F
    }
}

/// Holds eight movable bytes and peer data
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
    /// Creates new peer data
    ///
    /// # Parameters
    ///
    /// - `pxc`: The peer data
    /// - `d1` - `d8`: The data
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

    /// Parses the data from 10 bytes
    ///
    /// # Parameters
    ///
    /// - `pxct1`, `pxct2`: The peer data
    /// - `d1` - `d8`: The data
    pub(crate) fn parse(
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

    /// # Returns
    ///
    /// The peer data
    pub fn pxc(&self) -> u8 {
        self.pxc
    }

    /// # Returns
    ///
    /// The low part of the peer data and one data bit of the first four the data bits
    pub(crate) fn pxct1(&self) -> u8 {
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

    /// # Returns
    ///
    /// The high part of the peer data and one data bit of the last four the data bits
    pub(crate) fn pxct2(&self) -> u8 {
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

    /// # Returns
    ///
    /// The first data byte to move
    pub fn d1(&self) -> u8 {
        self.d1 & 0x3F
    }

    /// # Returns
    ///
    /// The second data byte to move
    pub fn d2(&self) -> u8 {
        self.d2 & 0x3F
    }

    /// # Returns
    ///
    /// The third data byte to move
    pub fn d3(&self) -> u8 {
        self.d3 & 0x3F
    }

    /// # Returns
    ///
    /// The fourth data byte to move
    pub fn d4(&self) -> u8 {
        self.d4 & 0x3F
    }

    /// # Returns
    ///
    /// The fifth data byte to move
    pub fn d5(&self) -> u8 {
        self.d5 & 0x3F
    }

    /// # Returns
    ///
    /// The sixth data byte to move
    pub fn d6(&self) -> u8 {
        self.d6 & 0x3F
    }

    /// # Returns
    ///
    /// The seventh data byte to move
    pub fn d7(&self) -> u8 {
        self.d7 & 0x3F
    }

    /// # Returns
    ///
    /// The eighth data byte to move
    pub fn d8(&self) -> u8 {
        self.d8 & 0x3F
    }
}

/// Send when service mode is aborted
///
/// As I do not now how this message is structured this message bytes is for now open to use.
/// Please feel free to contribute to provide a more powerful version of this arg
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ProgrammingAbortedArg {
    /// The count of args to write to the message 0x10 or 0x15
    pub arg_len: u8,
    /// The first argument
    pub arg01: u8,
    /// The second argument
    pub arg02: u8,
    /// The third argument
    pub arg03: u8,
    /// The fourth argument
    pub arg04: u8,
    /// The fifth argument
    pub arg05: u8,
    /// The sixth argument
    pub arg06: u8,
    /// The seventh argument
    pub arg07: u8,
    /// The eighth argument
    pub arg08: u8,
    /// The ninth argument
    pub arg09: u8,
    /// The tenth argument
    pub arg10: u8,
    /// The eleventh argument
    pub arg11: u8,
    /// The twelfth argument
    pub arg12: u8,
    /// The thirteenth argument
    pub arg13: u8,
    /// The fourteenth argument
    pub arg14: u8,
    /// The fifteenth argument
    pub arg15: u8,
    /// The sixteenth argument
    pub arg16: u8,
    /// The seventeenth argument
    pub arg17: u8,
    /// The eighteenth argument
    pub arg18: u8,
}

impl ProgrammingAbortedArg {
    /// Creates a new service mode aborted message.
    ///
    /// # Parameters
    ///
    /// - `len`: The messages length (0x10 or 0x15)
    /// - `args`: The argument values. 0x10 = 0 - 12 filled, 0x15 = 0 - 17 filled
    pub fn new(len: u8, args: &[u8]) -> Self {
        ProgrammingAbortedArg::parse(len, args)
    }

    /// Parses a new service mode aborted message.
    ///
    /// # Parameters
    ///
    /// - `len`: The messages length (0x10 or 0x15)
    /// - `args`: The argument values. 0x10 = 0 - 12 filled, 0x15 = 0 - 17 filled
    pub(crate) fn parse(len: u8, args: &[u8]) -> Self {
        match len {
            0x10 => ProgrammingAbortedArg {
                arg_len: len,
                arg01: args[0],
                arg02: args[1],
                arg03: args[2],
                arg04: args[3],
                arg05: args[4],
                arg06: args[5],
                arg07: args[6],
                arg08: args[7],
                arg09: args[8],
                arg10: args[9],
                arg11: args[10],
                arg12: args[11],
                arg13: args[12],
                arg14: 0,
                arg15: 0,
                arg16: 0,
                arg17: 0,
                arg18: 0,
            },

            0x15 => ProgrammingAbortedArg {
                arg_len: len,
                arg01: args[0],
                arg02: args[1],
                arg03: args[2],
                arg04: args[3],
                arg05: args[4],
                arg06: args[5],
                arg07: args[6],
                arg08: args[7],
                arg09: args[8],
                arg10: args[9],
                arg11: args[10],
                arg12: args[11],
                arg13: args[12],
                arg14: args[13],
                arg15: args[14],
                arg16: args[15],
                arg17: args[16],
                arg18: args[17],
            },
            _ => ProgrammingAbortedArg {
                arg_len: len,
                arg01: *args.first().unwrap_or(&0u8),
                arg02: *args.get(1).unwrap_or(&0u8),
                arg03: *args.get(2).unwrap_or(&0u8),
                arg04: *args.get(3).unwrap_or(&0u8),
                arg05: *args.get(4).unwrap_or(&0u8),
                arg06: *args.get(5).unwrap_or(&0u8),
                arg07: *args.get(6).unwrap_or(&0u8),
                arg08: *args.get(7).unwrap_or(&0u8),
                arg09: *args.get(8).unwrap_or(&0u8),
                arg10: *args.get(9).unwrap_or(&0u8),
                arg11: *args.get(10).unwrap_or(&0u8),
                arg12: *args.get(11).unwrap_or(&0u8),
                arg13: *args.get(12).unwrap_or(&0u8),
                arg14: *args.get(13).unwrap_or(&0u8),
                arg15: *args.get(14).unwrap_or(&0u8),
                arg16: *args.get(15).unwrap_or(&0u8),
                arg17: *args.get(16).unwrap_or(&0u8),
                arg18: *args.get(17).unwrap_or(&0u8),
            },
        }
    }

    /// # Returns
    ///
    /// This message as a count of bytes
    pub(crate) fn to_message(self) -> Vec<u8> {
        match self.arg_len {
            0x10 => vec![
                0xE6, 0x10, self.arg01, self.arg02, self.arg03, self.arg04, self.arg05, self.arg06,
                self.arg07, self.arg08, self.arg09, self.arg10, self.arg11, self.arg12, self.arg13,
            ],
            _ => vec![
                0xE6, 0x15, self.arg01, self.arg02, self.arg03, self.arg04, self.arg05, self.arg06,
                self.arg07, self.arg08, self.arg09, self.arg10, self.arg11, self.arg12, self.arg13,
                self.arg14, self.arg15, self.arg16, self.arg17, self.arg18,
            ],
        }
    }
}
