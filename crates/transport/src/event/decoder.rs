use crate::decoder;
use crate::event::Event;

pub type Decoder = decoder::Decoder<Event, 0xCC>;
