use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default)]
struct Stat {
    calls: u64,
    total: Duration,
}

static ENABLED: OnceLock<bool> = OnceLock::new();
static STATS: OnceLock<Mutex<BTreeMap<&'static str, Stat>>> = OnceLock::new();

pub struct Scope {
    name: &'static str,
    start: Instant,
    enabled: bool,
}

impl Scope {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            enabled: enabled(),
        }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }

        let elapsed = self.start.elapsed();
        let stats = STATS.get_or_init(|| Mutex::new(BTreeMap::new()));
        let mut stats = stats.lock().unwrap();
        let stat = stats.entry(self.name).or_default();
        stat.calls += 1;
        stat.total += elapsed;
    }
}

pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var_os("SVDD_PROFILE").is_some())
}

pub fn report() {
    if !enabled() {
        return;
    }

    let Some(stats) = STATS.get() else {
        return;
    };

    let stats = stats.lock().unwrap();
    eprintln!("== svdd profile ==");
    for (name, stat) in stats.iter().rev().collect::<Vec<_>>() {
        let avg = if stat.calls == 0 {
            Duration::ZERO
        } else {
            stat.total / stat.calls as u32
        };
        eprintln!(
            "{name}: total={:.3?} calls={} avg={:.3?}",
            stat.total, stat.calls, avg
        );
    }
}
