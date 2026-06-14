use hardware::BoardLeds;

#[embassy_executor::task]
pub async fn task_leds(mut board_leds: BoardLeds<'static>) {
    let mut led_manager = led_manager::LedAnimator::new();
    led_manager.trigger(led_manager::VisualEvent::Initialized);
    led_manager.run(&mut board_leds).await;
}
