use core::sync::atomic::{AtomicU32, Ordering};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

static NEXT_DFU_REQUEST_ID: AtomicU32 = AtomicU32::new(1);
static DFU_SHUTDOWN_REQUEST: Signal<CriticalSectionRawMutex, u32> = Signal::new();
static DFU_OUTPUT_INHIBITED: Signal<CriticalSectionRawMutex, u32> = Signal::new();

pub(super) fn request_dfu_shutdown() {
    let request_id = NEXT_DFU_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    DFU_SHUTDOWN_REQUEST.signal(request_id);
}

pub(super) async fn wait_for_dfu_shutdown_request() -> u32 {
    DFU_SHUTDOWN_REQUEST.wait().await
}

pub(super) fn acknowledge_dfu_output_inhibited(request_id: u32) {
    DFU_OUTPUT_INHIBITED.signal(request_id);
}

pub(super) async fn wait_for_dfu_output_inhibited(request_id: u32) {
    loop {
        if DFU_OUTPUT_INHIBITED.wait().await == request_id {
            return;
        }
    }
}
