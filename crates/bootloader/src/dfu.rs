use defmt::info;
use embassy_boot_stm32::BlockingFirmwareUpdater;
use embassy_usb::class::dfu::consts::DfuAttributes;
use embassy_usb::class::dfu::dfu_mode::Handler;
use embassy_usb::control::{InResponse, OutResponse, Request};
use embassy_usb::driver::Driver;
use embassy_usb::{Builder, FunctionBuilder};
use embassy_usb_dfu::dfu::{FirmwareHandler, UsbDfuState};
use embassy_usb_dfu::{Reset, ResetImmediate};
use embedded_storage::nor_flash::NorFlash;
use hardware::BoardLeds;
// This code is derived from embassy-usb and embassy-usb-dfu.
// Modifications were made to ensure that restarts triggered by dfu-util function correctly.

pub(crate) const USB_CLASS_APPN_SPEC: u8 = 0xFE;
pub(crate) const APPN_SPEC_SUBCLASS_DFU: u8 = 0x01;
pub(crate) const DFU_PROTOCOL_DFU: u8 = 0x02;
pub(crate) const DESC_DFU_FUNCTIONAL: u8 = 0x21;

pub fn new_state<'a, DFU: NorFlash, STATE: NorFlash, const BLOCK_SIZE: usize>(
    updater: BlockingFirmwareUpdater<'a, DFU, STATE>,
    board_leds: BoardLeds<'a>,
) -> DfuState<'a, FirmwareHandler<'a, DFU, STATE, ResetImmediate, BLOCK_SIZE>> {
    let handler = FirmwareHandler::new(updater, ResetImmediate);
    DfuState::new(handler, board_leds)
}

pub struct DfuState<'a, H: Handler> {
    inner: UsbDfuState<H>,
    attrs: DfuAttributes,
    board_leds: BoardLeds<'a>,
    finished: bool,
}

impl<'a, H: Handler> DfuState<'a, H> {
    pub fn new(handler: H, board_leds: BoardLeds<'a>) -> Self {
        let attrs = DfuAttributes::CAN_DOWNLOAD | DfuAttributes::MANIFESTATION_TOLERANT;
        let inner = UsbDfuState::new(handler, attrs);
        Self {
            inner,
            attrs,
            board_leds,
            finished: false,
        }
    }
}

impl<H: Handler> embassy_usb::Handler for DfuState<'_, H> {
    fn reset(&mut self) {
        self.inner.reset();
        if self.finished {
            info!("Goodbye!");
            ResetImmediate.sys_reset();
        }
    }

    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if req.request == 1
        //Request::Download && finished
        {
            self.board_leds.red.toggle();
            if req.length == 0 {
                self.finished = true;
            }
        }
        self.inner.control_out(req, data)
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        self.inner.control_in(req, buf)
    }
}

pub fn usb_dfu<'d, D: Driver<'d>, DFU: NorFlash, STATE: NorFlash, const BLOCK_SIZE: usize>(
    builder: &mut Builder<'d, D>,
    state: &'d mut DfuState<FirmwareHandler<DFU, STATE, ResetImmediate, BLOCK_SIZE>>,
    func_modifier: impl Fn(&mut FunctionBuilder<'_, 'd, D>),
) {
    let mut func = builder.function(0x00, 0x00, 0x00);

    func_modifier(&mut func);

    let mut iface = func.interface();
    let mut alt = iface.alt_setting(
        USB_CLASS_APPN_SPEC,
        APPN_SPEC_SUBCLASS_DFU,
        DFU_PROTOCOL_DFU,
        None,
    );
    alt.descriptor(
        DESC_DFU_FUNCTIONAL,
        &[
            state.attrs.bits(),
            0xc4,
            0x09,
            (BLOCK_SIZE & 0xff) as u8,
            ((BLOCK_SIZE & 0xff00) >> 8) as u8,
            0x10,
            0x01,
        ],
    );

    drop(func);
    builder.handler(state);
}
