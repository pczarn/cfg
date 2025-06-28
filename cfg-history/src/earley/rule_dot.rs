use super::{EventAndDistance, EventId, ExternalDottedRule, MinimalDistance};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct RuleDot {
    pub event: EventId,
    pub trace: ExternalDottedRule,
    pub distance: MinimalDistance,
}

impl RuleDot {
    pub fn new(id: u32, pos: u32) -> Self {
        RuleDot {
            event: EventId::null(),
            trace: ExternalDottedRule { id, pos },
            distance: MinimalDistance::null(),
        }
    }

    pub fn none() -> Self {
        RuleDot {
            event: EventId::null(),
            trace: ExternalDottedRule::null(),
            distance: MinimalDistance::null(),
        }
    }

    pub fn trace(self) -> ExternalDottedRule {
        self.trace
    }

    pub fn event(self) -> EventId {
        self.event
    }

    pub fn distance(self) -> MinimalDistance {
        self.distance
    }

    pub fn event_and_distance(self) -> EventAndDistance {
        (self.event(), self.distance())
    }
}