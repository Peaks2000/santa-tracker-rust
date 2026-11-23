//! Selected Web Audio set/linear/cancel automation for native shared music.
//! Getter snapshots update at explicit render-quantum boundaries, independently
//! of scheduling calls. No target/exponential/value-curve or input modulation.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Set,
    Linear,
}
#[derive(Clone, Copy, Debug)]
struct Event {
    at: f64,
    value: f32,
    kind: Kind,
}
#[derive(Clone, Debug)]
pub struct Param {
    default: f32,
    current: f32,
    events: Vec<Event>,
}
impl Param {
    pub fn new(value: f32) -> Self {
        Self {
            default: value,
            current: value,
            events: vec![],
        }
    }
    pub fn current(&self) -> f64 {
        self.current as f64
    }
    pub fn begin_quantum(&mut self, now: f64) {
        self.current = self.value(now);
        // Only the last past event can anchor a future linear ramp. Preserve
        // it when trimming history; never replace a future ramp by an envelope.
        if let Some(last) = self.events.iter().rposition(|e| e.at <= now) {
            self.events.drain(..last);
        }
    }
    pub fn cancel(&mut self, at: f64, now: f64) {
        self.events.retain(|e| e.at < at.max(now));
        if self.events.is_empty() {
            self.default = self.current;
        }
    }
    pub fn set(&mut self, value: f64, at: f64, now: f64) {
        self.insert(Event {
            at: at.max(now),
            value: value as f32,
            kind: Kind::Set,
        });
    }
    pub fn linear(&mut self, value: f64, at: f64, now: f64) {
        if !self.events.iter().any(|e| e.at <= at.max(now)) {
            self.set(self.current(), now, now);
        }
        self.insert(Event {
            at: at.max(now),
            value: value as f32,
            kind: Kind::Linear,
        });
    }
    fn insert(&mut self, event: Event) {
        let index = self.events.partition_point(|e| e.at <= event.at);
        self.events.insert(index, event);
    }
    pub fn value(&self, now: f64) -> f32 {
        let mut value = self.default;
        let mut previous_time = None;
        for event in &self.events {
            if now < event.at {
                if event.kind == Kind::Linear {
                    if let Some(start) = previous_time {
                        let fraction = (now - start) / (event.at - start);
                        return (value as f64 + (event.value as f64 - value as f64) * fraction)
                            as f32;
                    }
                }
                return value;
            }
            value = event.value;
            previous_time = Some(event.at);
        }
        value
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scheduled_changes_do_not_rewrite_control_thread_getter() {
        let mut p = Param::new(1.);
        p.set(0., 0., 0.);
        p.linear(1., 1., 0.);
        assert_eq!(p.current(), 1.);
        p.begin_quantum(0.25);
        assert_eq!(p.current(), 0.25);
        p.cancel(0.25, 0.25);
        assert_eq!(p.current(), 0.25); // read before another render quantum
        p.set(p.current(), 0.25, 0.25);
        p.linear(1., 0.5, 0.25);
        assert_eq!(p.value(0.375), 0.625);
        p.begin_quantum(0.5);
        assert_eq!(p.current(), 1.);
    }
    #[test]
    fn scheduled_ramp_end_removal_restores_prior_event_and_ties_keep_order() {
        let mut p = Param::new(1.);
        p.set(1., 0., 0.);
        p.linear(0., 2., 0.);
        assert_eq!(p.value(0.5), 0.75);
        p.cancel(1., 0.5);
        assert_eq!(p.value(0.5), 1.);
        p.set(0.25, 1., 0.5);
        p.set(0.5, 1., 0.5);
        p.linear(1., 1., 0.5);
        assert_eq!(p.value(1.), 1.);
        p.begin_quantum(1.);
        p.linear(0., 2., 1.);
        assert_eq!(p.value(1.5), 0.5);
    }
    #[test]
    fn past_schedules_clamp_and_parameter_inputs_are_float32() {
        let mut p = Param::new(1.);
        p.set(0.1, 0., 2.);
        assert_eq!(p.value(1.), 1.);
        assert_eq!(p.value(2.), 0.1f32);
        p.begin_quantum(2.);
        p.cancel(0., 2.);
        p.linear(0.3, 3., 2.);
        assert_eq!(p.value(2.5), ((0.1f32 as f64 + 0.3f32 as f64) / 2.) as f32);
    }
}
