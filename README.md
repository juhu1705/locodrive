# LocoDrive [![Rust](https://github.com/juhu1705/locodrive-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/juhu1705/locodrive-rust/actions/workflows/rust.yml)[![Doc](https://github.com/juhu1705/Locodrive/actions/workflows/codeql-release.yml/badge.svg)](https://github.com/juhu1705/Locodrive/actions/workflows/codeql-release.yml)

A rust library for controlling a model train railroad based on the `LocoNet` system.

## Features
| Feature                          | Description                                                                                      | Status |
|----------------------------------|--------------------------------------------------------------------------------------------------|--------|
| Sending control                  | Control of sending messages to the `LocoNet`                                                     | DONE   |
| Receiving control                | Possibility to handle received messages                                                          | DONE   |
| Configuration of the connection  | Control over the configuration settings of the `LocoNet` connection like BaudRate or FlowControl | DONE   |

## Importing the LocoDrive

As rust is able to use GitHub repositories directly as dependencies you can simply add 
`locodrive = { git = "https://github.com/juhu1705/locodrive-rust.git" }` to your `Cargo.toml`

## Using the LocoDrive

The LocoDrive has the struct `loco_controller::LocoNetController` made for connecting to a LocoNet over a serial port.
This reader will care of parsing received messages correctly before sending them to you.

## Documentation

The documentation is published [here](https://juhu1705.github.io/locodrive-rust)

## Committing to the LocoDrive

### Setting up the project

To set up the project yourself please make sure to have rust installed.

### Commitment rules

To commit to this repository please consider the Contributing rules.

Please note: Always add me to your pull request to test your changes with an active `LocoNet` connection 
or add some test logs to your commitment.

## Used Dependencies

### Rust

| Dependency   | License |
|--------------|---------|
| tokio-serial | MIT     |
| tokio-util   | MIT     |
| bytes        | MIT     |
| tokio        | MIT     |

### Loco Net information

For getting the needed information about the loco net protocol I mostly used the [rocrail wiki](https://wiki.rocrail.net/doku.php?id=loconet:ln-pe-en). Thanks for the detailed information.