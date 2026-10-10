use embassy_stm32::adc::{Adc, AdcChannel, Exten, RingBufferedAdc, RxDma, SampleTime};
use embassy_stm32::dma::InterruptHandler;
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::peripherals::ADC4;
use embassy_stm32::triggers::TIM1_TRGO;
use embassy_time::{Duration, block_for};

pub const VREF_DMA_SAMPLES: usize = 1024;
const READ_SAMPLES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlowVrefError {
    NoSample,
    ZeroSample,
}

/// Dropping ADC4's Embassy ring disables the shared ADC345 clock, stopping ADC3/5.
pub struct SlowVref<'d> {
    ring: RingBufferedAdc<'d, ADC4>,
    last_nonzero: Option<u16>,
}

impl<'d> SlowVref<'d> {
    pub fn new<D: RxDma<ADC4>>(
        adc: Adc<'d, ADC4>,
        dma: embassy_stm32::Peri<'d, D>,
        dma_buf: &'d mut [u16; VREF_DMA_SAMPLES],
        irq: impl Binding<D::Interrupt, InterruptHandler<D>> + 'd,
    ) -> Self {
        let vref = adc.enable_vrefint();
        block_for(Duration::from_micros(12));
        let ring = adc.into_ring_buffered(
            dma,
            dma_buf,
            irq,
            [(vref.degrade_adc(), SampleTime::CYCLES640_5)].into_iter(),
            TIM1_TRGO,
            Exten::RISING_EDGE,
        );

        let mut result = Self {
            ring,
            last_nonzero: None,
        };
        result.ring.start();
        result
    }

    pub fn poll_latest(&mut self) -> Result<u16, SlowVrefError> {
        let mut samples = [0; READ_SAMPLES];
        let count = self.ring.read_latest(&mut samples);
        for &sample in &samples[..count] {
            if sample != 0 {
                self.last_nonzero = Some(sample);
            }
        }
        if count != 0 && samples[count - 1] == 0 {
            return Err(SlowVrefError::ZeroSample);
        }
        self.snapshot()
    }

    pub fn snapshot(&self) -> Result<u16, SlowVrefError> {
        self.last_nonzero.ok_or(SlowVrefError::NoSample)
    }
}
