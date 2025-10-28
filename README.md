# Pyrion ESC

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.html)

Firmware for Pyrion, an open-source ESC for BLDC motors, written in Rust.

Currently in alpha, targeting STM32G4 boards.

---

## Project Structure

This project consists of two main crates:

### Firmware (`crates/firmware`)

Embedded firmware for STM32G4 microcontrollers that run on the ESC.

### Server (`crates/server`)

A host-side server application that communicates with the ESC hardware. It has a gRPC-based interface for device
discovery and communication. Check out the [proto definitions](https://github.com/UnoPromilo/pyrion-proto) for more
information.

---

## Running the Server

1. Navigate to the server crate:
   ```bash
   cd crates/server
   ```

2. Optionally configure the server by editing `configuration.yaml` to match your setup.

3. Run the server:
   ```bash
   cargo run --release
   ```

---

## Running the Firmware

### Prerequisites

- Install the ARM embedded target for Rust:
  ```bash
  rustup target add thumbv7em-none-eabihf
  ```

- Install `probe-rs` for flashing and debugging:
  ```bash
  cargo install probe-rs-tools
  ```

### Building and Flashing

1. Navigate to the firmware crate:
   ```bash
   cd crates/firmware
   ```

2. Build and flash the firmware to your STM32G4 board:
   ```bash
   cargo run --release
   ```

The firmware's `.cargo/config.toml` is configured to automatically use `probe-rs` as the runner.

---

## License

The software is released under the GNU General Public License version 3.0