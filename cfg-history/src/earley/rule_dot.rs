use super::{EventId, Event, MinimalDistance, ExternalDottedRule};

#[derive(Copy, Clone, Debug, Default)]
pub struct RuleDot {
    pub event: Option<(EventId, ExternalDottedRule)>,
    pub distance: MinimalDistance,
}

impl RuleDot {
    pub fn new(id: u32, pos: usize) -> Self {
        RuleDot {
            event: Some((EventId(None.into()), (id, pos as u32))),
            distance: MinimalDistance(None.into()),
        }
    }

    pub fn none() -> Self {
        RuleDot {
            event: None,
            distance: MinimalDistance(None.into()),
        }
    }

    pub fn trace(self) -> Option<ExternalDottedRule> {
        self.event.map(|x| x.1)
    }

    pub fn event(self) -> Option<(EventId, ExternalDottedRule)> {
        self.event
    }

    pub fn event_without_tracing(self) -> Event {
        (EventId(self.event.and_then(|(id, _external_dotted_rule)| id.0.to_option()).into()), self.distance)
    }

    pub fn distance(&self) -> MinimalDistance {
        self.distance
    }
}