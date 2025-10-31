//! Rule dot semantics.

use super::{EventAndDistance, EventId, ExternalDottedRule, MinimalDistance};

/// Semantics for a dotted rule.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct RuleDot {
    /// Event at this rule dot. Can be none.
    pub event: EventId,
    /// Tracing information for this rule dot. Can be none.
    pub trace: ExternalDottedRule,
    /// Distance from this dot to events. Can be none.
    pub distance: MinimalDistance,
}

impl RuleDot {
    /// Rule dot with the given tracing information.
    pub fn new(id: u32, pos: u32) -> Self {
        RuleDot {
            event: EventId::null(),
            trace: ExternalDottedRule { id, pos },
            distance: MinimalDistance::null(),
        }
    }

    /// Rule dot with no semantics.
    pub fn none() -> Self {
        RuleDot {
            event: EventId::null(),
            trace: ExternalDottedRule::null(),
            distance: MinimalDistance::null(),
        }
    }

    /// Tracing information.
    pub fn trace(self) -> ExternalDottedRule {
        self.trace
    }

    /// Event at this dotted rule.
    pub fn event(self) -> EventId {
        self.event
    }

    /// Minimal distance to events at this dotted rule.
    pub fn distance(self) -> MinimalDistance {
        self.distance
    }

    /// The dotted rule's semantics for both event
    /// and minimal distance to events.
    pub fn event_and_distance(self) -> EventAndDistance {
        (self.event(), self.distance())
    }
}
