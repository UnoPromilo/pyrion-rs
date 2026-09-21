use crate::BoardAdc;
use crate::irqs::Irqs;
use adc::Adc;
use adc::injected::{ExtTriggerSourceADC12, ExtTriggerSourceADC345};
use adc::trigger_edge::ExtTriggerEdge;
use embassy_stm32::Peri;
use embassy_stm32::adc::{AdcChannel, SampleTime};
use embassy_stm32::peripherals::{
    ADC1, ADC2, ADC3, ADC4, ADC5, PA1, PA2, PA3, PA8, PA9, PB0, PB1, PB2, PB11, PC3,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_adc(
    adc1: Peri<'static, ADC1>,
    adc2: Peri<'static, ADC2>,
    adc3: Peri<'static, ADC3>,
    adc4: Peri<'static, ADC4>,
    adc5: Peri<'static, ADC5>,
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
    let adc_config = adc::Config::default();
    let adc1 = Adc::new(adc1, adc_config);
    let adc2 = Adc::new(adc2, adc_config);
    let adc3 = Adc::new(adc3, adc_config);
    let adc4 = Adc::new(adc4, adc_config);
    let adc5 = Adc::new(adc5, adc_config);

    let (adc1, adc1_configured) =
        adc1.configure_injected_ext_trigger(ExtTriggerSourceADC12::T1_TRGO, ExtTriggerEdge::Rising);
    let (adc2, adc2_configured) =
        adc2.configure_injected_ext_trigger(ExtTriggerSourceADC12::T1_TRGO, ExtTriggerEdge::Rising);
    let (adc3, adc3_configured) =
        adc3.configure_injected_ext_trigger(ExtTriggerSourceADC345::T1_TRGO, ExtTriggerEdge::Rising);
    let (adc4, adc4_configured) =
        adc4.configure_injected_ext_trigger(ExtTriggerSourceADC345::T1_TRGO, ExtTriggerEdge::Rising);
    let (adc5, adc5_configured) =
        adc5.configure_injected_ext_trigger(ExtTriggerSourceADC345::T1_TRGO, ExtTriggerEdge::Rising);

    let adc1_running = adc1_configured.start(
        [
            (i_u.degrade_adc(), SampleTime::CYCLES6_5),
            (v_u.degrade_adc(), SampleTime::CYCLES6_5),
            (analog_in.degrade_adc(), SampleTime::CYCLES6_5),
        ],
        Irqs,
    );

    let adc2_running = adc2_configured.start(
        [
            (mosfet_temp.degrade_adc(), SampleTime::CYCLES6_5),
            (motor_temp.degrade_adc(), SampleTime::CYCLES6_5),
            (v_bus.degrade_adc(), SampleTime::CYCLES6_5),
        ],
        Irqs,
    );

    let adc3_running = adc3_configured.start(
        [
            (i_v.degrade_adc(), SampleTime::CYCLES6_5),
            (v_v.degrade_adc(), SampleTime::CYCLES6_5),
        ],
        Irqs,
    );

    let v_ref_int = adc4.enable_vrefint();
    let adc4_running =
        adc4_configured.start([(v_ref_int.degrade_adc(), SampleTime::CYCLES47_5)], Irqs);

    let adc5_running = adc5_configured.start(
        [
            (i_w.degrade_adc(), SampleTime::CYCLES6_5),
            (v_w.degrade_adc(), SampleTime::CYCLES6_5),
        ],
        Irqs,
    );

    BoardAdc {
        _adc1: adc1,
        _adc2: adc2,
        _adc3: adc3,
        _adc4: adc4,
        _adc5: adc5,
        adc1_running,
        adc2_running,
        adc3_running,
        adc4_running,
        adc5_running,
    }
}
