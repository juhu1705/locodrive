use std::sync::{Arc, Condvar, Mutex};
use std::{mem};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_serial::{DataBits, FlowControl, Parity, StopBits, Error, SerialPortBuilderExt, SerialStream, SerialPort};
use crate::error::MessageParseError;
use crate::protocol::Message;

#[derive(Debug)]
pub enum LocoNetMessage {
    MESSAGE(Message),
    LACK(Message, Message),
    ERROR(MessageParseError)
}

pub struct LocoNetConnector {
    port: SerialStream,
    send: Arc<(Arc<Mutex<Vec<u8>>>, Arc<Condvar>, Arc<Condvar>)>,
    stop: Arc<Mutex<bool>>,
    reading_thread: Option<JoinHandle<()>>,
    sending_timeout: u64,
    send_to: Sender<LocoNetMessage>,
}

impl LocoNetConnector {
    /// Creates a new port
    pub fn new(port_name: &str, baud_rate: u32, sending_timeout: u64, update_cycles: u64, flow_control: FlowControl, send_to: Sender<LocoNetMessage>) -> Result<Self, Error> {
        let mut port = match tokio_serial::new(port_name, baud_rate)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::Two)
            .parity(Parity::None)
            .flow_control(flow_control)
            .timeout(Duration::from_millis(update_cycles))
            .open_native_async() {
                Ok(port) => port,
                Err(e) => return Err(e)
        };

        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");

        Ok(LocoNetConnector {
            port,
            send: Arc::new((Arc::new(Mutex::new(vec![0u8; 0])), Arc::new(Condvar::new()), Arc::new(Condvar::new()))),
            stop: Arc::new(Mutex::new(false)),
            reading_thread: None,
            sending_timeout,
            send_to,
        })
    }

    /// Start a new thread that reads new loco net message
    pub async fn start_reader(&mut self) -> bool {

        // let s = Arc::new(Mutex::new(self));
        // let new_s = Arc::clone(&s);

        // let mut save_to = self;

        let wait_to = &self.stop;
        let port = &self.port;

        let port_name = port.name().unwrap();
        let baud_rate = port.baud_rate().unwrap();
        let flow_control = port.flow_control().unwrap();
        let timeout = port.timeout();

        let send = &self.send;
        let send_to = &self.send_to;

        if self.reading_thread.is_none() {
            self.reading_thread = Some(
                LocoNetConnector::start_reading_thread(port_name, baud_rate, flow_control, timeout, send, send_to, wait_to).await
            );
            true
        } else {
            false
        }
    }

    pub async fn start_reading_thread(port_name: String,
                                      baud_rate: u32,
                                      flow_control: FlowControl,
                                      timeout: Duration,
                                      send: &Arc<(Arc<Mutex<Vec<u8>>>, Arc<Condvar>, Arc<Condvar>)>,
                                      send_to: &Sender<LocoNetMessage>,
                                      wait_to: &Arc<Mutex<bool>>) -> JoinHandle<()> {
        let arc_send_to = send_to.clone();

        let last_message = &send.0;
        let notify_wait = &send.1;
        let notify_received = &send.2;

        let last_message_move = last_message.clone();
        let notify_wait_move = notify_wait.clone();
        let notify_received_move = notify_received.clone();

        let new_arc_wait_to = wait_to.clone();

        tokio::spawn(async move {
            let mut port = match tokio_serial::new(port_name, baud_rate)
                .data_bits(DataBits::Eight)
                .stop_bits(StopBits::Two)
                .parity(Parity::None)
                .flow_control(flow_control)
                .timeout(timeout)
                .open_native_async() {
                Ok(port) => port,
                Err(_) => return ()
            };

            #[cfg(unix)]
            port.set_exclusive(false)
                .expect("Unable to set serial port exclusive to false");

            let mut lack = false;
            let mut last_message = Message::Busy;


            while *new_arc_wait_to.lock().unwrap() {
                let new_arc_send_locked = Arc::new((&last_message_move, &notify_wait_move, &notify_received_move));

                LocoNetConnector::read(&mut port,
                                       &new_arc_send_locked,
                                       &mut lack,
                                       &mut last_message,
                                       &arc_send_to).await;
            }
        })
    }

    /// Stops the loco net message reader and wait for the stop
    pub async fn stop_reader(&mut self) {
        if self.reading_thread.is_some() {
            *self.stop.lock().unwrap() = true;
            mem::replace(&mut self.reading_thread, None).take().unwrap().await.unwrap();
        }
    }

    /// Handels a message after a it was parsed successfully
    pub async fn read(port: &mut SerialStream,
                      send: &Arc<(&Arc<Mutex<Vec<u8>>>, &Arc<Condvar>, &Arc<Condvar>)>,
                      lack: &mut bool,
                      last_message: &mut Message,
                      send_to: &Sender<LocoNetMessage>) {
        let parsed = LocoNetConnector::parse(port, send).await;

        match parsed {
            Err(MessageParseError::Update) => {},
            Err(err) => {
                send_to.send(LocoNetMessage::ERROR(err)).await.unwrap();
                *lack = false;
            }
            Ok(message) => {
                if *lack {
                    if let Message::LongAck(_, _) = *last_message {
                        send_to.send(LocoNetMessage::LACK(*last_message, message)).await.unwrap();
                    }
                }

                if message.lack_follows() {
                    *lack = true;
                    *last_message = message;
                } else {
                    *lack = false;
                }

                send_to.send(LocoNetMessage::MESSAGE(message)).await.unwrap();
            }
        }
    }

    pub async fn parse(port: &mut SerialStream, send: &Arc<(&Arc<Mutex<Vec<u8>>>, &Arc<Condvar>, &Arc<Condvar>)>) -> Result<Message, MessageParseError> {
        let mut buf = vec![0u8; 1];

        let opc = match port.read_exact(&mut buf).await {
            Ok(_) => buf[0],
            Err(_) => return Err(MessageParseError::Update)
        };

        let len = match opc & 0xE0 {
            0x80 => 2,
            0xA0 => 4,
            0xC0 => 6,
            0xE0 => {
                let mut read_len = [0u8; 1];
                match port.read_exact(&mut read_len).await {
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

        buf.append(match port.read_exact(&mut message).await {
            Ok(_) => &mut message,
            Err(_) => return Err(MessageParseError::UnexpectedEnd),
        });

        // Check for receiving last send message
        let (lock, cvar, waiter) = **send;
        let mut last_send = lock.lock().unwrap();

        if !(*last_send).is_empty() && (*last_send) == buf {
            *last_send = vec![0u8; 0];
            waiter.notify_all();
            cvar.notify_one();
        }

        Message::parse(buf.as_slice(), opc, len)
    }

    /// Writes a set of bytes to the loco net by appending the checksum and sending it to the connection
    pub async fn write(&mut self, message: Message) -> bool {
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

        match self.port.write_all(&*send).await {
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