use std::fmt::Debug;
use crate::error::{LocoDriveSendingError, MessageParseError};
use crate::protocol::Message;
use std::sync::{Arc, Condvar, Mutex};
use tokio::time::{sleep, Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;
use tokio::sync::Notify;
use tokio_serial::{DataBits, Error, FlowControl, Parity, SerialPort, SerialPortBuilderExt, SerialStream, StopBits};

/// This message is sent when data are received from the loco connection.
#[derive(Debug, Clone)]
pub enum LocoDriveMessage {
    /// A normal loco connection message. Consider that all [`LocoDriveMessage::Answer`] messages are also send this way.
    Message(Message),
    /// This is a response for the before received message.
    /// The response message is represent by the first argument and
    /// the before received message is represented the second argument.
    /// Consider that the here mentioned received message is also send as normal [`LocoDriveMessage::Message`] afterwards.
    Answer(Message, Message),
    /// This message is send when the by the LocoDrive received message is not readable.
    /// Please look at [`MessageParseError`] for more information on the errors.
    Error(MessageParseError),
    /// This message is send when some error appears on opening the serial port.
    SerialPortError(Error),
}

type SendSynchronisation = Arc<(Arc<Mutex<Vec<u8>>>, Arc<Notify>)>;
type ReferencedSendSynchronisation<'a> = Arc<(&'a Arc<Mutex<Vec<u8>>>, &'a Arc<Notify>)>;

/// This struct handles a connection to a serial port based railroad controlling system.
///
/// All received messages on the port are send to the defined channel.
/// - Note: The auto returned messages as defined in the model railroads protocol are also send back to the channel.
/// But the protocol ensures itself that the writer waits until the model railroad response is received.
///
/// # Usage
///
/// To send a message see [`LocoDriveController::send_message()`].
/// The reading thread is start automatically on port creation.
/// You can just check on your reader channel for new messages.
/// The reader is automatically dropped when the [`LocoDriveController`] is dropped.
///
/// # Examples
///
/// Reading ten messages received from the model railroads:
/// ```
/// # use tokio_serial::FlowControl;
/// # use locodrive::loco_controller::LocoDriveController;
/// # use locodrive::protocol::Message;
/// # use locodrive::args::{SlotArg, SpeedArg};
/// # use locodrive::protocol::Message::LocoSpd;
///
/// #[tokio::main]
/// async fn main() {
///     // Creating a sender and receiver for the LocoDriveConnector.
///     let (sender, mut receiver) = tokio::sync::broadcast::channel(1);
///
///     // Creating a LocoDriveConnector, reading from the port '/dev/ttyUSB0'.
///     let mut loco_controller = match LocoDriveController::new(
///         "/dev/ttyUSB0",
///         115_200,
///         5000,
///         FlowControl::Software,
///         sender,
///         false,
///     ).await {
///         Ok(loco_controller) => loco_controller,
///         Err(err) => {
///             eprintln!("Error: Could not connect to the serial port!");
///             return;
///         }
///     };
///
///     loco_controller.send_message(LocoSpd(SlotArg::new(3), SpeedArg::Stop));
///
///     let mut read_messages = 0;
///     while let Ok(message) = receiver.recv().await {
///         println!("GOT = {:?}", message);
///         read_messages += 1;
///         if read_messages >= 10 {
///            break;
///         }
///     }
/// }
/// ```
pub struct LocoDriveController {
    /// The serial port used to connect to the model railroads.
    port: SerialStream,
    /// Here are all values bundled for the intern check of the message sending and receiving.
    /// The Mutex is used to save the last message send to check against.
    /// The two Condvar args are used synchronize the writer.
    send: SendSynchronisation,
    /// This is used to call the reader to stop reading.
    stop: Arc<Mutex<bool>>,
    /// Fire stop to notify the reader to recheck if it should stop
    fire_stop: Arc<Notify>,
    /// This is the thread to await for joining if one reading thread should be closed.
    reading_thread: Option<JoinHandle<()>>,
    /// How long to wait on success of sending.
    sending_timeout: u64,
    /// Securing one writing thread at a time
    wait_for_write: Arc<tokio::sync::Mutex<bool>>,
}

impl LocoDriveController {
    /// Creates a new serial port connection to a model railroad and starts reading on that port
    ///
    /// # Parameter
    ///
    /// - `port_name`: Is the name of the port to connect to.
    ///   If you are not sure, which ports are allowed use [`tokio_serial::available_ports()`](https://docs.rs/tokio-serial/latest/tokio_serial/fn.available_ports.html).
    /// - `baud_rate`: The baud rate to use for the port connection.
    /// - `sending_timeout`: How long to wait for response for the model railroads connection
    ///   while sending messages.
    /// - `update_cycles`: How long to wait for incoming messages on reader side,
    ///   before checking if this reader should close.
    /// - `flow_control`: Which mode of flow control to use for this port.
    ///   It is recommended to use [`FlowControl::Software`](https://docs.rs/tokio-serial/latest/tokio_serial/enum.FlowControl.html).
    ///
    /// # Error
    ///
    /// This method exit with an error if the serial port is not reachable or the port could
    /// not be configured correctly.
    ///
    /// # Reading
    ///
    /// - If this thread communicates a [`LocoDriveMessage::SerialPortError`] it will
    ///   not be receiving any more messages. All other Messages would not lead
    ///   the reading thread to stop.
    /// - Lack messages are send twice. Ones as [`LocoDriveMessage::Answer`] and
    ///   then a second time as [`LocoDriveMessage::Message`].
    pub async fn new(
        port_name: &str,
        baud_rate: u32,
        sending_timeout: u64,
        flow_control: FlowControl,
        send_to: Sender<LocoDriveMessage>,
        ignore_send_messages: bool,
    ) -> Result<Self, Error> {
        // Creation of the port to write to
        let mut port = match tokio_serial::new(port_name, baud_rate)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::Two)
            .parity(Parity::None)
            .flow_control(flow_control)
            .timeout(Duration::from_millis(sending_timeout))
            .open_native_async()
        {
            Ok(port) => port,
            Err(e) => return Err(e),
        };

        // For unix systems we must ensure the port to be available
        // for parallel opening by the reading thread.
        #[cfg(unix)]
        port.set_exclusive(false)?;

        // Takes care of the writer reader synchronisation
        let send = Arc::new((
            Arc::new(Mutex::new(vec![0u8; 0])),
            Arc::new(Notify::new()),
        ));

        // Used to stop a reader when the the value was dropped
        let stop = Arc::new(Mutex::new(false));
        let fire_stop = Arc::new(Notify::new());

        // Starts the reading thread
        let reading_thread = Some(LocoDriveController::start_reading_thread(
            port_name.to_string(),
            baud_rate,
            flow_control,
            &send,
            &send_to,
            &stop,
            &fire_stop,
            ignore_send_messages
        ).await);

        let wait_for_write = Arc::new(tokio::sync::Mutex::new(false));

        // All steps has passed successfully
        Ok(LocoDriveController {
            port,
            send,
            stop,
            fire_stop,
            reading_thread,
            sending_timeout,
            wait_for_write,
        })
    }

    /// # Return
    ///
    /// The port the `LocoDriveConnector` is connected to.
    pub fn get_port_name(&self) -> Option<String> {
        self.port.name()
    }

    /// # Return
    ///
    /// The connected ports baud rate.
    pub fn get_baud_rate(&self) -> tokio_serial::Result<u32> {
        self.port.baud_rate()
    }

    /// # Return
    ///
    /// The maximum time to wait for a message to be send correctly.
    pub fn get_sending_timeout(&self) -> u64 {
        self.sending_timeout
    }

    /// Overrides the sending timeout with the give value.
    ///
    /// # Parameter
    ///
    /// - `sending_timeout`: The time to wait for a reading action to complete.
    ///
    /// # Returns
    ///
    /// If some error occurred on overriding the timeout on the port.
    pub fn set_sending_timeout(&mut self, sending_timeout: u64) -> Result<(), Error> {
        self.sending_timeout = sending_timeout;
        self.port.set_timeout(Duration::from_millis(sending_timeout))
    }

    /// Stops the async model railroads message reader and wait until the tokio thread is joined.
    ///
    /// If no thread is opened the function returns immediately.
    ///
    /// # Panics
    ///
    /// This function panics if the reading thread has panicked or the reading thread was killed,
    /// by some external source.
    async fn stop_reader(&mut self) {
        if let Some(reader) = self.reading_thread.take() {
            // Note the thread to end reading
            *self.stop.lock().unwrap() = true;
            self.fire_stop.notify_waiters();
            // Wait until the thread is stopped
            reader.await.unwrap();

            // We allow new threads to spawn and read from the port
            *self.stop.lock().unwrap() = false;
        }
    }

    /// Helper method that spawns a new async tokio thread for reading model railroads
    /// messages from the specified serial port.
    ///
    /// # Parameter
    ///
    /// - `port_name`: The name of the serial port to read from
    /// - `baud_rate`: The baud rate to use
    /// - `flow_control`: The used [`FlowControl`]
    /// - `send`: The information to free the writer when rechecking that the message is received by the model railroad
    /// - `send_to`: Where to send the received and parsed model railroad messages
    /// - `wait_to`: A mutex indicates this thread to stop.
    /// - `stopping`: A notify used to awake the reading thread from waiting for new incoming messages
    ///
    /// # Returns
    ///
    /// The spawned threads join handle.
    #[allow(clippy::too_many_arguments)]
    async fn start_reading_thread(
        port_name: String,
        baud_rate: u32,
        flow_control: FlowControl,
        send: &SendSynchronisation,
        send_to: &Sender<LocoDriveMessage>,
        wait_to: &Arc<Mutex<bool>>,
        stopping: &Arc<Notify>,
        ignore_send_messages: bool,
    ) -> JoinHandle<()> {
        // Clone all arcs to make them save to use in the reading thread
        let arc_send_to = send_to.clone();

        let last_message = &send.0;
        let notify_wait = &send.1;

        let last_message_move = last_message.clone();
        let notify_wait_move = notify_wait.clone();

        let new_arc_wait_to = wait_to.clone();
        let new_arc_stopping = stopping.clone();

        tokio::spawn(async move {
            // Connects the port to read from
            let mut port = match tokio_serial::new(port_name, baud_rate)
                .data_bits(DataBits::Eight)
                .stop_bits(StopBits::Two)
                .parity(Parity::None)
                .flow_control(flow_control)
                .open_native_async()
            {
                Ok(port) => port,
                Err(err) => {
                    if let Err(err) = arc_send_to.send(LocoDriveMessage::SerialPortError(err)) {
                        eprintln!("[locodrive:ERROR] Unable to send critical error to receiver! \
                        Closed connection to the serial port!\n \
                        Following error occurred: {:?}", err);
                    }
                    return;
                },
            };

            // For linux systems we once more ensure that this set is not exclusive usable for us
            #[cfg(unix)]
            if let Err(err) = port.set_exclusive(false) {
                if let Err(err) = arc_send_to.send(LocoDriveMessage::SerialPortError(err)) {
                    eprintln!("[locodrive:ERROR] Unable to send critical error to receiver! \
                    Closed connection to the serial port!\n \
                    Following error occurred: {:?}", err);
                };
                return;
            };

            // The lack indicates the last message to await a model railroads response
            let mut lack = false;
            // The last message to pass when a lack was received
            let mut last_message = Message::Busy;

            let new_arc_send_locked =
                Arc::new((&last_message_move, &notify_wait_move));

            println!("[locodrive:INFO] Reading thread started!");

            // This thread reads till it is notified to stop
            while !*new_arc_wait_to.lock().unwrap() {
                // We read and directly handle received messages
                LocoDriveController::handle_next_message(
                    &mut port,
                    &new_arc_send_locked,
                    &mut lack,
                    &mut last_message,
                    &arc_send_to,
                    &new_arc_stopping,
                    ignore_send_messages
                )
                .await;
            }

            println!("[locodrive:INFO] Reading thread closed!");
        })
    }

    /// Handles a model railroad message after it was parsed successfully.
    ///
    /// # Parameter
    ///
    /// - `port`: The port to read messages from
    /// - `send`: The information to free the writer when rechecking that the message is received by the model railroad
    /// - `lack`: Whether the last received message expects a lack to follow
    /// - `last_message`: The previous received message
    /// - `send_to`: Where to send the received and parsed model railroad messages
    /// - `stopping`: A notify used to awake the reading thread from waiting for new incoming messages
    async fn handle_next_message<'a>(
        port: &mut SerialStream,
        send: &ReferencedSendSynchronisation<'a>,
        await_response: &mut bool,
        last_message: &mut Message,
        send_to: &Sender<LocoDriveMessage>,
        stopping: &Arc<Notify>,
        ignore_send_messages: bool,
    ) {
        // We read the next message from the serial port
        let parsed = LocoDriveController::read_next_message(port, send, stopping, ignore_send_messages).await;

        // We check which type the message we received is
        match parsed {
            // We can at this level ignore update messages
            Err(MessageParseError::Update) => {}
            // For errors we only give them to our listener and if this fails we print them
            Err(err) => {
                if let Err(err) = send_to.send(LocoDriveMessage::Error(err)) {
                    eprintln!("[locodrive:ERROR] {:?}", err);
                };
                *await_response = false;
            }
            Ok(message) => {
                // If our last received message expects a response message to follow, we check
                // for this response message to be received
                if *await_response {
                    match message {
                        Message::LongAck(lopc, _) => {
                            if lopc.check_opc(last_message) {
                                // We notify our listener of that long acknowledgment
                                if let Err(err) = send_to.send(
                                    LocoDriveMessage::Answer(message, *last_message)
                                ) {
                                    eprintln!("[locodrive:ERROR] {:?}", err);
                                };
                            }
                        }
                        Message::SlRdData(..) => {
                            if last_message.await_slot_data() {
                                if let Err(err) = send_to.send(
                                    LocoDriveMessage::Answer(message, *last_message)
                                ) {
                                    eprintln!("[locodrive:ERROR] {:?}", err);
                                };
                            }
                        }
                        _ => {}
                    }
                }

                // Checks whether our message is followed by an acknowledgment
                if message.answer_follows() {
                    *await_response = true;
                    *last_message = message;
                } else if Message::Busy != message {
                    *await_response = false;
                }

                // We at least notify our listener about the received message
                if let Err(err) = send_to.send(LocoDriveMessage::Message(message)) {
                    eprintln!("[locodrive:ERROR] {:?}", err);
                }
            }
        }
    }

    /// Waits for the next model railroad message and reads that message from a given serial port.
    ///
    /// # Parameter
    ///
    /// - `port`: The serial port to read the message from
    /// - `send`: Used to notify the writer that the model railroad has successfully received the send message
    /// - `stopping`: This is used to notify this thread to awake from waiting at new messages
    ///
    /// # Return
    ///
    /// [`Message`]: If a model railroad message was read from the port
    /// [`MessageParseError`]: If there occurred some error while parsing the message
    /// [`MessageParseError::Update`]: If a notification was send over `stopping` to awake
    ///
    /// # Note
    ///
    /// This method sleeps until a message was received as long as the maximum timeout is set.
    async fn read_next_message<'a>(
        port: &mut SerialStream,
        send: &ReferencedSendSynchronisation<'a>,
        stopping: &Arc<Notify>,
        ignore_send_messages: bool,
    ) -> Result<Message, MessageParseError> {
        // The buffer we want to read the model railroads message to
        let mut buf = vec![0u8; 1];

        // We wait for a messages op code to be received or to a wakeup by a notification
        let opc = tokio::select! {
            opc = port.read_exact(&mut buf) => match opc {
                Ok(_) => buf[0],
                Err(_) => return Err(MessageParseError::UnexpectedEnd),
            },
            _ = stopping.notified() => {
                return Err(MessageParseError::Update)
            }
        };

        // We calculate the length of the remaining message to read
        let len = match opc & 0xE0 {
            0x80 => 2,
            0xA0 => 4,
            0xC0 => 6,
            0xE0 => {
                // The code 0xE0 indicates that the second byte of the message is used to display
                // the messages length so we read that second byte.
                let mut read_len = [0u8; 1];
                match port.read_exact(&mut read_len).await {
                    Ok(_) => {
                        buf.push(read_len[0]);
                        // We already read the messages first byte
                        read_len[0] as usize - 1
                    }
                    Err(_) => return Err(MessageParseError::UnexpectedEnd),
                }
            }
            _ => return Err(MessageParseError::UnknownOpcode(opc)),
        };

        // As we already read the messages opcode
        let mut message = vec![0u8; len - 1];

        // We read the remaining message from the serial port
        buf.append(match port.read_exact(&mut message).await {
            Ok(_) => &mut message,
            Err(_) => return Err(MessageParseError::UnexpectedEnd),
        });

        // Check for receiving last send message to awake the writing thread
        let (lock, cvar) = **send;
        let mut last_send = lock.lock().unwrap();

        if !(*last_send).is_empty() && (*last_send) == buf {
            *last_send = vec![0u8; 0];
            cvar.notify_waiters();

            if ignore_send_messages {
                return Err(MessageParseError::Update)
            }
        }

        // We now parse the read bytes to our message
        Message::parse(buf.as_slice())
    }

    /// Sends a Message to the model railroad.
    ///
    /// # Parameter
    ///
    /// - `message`: The message to send to the model railroads serial port
    ///
    /// # Return
    ///
    /// If the message was successfully written nothing is returned else
    /// an [`LocoDriveSendingError`] describing the reason for the fail of the writing is returned.
    pub async fn send_message(&mut self, message: Message) -> Result<(), LocoDriveSendingError> {
        // If we have no reading thread we raise an error, that should not be possible
        if self.reading_thread.is_none() {
            return Err(LocoDriveSendingError::IllegalState)
        }

        let _send_message_waiting = self.wait_for_write.lock().await;

        // We parse the message to send in a byte vector
        let bytes = message.to_message();

        // We wait for possible other waiting operations to finish
        let (lock, notify) = &*self.send;

        {
            // We say the Reader which method to expect
            let mut send = lock.lock().unwrap();

            *send = bytes.clone();
        }

        // Write the message to the serial port
        match self.port.write_all(&bytes).await {
            Ok(_) => {
                // When successfully written, wait until the positive response
                // by the reading thread is received or raise an error
                if !(*lock.lock().unwrap()).is_empty() {
                    if tokio::select! {
                        _ = notify.notified() => false,
                        _ = sleep(Duration::from_millis(self.sending_timeout)) => true,
                    } {
                        return Err(LocoDriveSendingError::Timeout)
                    }
                }
                Ok(())
            }
            Err(_) => Err(LocoDriveSendingError::NotWritable),
        }
    }
}

/// Extends standard drop implementation to close the reading thread.
impl Drop for LocoDriveController {
    /// Handles drop Actions for the [`LocoDriveController`].
    ///
    /// In detail: We stop and join our reading thread on drop.
    ///
    /// # Panics
    ///
    /// The drop panics if the reading thread has panicked.
    fn drop(&mut self) {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(_) => { return; }
        };
        runtime.block_on(self.stop_reader());
    }
}
