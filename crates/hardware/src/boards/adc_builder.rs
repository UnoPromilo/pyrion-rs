use crate::irqs::Irqs;
use crate::{BoardAdc, BoardAdcFast};
use drivers::adc::Adc;
use drivers::adc::slow_vref::{SlowVref, VREF_DMA_SAMPLES};
use embassy_stm32::Peri;
use embassy_stm32::adc::{AdcChannel, AdcConfig, CONTINUOUS, Exten, SampleTime};
use embassy_stm32::peripherals::{
    ADC1, ADC2, ADC3, ADC4, ADC5, DMA1_CH2, DMA1_CH3, PA1, PA2, PA3, PA8, PA9, PB0, PB1, PB2, PB11,
    PC3,
};
use static_cell::StaticCell;

static VREF_DMA_BUFFER: StaticCell<[u16; VREF_DMA_SAMPLES]> = StaticCell::new();
const AUX_DMA_SAMPLES: usize = 3 * 512;
static AUX_DMA_BUFFER: StaticCell<[u16; AUX_DMA_SAMPLES]> = StaticCell::new();

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_adc(
    adc1: Peri<'static, ADC1>,
    adc2: Peri<'static, ADC2>,
    adc3: Peri<'static, ADC3>,
    adc4: Peri<'static, ADC4>,
    adc5: Peri<'static, ADC5>,
    aux_dma: Peri<'static, DMA1_CH2>,
    vref_dma: Peri<'static, DMA1_CH3>,
    i_u: Peri<'static, PA1>,
    v_u: Peri<'static, PA2>,
    analog_in: Peri<'static, PA3>,
    mosfet_temp: Peri<'static, PB11>,
    motor_temp: Peri<'static, PC3>,
    v_bus: Peri<'static, PB2>,
    i_v: Peri<'static, PB0>,
    v_v: Peri<'static, PB1>,
    i_w: Peri<'static, PA9>,
    v_w: Peri<'static, PA8>,
) -> BoardAdc<'static> {
    let adc2 = embassy_stm32::adc::Adc::new(adc2, AdcConfig::default());
    let adc1 = Adc::new_in_enabled_group(adc1);
    let adc4 = embassy_stm32::adc::Adc::new(adc4, AdcConfig::default());
    let adc3 = Adc::new_in_enabled_group(adc3);
    let adc5 = Adc::new_in_enabled_group(adc5);
    let _analog_channel = analog_in.degrade_adc();

    let (adc1, adc1_configured) = adc1.configure_tim1_triggered();
    let (adc3, adc3_configured) = adc3.configure_tim1_triggered();
    let (adc5, adc5_configured) = adc5.configure_tim1_triggered();

    let adc1_running = adc1_configured.start(
        [
            (i_u.degrade_adc(), SampleTime::CYCLES12_5),
            (v_u.degrade_adc(), SampleTime::CYCLES12_5),
        ],
        Irqs,
    );

    let adc3_running = adc3_configured.start(
        [
            (i_v.degrade_adc(), SampleTime::CYCLES12_5),
            (v_v.degrade_adc(), SampleTime::CYCLES12_5),
        ],
        Irqs,
    );

    let adc5_running = adc5_configured.start(
        [
            (i_w.degrade_adc(), SampleTime::CYCLES12_5),
            (v_w.degrade_adc(), SampleTime::CYCLES12_5),
        ],
        Irqs,
    );

    let mut slow_aux = adc2.into_ring_buffered(
        aux_dma,
        AUX_DMA_BUFFER.init([0; AUX_DMA_SAMPLES]),
        Irqs,
        [
            (mosfet_temp.degrade_adc(), SampleTime::CYCLES640_5),
            (motor_temp.degrade_adc(), SampleTime::CYCLES640_5),
            (v_bus.degrade_adc(), SampleTime::CYCLES640_5),
        ]
        .into_iter(),
        CONTINUOUS,
        Exten::DISABLED,
    );
    slow_aux.start();

    let slow_vref = SlowVref::new(
        adc4,
        vref_dma,
        VREF_DMA_BUFFER.init([0; VREF_DMA_SAMPLES]),
        Irqs,
    );

    BoardAdc {
        fast: BoardAdcFast {
            _adc1: adc1,
            _adc3: adc3,
            _adc5: adc5,
            adc1_running,
            adc3_running,
            adc5_running,
        },
        slow_vref,
        slow_aux,
    }
}
