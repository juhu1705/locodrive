/// Holds all arguments used in the messages
pub mod args;
/// Holds all error messages that may occur
pub mod error;
/// Holds a [`loco_controller::LocoDriveController`] to manage communication to a serial port based model railroad system.
/// This modules is contained in the `control` feature. You have to explicitly activate it.
#[cfg(feature = "control")]
pub mod loco_controller;
/// Holds the [`protocol::Message`]s that can be send to and received from the model railroad system.
pub mod protocol;
/// Holds test for controlling the correctness of the implemented protocol
mod tests;
