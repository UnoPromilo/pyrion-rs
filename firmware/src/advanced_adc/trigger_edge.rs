use stm32_metapac::adc::vals::Exten;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ExternalTriggerEdge {
    Rising,
    Falling,
    Both,
}

impl Into<Exten> for ExternalTriggerEdge {
    fn into(self) -> Exten {
        match self {
            ExternalTriggerEdge::Rising => Exten::RISING_EDGE,
            ExternalTriggerEdge::Falling => Exten::FALLING_EDGE,
            ExternalTriggerEdge::Both => Exten::BOTH_EDGES,
        }
    }
}
