#!/usr/bin/env bash
#
# pyrion-rs Development Script
# Helps with flashing, building, DFU, and logs.

set -euo pipefail

CHIP="STM32G474RE"
TARGET_TRIPLE="thumbv7em-none-eabihf"
DFU_VID_PID="1209:2aaa"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [[ ! -d "crates" ]]; then
    error "Could not find the 'crates' directory. Please place this script in the repository root."
fi

ACTION_FLASH=""
ACTION_DFU=""
RUN_SERVER=0
ATTACH_LOGS=0

check_deps() {
    local missing_deps=0
    for cmd in cargo dfu-util probe-rs; do
        if ! command -v "$cmd" &> /dev/null; then
            warn "'$cmd' is not installed."
            missing_deps=1
        fi
    done

    if ! cargo objcopy --help &> /dev/null; then
        warn "'cargo-objcopy' is missing. Install with: cargo install cargo-binutils && rustup component add llvm-tools"
        missing_deps=1
    fi

    if [[ $missing_deps -eq 1 ]]; then
        error "Missing dependencies. Please install them before continuing."
    fi
}

print_help() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

A development tool for pyrion-rs.

Commands:
  flash [TARGET]      Flash using probe-rs (cargo run).
                      Targets: bootloader, firmware (default).
  dfu                 Flash firmware using usb-dfu (dfu-util).
  server              Run the server crate in release mode
  logs                Attach to firmware logs using probe-rs
  help, -h, --help    Print this help message
EOF
}

do_flash() {
    local target=$1
    if [[ "$target" == "bootloader" ]]; then
        info "Flashing bootloader via probe-rs..."
        pushd "crates/bootloader" > /dev/null
          cargo run --release
        popd > /dev/null
        success "Bootloader flashed."
    fi
    if [[ "$target" == "firmware" ]]; then
        info "Flashing firmware via probe-rs..."
        pushd "crates/firmware" > /dev/null
          cargo run --release
        popd > /dev/null
        success "Firmware flashed."
        info "Goodbye!"
    fi
}

do_dfu() {
    pushd "crates/firmware" > /dev/null
      info "Building 'firmware' for DFU..."
      cargo build --release

      local bin_out="firmware.bin"
      info "Generating raw binary ($bin_out)..."
      cargo objcopy --release -- -O binary "$bin_out"

      info "Flashing 'firmware' via DFU..."
      local dfu_output
      local dfu_status
      set +e
      dfu_output=$(dfu-util -d "$DFU_VID_PID" -D "$bin_out" -R 2>&1 | tee /dev/stderr)
      dfu_status=${PIPESTATUS[0]}
      set -e

       # Simple one-liner check
      if [[ $dfu_status -eq 0 ]] || [[ "$dfu_output" == *"Done!"* ]]; then
          success "Firmware flashed."
          success "Goodbye!"
      fi
    popd > /dev/null
}

if [[ $# -eq 0 ]]; then
    print_help
    exit 0
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        flash)
            if [[ -n "${2:-}" && "$2" =~ ^(bootloader|firmware)$ ]]; then
                ACTION_FLASH="$2"
                shift 2
            else
                ACTION_FLASH="firmware"
                shift 1
            fi
            ;;
        dfu)
            ACTION_DFU=1
            shift
            ;;
        server)
            RUN_SERVER=1
            shift
            ;;
        logs)
            ATTACH_LOGS=1
            shift
            ;;
        -h|--help|help)
            print_help
            exit 0
            ;;
        *)
            error "Unknown option: $1\nUse -h or --help for usage."
            ;;
    esac
done

check_deps

if [[ -n "$ACTION_FLASH" ]]; then
    do_flash "$ACTION_FLASH"

elif [[ -n "$ACTION_DFU" ]]; then
    do_dfu

elif [[ $RUN_SERVER -eq 1 ]]; then
    info "Starting server..."
    pushd crates/server > /dev/null
      cargo run --release
    popd > /dev/null

elif [[ $ATTACH_LOGS -eq 1 ]]; then
    info "Attaching probe-rs logs..."
    probe-rs attach --chip "$CHIP" "target/$TARGET_TRIPLE/release/firmware"
fi
