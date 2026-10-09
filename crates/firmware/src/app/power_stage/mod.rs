#[cfg(feature = "cap-drv8301")]
mod drv8301;

#[cfg(feature = "cap-drv8301")]
pub(crate) use drv8301::{
    BoardOutput, UncheckedOutput, handle_request, next_gate_request, task_gate_driver,
};
