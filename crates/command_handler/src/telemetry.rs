use core::sync::atomic::Ordering;
use embassy_time::Instant;
use transport::event::Telemetry;
use units::si::electric_potential::volt;
use units::si::thermodynamic_temperature::kelvin;

pub fn get_telemetry() -> Telemetry {
    let controller_state = controller_shared::state::state();
    Telemetry {
        cpu_temperature: controller_state
            .cpu_temp
            .load(Ordering::Relaxed)
            .get::<kelvin>(),
        motor_temperature: 0.0, // TODO add motor temperature
        v_bus: controller_state.v_bus.load(Ordering::Relaxed).get::<volt>(),
        current_consumption: 0.0, // TODO add current consumption
        power_consumption: 0.0,   // TODO add power consumption
        duty_cycle: 0.0,          // TODO add duty cycle
        uptime: Instant::now().as_millis(),
    }
}
