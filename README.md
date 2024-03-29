# LocoDrive ![https://github.com/juhu1705/locodrive/blob/main/LICENSE-APACHE](https://img.shields.io/badge/license-Apache-blue.svg)![https://github.com/juhu1705/locodrive/blob/main/LICENSE-MIT](https://img.shields.io/badge/license-MIT-blue.svg)[![Tests](https://github.com/juhu1705/locodrive/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/juhu1705/locodrive/actions/workflows/test.yml)[![Docs](https://github.com/juhu1705/locodrive/actions/workflows/doc.yml/badge.svg?branch=main)](https://github.com/juhu1705/locodrive/actions/workflows/doc.yml)[![rust-clippy analyze](https://github.com/juhu1705/locodrive/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/juhu1705/locodrive/actions/workflows/rust-clippy.yml)

A rust library for controlling a model train railroad system.

## Features
| Feature                          | Description                                                                                           | Status |
|----------------------------------|-------------------------------------------------------------------------------------------------------|--------|
| Sending control                  | Control of sending messages to the model railroad                                                     | DONE   |
| Receiving control                | Possibility to handle received messages                                                               | DONE   |
| Configuration of the connection  | Control over the configuration settings of the model railroad connection like BaudRate or FlowControl | DONE   |

## Importing the LocoDrive

As rust is able to use GitHub repositories directly as dependencies you can simply add 
`locodrive = "0.1.2"` to your `Cargo.toml`

### Features

- `control`: The control feature allows you to access the `LocoDriveController`. This struct allows you to read and write messages to a specified serial port on your device. 
             Therefore, the async runtime `tokio`, with the extras `tokio-serial` and `tokio-util` as well as the `bytes` module are needed. Please read the documentation for more information about how to use the LocoDriveController.

## Using the LocoDrive

The LocoDrive has the struct `loco_controller::LocoDriveController` made for connecting to a model railroad over a serial port.
This reader will care of parsing received messages correctly before sending them to you.

## Documentation

The documentation is published [here](https://juhu1705.github.io/locodrive/doc/locodrive)

## Committing to the LocoDrive

### Setting up the project

To set up the project yourself please make sure to have rust installed.

### Commitment rules

To commit to this repository please consider the Contributing rules.

Please note: Always add me to your pull request to test your changes with an active model railroad connection 
or add some test logs to your commitment.

## Used Dependencies

### Rust

| Dependency   | License |
|--------------|---------|
| tokio-serial | MIT     |
| tokio-util   | MIT     |
| bytes        | MIT     |
| tokio        | MIT     |

### Protocol information

For getting the needed information about the used protocol I mostly used the [rocrail wiki](https://wiki.rocrail.net/doku.php?id=loconet:ln-pe-en). Thanks for the detailed information.
