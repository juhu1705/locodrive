use std::sync::{Arc, Condvar, Mutex};
use std::{mem, thread};
use std::ffi::OsStr;
use std::io::Read;
use std::thread::JoinHandle;
use std::time::Duration;
use serialport::{DataBits, FlowControl, Parity, StopBits, SerialPort, Error};
use crate::error::MessageParseError;
use crate::protocol::Message;

pub struct LocoNetConnector {
    port: Box<dyn SerialPort>,
    lack: bool,
    last_message: Message,
    send: Arc<(Mutex<Vec<u8>>, Condvar, Condvar)>,
    stop: Mutex<bool>,
    reading_thread: Option<JoinHandle<()>>,
    sending_timeout: u64
}

impl LocoNetConnector {
    /// Creates a new port
    pub fn new(port_name: &str, baud_rate: u32, sending_timeout: u64, update_cycles: u64, flow_control: FlowControl) -> Result<Self, Error> {
        let port = match serialport::new(port_name, baud_rate)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::Two)
            .parity(Parity::None)
            .flow_control(flow_control)
            .timeout(Duration::from_millis(update_cycles))
            .open() {
                Ok(port) => port,
                Err(e) => return Err(e)
        };

        Ok(LocoNetConnector {
            port,
            lack: false,
            last_message: Message::Busy,
            send: Arc::new((Mutex::new(vec![0u8; 0]), Condvar::new(), Condvar::new())),
            stop: Mutex::new(false),
            reading_thread: None,
            sending_timeout,
        })
    }

    /// Start a new thread that reads new loco net message
    pub fn start_reader(&'static mut self) -> bool {
        let s = Arc::new(Mutex::new(self));
        let new_s = Arc::clone(&s);
        let mut save_to = (&*s).lock().unwrap();
        if save_to.reading_thread.is_none() {
            save_to.reading_thread = Some(
                thread::spawn(move|| {
                    let mut save_inner = (&*new_s).lock().unwrap();
                    *save_inner.stop.lock().unwrap() = false;
                    while !*save_inner.stop.lock().unwrap() {
                        (*save_inner).read();
                    }
                })
            );
            true
        } else {
            false
        }
    }

    /// Stops the loco net message reader and wait for the stop
    pub fn stop_reader(&mut self) {
        if self.reading_thread.is_some() {
            *self.stop.lock().unwrap() = true;
            mem::replace(&mut self.reading_thread, None).take().unwrap().join().unwrap();
        }
    }

    /// Handels a message after a it was parsed successfully
    pub fn read(&mut self) {
        let parsed = self.parse();

        match parsed {
            Err(MessageParseError::Update) => {},
            Err(_) => {
                // TODO: Handle error occurrence
                self.lack = false;
            }
            Ok(message) => {
                if self.lack {
                    if let Message::LongAck(_, _) = self.last_message {
                        // TODO: handle long acknowledgment occurrence
                    }
                }

                if message.lack_follows() {
                    self.lack = true;
                    self.last_message = message;
                } else {
                    self.lack = false;
                }

                // TODO: handle message occurrence
            }
        }
    }

    pub fn parse(&mut self) -> Result<Message, MessageParseError> {
        let mut buf = vec![0u8; 1];

        let opc = match self.port.read_exact(&mut buf) {
            Ok(_) => buf[0],
            Err(_) => return Err(MessageParseError::Update)
        };

        let len = match opc & 0xE0 {
            0x80 => 2,
            0xA0 => 4,
            0xC0 => 6,
            0xE0 => {
                let mut read_len = [0u8; 1];
                match self.port.read_exact(&mut read_len) {
                    Ok(_) => {
                        buf.push(read_len[0]);
                        read_len[0] as usize - 1
                    },
                    Err(_) => return Err(MessageParseError::UnexpectedEnd)
                }
            }
            _ => return Err(MessageParseError::UnknownOpcode(opc)),
        };

        let mut message = vec![0u8; len - 1];

        buf.append(match self.port.read_exact(&mut message) {
            Ok(_) => &mut message,
            Err(_) => return Err(MessageParseError::UnexpectedEnd),
        });

        // Check for receiving last send message
        let (lock, cvar, waiter) = &*self.send;
        let mut last_send = lock.lock().unwrap();

        if !(*last_send).is_empty() && (*last_send) == buf {
            *last_send = vec![0u8; 0];
            waiter.notify_all();
            cvar.notify_one();
        }

        Message::parse(buf.as_slice(), opc, len)
    }

    /// Writes a set of bytes to the loco net by appending the checksum and sending it to the connection
    pub fn write(&mut self, message: Message) -> bool {
        if self.reading_thread.is_none() {
            return false;
        }

        let bytes = Self::append_checksum(message.to_message());

        let (lock, cvar, waiter) = &*self.send;

        if !(*lock.lock().unwrap()).is_empty() {
            let result = cvar.wait_timeout_while(
                lock.lock().unwrap(),
                Duration::from_millis(self.sending_timeout),
                |pending| !(*pending).is_empty())
                .unwrap();

            if result.1.timed_out() {
                return false;
            }
        }

        let mut send = lock.lock().unwrap();

        *send = bytes;

        match self.port.write_all(&*send) {
            Ok(_) => {
                drop(send);
                if !(*lock.lock().unwrap()).is_empty() {
                    let result = waiter.wait_timeout_while(lock.lock().unwrap(), Duration::from_millis(self.sending_timeout), |pending| !(*pending).is_empty()).unwrap();
                    if result.1.timed_out() {
                        return false;
                    }
                }
                true
            },
            Err(_) => false,
        }
    }

    /// Appends the checksum at the end of the message
    pub fn append_checksum(mut bytes: Vec<u8>) -> Vec<u8> {
        bytes.push(Self::check_sum(&bytes));

        bytes
    }

    /// Calculates the checksum for an array of bytes
    fn check_sum(msg: &[u8]) -> u8 {
        0xFF - msg.iter().fold(0, |acc, &b| acc ^ b)
    }
}