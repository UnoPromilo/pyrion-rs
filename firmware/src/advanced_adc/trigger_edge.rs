use stm32_metapac::adc::vals::Exten;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ExtTriggerEdge {
    Rising,
    Falling,
    Both,
}

impl Into<Exten> for ExtTriggerEdge {
    fn into(self) -> Exten {
        match self {
            ExtTriggerEdge::Rising => Exten::RISING_EDGE,
            ExtTriggerEdge::Falling => Exten::FALLING_EDGE,
            ExtTriggerEdge::Both => Exten::BOTH_EDGES,
        }
    }
}
