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

### Test CLI (`crates/cli`)

`pyrionctl` is a gRPC-only development client. With the server running:

```bash
./pyrion.sh pyrionctl devices list
./pyrion.sh pyrionctl device info
```

The info command discovers the only attached device, opens a short-lived session, reads
its firmware version and UID, and disconnects. Pass `--connection` when multiple
devices are available and `--output json` for automation.

`pyrionctl faults list` shows `GATE_DRIVER_STARTUP` as Active if power-up
preflight fails. The fault event contains six positional states, with
GateDriverStartup at index 4 and GateDriverRuntime at index 5. Runtime
`nFAULT` monitoring is deferred until there is a gate-enabled operational
path; the gate is lowered after preflight, so an off-state pin level must
not set `GATE_DRIVER_RUNTIME`. `GATE_DRIVER_STARTUP` remains Active for the
boot; other Active faults may resolve to Latched after verified recovery.
Latched faults preserve history without blocking operation, and
`faults clear-resolved` clears only Latched faults. Detailed startup causes are
logged via defmt/RTT, not returned by the fault query. The protobuf schema
is also maintained separately; both the in-repository schema and host
mapping name fault type numbers 5 (startup) and 6 (runtime).

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

## Running Unit Tests

```bash
cargo test -p controller-shared -p foc -p pid -p units -p crc-engine \
  -p transport -p logging -p led-manager --lib
```

The ADC epoch tests can also run independently of the embedded driver:

```bash
rustc --edition=2024 --test crates/drivers/src/adc/epoch.rs -o target/epoch-tests
./target/epoch-tests
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

### Flashing

1. Build and flash the bootloader:
    ```bash
    cargo flash --manifest-path crates/bootloader/Cargo.toml --release --chip STM32G474RE --target thumbv7em-none-eabihf
    ```

2. Build and flash the firmware:
   ```bash
   cargo flash --manifest-path crates/firmware/Cargo.toml --release --chip STM32G474RE --target thumbv7em-none-eabihf
   ```

Alternatively you can navigate to bootloader/firmware crates' folders and use `cargo run --release`.
The firmware's and bootloader's `.cargo/config.toml` is configured to automatically use `probe-rs` with correct chip and
target as the runner.

### ADC diagnostics

Ovo currently defaults to 30 kHz PWM. To flash the optional acquisition
diagnostic build, run:

```bash
./pyrion.sh flash --board ovo --features adc-timing
```

Earlier bench runs rejected roughly 2-4% of software epochs, mostly
when TIM1_CC encountered pending ADC flags. The firmware now
accepts such a result only if its ADC already reported the preceding
epoch, TIM1 is past the current downcount compare, and no timer
overcapture was observed; otherwise it continues to reject the epoch.
In a subsequent approximately nine-second Ovo run at 30 kHz with
Disabled control, diagnostics reported zero missed epochs and invalid
frames and about 1,400 recovered pending results per second. That
establishes the observed software improvement, not guaranteed
sample-to-trigger ownership or PWM latch timing. Energized-control
timing and physical sampling aperture remain unverified.
This run did not exercise duty updates; motor arming and energized
control are separate work, not outcomes of the ADC refactor.

Capture several uninterrupted `ADC diag:` RTT windows. All counts are
per `window_us` (usually about one second); calculate the observed
trigger rate as `triggers * 1_000_000 / window_us`. With a 170 MHz
timer/CPU clock, the nominal 30 kHz period is about 5,667 cycles.

| Field | Meaning |
| --- | --- |
| `period_min`, `period_max` | DWT cycles between TIM1_CC ISR entries, **not** between physical PWM edges. |
| `pending_all`, `pending_partial`, `recovered` | TIM1_CC found JEOS pending on all three or some ADCs; `recovered` counts pending epochs that passed the preceding-conversion and timer-phase checks instead of being rejected. |
| `phase=[a,b,c]` | Pending entries with estimated timer phase `0..255`, `256..511`, or `>=512` timer ticks since the downcount CCR4 compare. `clear_late` counts no-pending entries at phase `>=256`; `uncertain_phase` counts inconsistent timer snapshots. |
| `invalid`, `stale`, `refresh` | Complete frames discarded as uncertain, duplicate/old, or retried while aligning ADC results. |
| `JEOS counts`, `max`, `unmatched` | Per-ADC interrupt counts and largest matched trigger-to-JEOS ISR-entry delays (cycles); unmatched events have no valid trigger-relative timestamp. |
| `undelivered=[ADC1,ADC3,ADC5]` | Results published by an ADC ISR but missing between two reads by the fast task; possible overwritten signals or canceled waits. |
| `missed_epochs`, `frames`, `frame_max` | Skipped software epochs, matched accepted frames, and largest trigger-to-frame delay (cycles); `unmatched_frames` counts frames without a valid delay. |
| `decisions`, `decision_max`, `work_max` | Matched output decisions and largest trigger-to-decision / frame-to-decision delays (cycles). `unmatched_decisions` means the trigger timestamp was replaced before the decision marker. |
| `duty_writes`, `duty_max` | Successful duty updates and largest matched trigger-to-write delay; normally zero while control is Disabled. |

Window counters are not an atomic snapshot, so adjacent totals may
differ slightly. A pending JEOS flag does **not** identify which timer
edge produced the ADC result. DWT timings are software intervals; they
do not measure analog sampling aperture, physical PWM edges, or the
PWM preload latch. Instrumentation and debugger halts perturb timing;
discard halted windows. Normal firmware builds exclude the diagnostic
hooks. The RTT logger does not mask interrupts or wait for a slow probe:
under contention or a full buffer it may drop complete log frames.

---

## License

The software is released under the GNU General Public License version 3.0