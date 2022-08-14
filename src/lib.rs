/// Holds all arguments used in `LocoNet` message
pub mod args;
/// Holds all error messages that may occur
pub mod error;
/// Holds a [`loco_controller::LocoNetController`] to manage communication to a serial port based `LocoNet`
pub mod loco_controller;
/// Holds the [`protocol::Message`]s that can be send to and received from the `LocoNet`.
pub mod protocol;
/// Holds test on the `LocoNet`
mod tests;