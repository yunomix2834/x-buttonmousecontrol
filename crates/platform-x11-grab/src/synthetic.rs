use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use xbuttonmousecontrol_core::InputEvent;

const TTL: Duration = Duration::from_millis(150);

#[derive(Clone, Default)]
pub struct X11SyntheticFilter {
    inner: Arc<Mutex<VecDeque<(InputEvent, Instant)>>>,
}

impl X11SyntheticFilter {
    pub fn record(&self, event: InputEvent) {
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();
        retain_recent(&mut inner, now);
        inner.push_back((event, now));
    }

    pub fn consume(&self, event: &InputEvent) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();
        retain_recent(&mut inner, now);

        if let Some(index) = inner.iter().position(|(candidate, _)| candidate == event) {
            inner.remove(index);
            true
        } else {
            false
        }
    }
}

fn retain_recent(inner: &mut VecDeque<(InputEvent, Instant)>, now: Instant) {
    inner.retain(|(_, ts)| now.duration_since(*ts) < TTL);
}
