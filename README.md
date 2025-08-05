# BLDC Motor Controller Firmware for RP2040
This is a bare-metal, async motor controller firmware for 3-phase BLDC motors running on the Raspberry Pi Pico (RP2040). It uses Rust, Embassy async runtime, and no_std environment.

# To run debugger
1. Run `cargo run` to upload code
2. Run `openocd -f interface/cmsis-dap.cfg -f target/rp2040.cfg -c "adapter speed 5000"` to run openocd server
3. Run `picocom /dev/tty.usbmodem12302 -b 115200` to start serial monitor
4. Run *__Debugger__* configuration to start debugging