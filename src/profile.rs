use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Profile {
    state: Option<State>,
}

struct State {
    start: Instant,
    phase: &'static str,
    phase_start: Instant,
    phases: Vec<(&'static str, Duration)>,
    first_content: Option<Instant>,
    last_content: Option<Instant>,
    first_flush: Option<Duration>,
    output: Duration,
}

impl Profile {
    pub fn new(enabled: bool, start: Instant) -> Self {
        Self {
            state: enabled.then(|| State {
                start,
                phase: "startup / arguments",
                phase_start: start,
                phases: Vec::new(),
                first_content: None,
                last_content: None,
                first_flush: None,
                output: Duration::ZERO,
            }),
        }
    }

    pub fn clock(&self) -> Option<Instant> {
        self.state.as_ref().map(|_| Instant::now())
    }

    pub fn enter(&mut self, phase: &'static str) {
        if let Some(state) = &mut self.state {
            state.enter_at(phase, Instant::now());
        }
    }

    pub fn content(&mut self) {
        if let Some(state) = &mut self.state {
            let now = Instant::now();
            if state.first_content.is_none() {
                state.first_content = Some(now);
                state.enter_at("streaming content", now);
            }
            state.last_content = Some(now);
        }
    }

    pub fn flushed(&mut self, start: Option<Instant>, nonempty: bool) {
        if let Some(state) = &mut self.state {
            let now = Instant::now();
            if let Some(start) = start {
                state.output += now.duration_since(start);
            }
            if nonempty && state.first_flush.is_none() {
                state.first_flush = state.first_content.map(|first| now.duration_since(first));
            }
        }
    }

    pub fn stream_finished(&mut self) {
        if let Some(state) = &mut self.state {
            if let Some(last) = state.last_content {
                state.enter_at("stream tail", last);
            }
            state.enter_at("response finalization / stats", Instant::now());
        }
    }

    pub fn report(&mut self, success: bool, json: bool) {
        if self.state.is_none() {
            return;
        }
        let report = self.finish(Instant::now(), success);
        if json {
            eprintln!("{report}");
        } else {
            eprintln!("profile: {}", report["outcome"].as_str().unwrap());
            for phase in report["phases"].as_array().unwrap() {
                eprintln!(
                    "  {:<30} {:>10.3} ms{}",
                    phase["name"].as_str().unwrap(),
                    phase["duration_ms"].as_f64().unwrap(),
                    if phase["incomplete"] == true {
                        " (incomplete)"
                    } else {
                        ""
                    }
                );
            }
            eprintln!(
                "  {:<30} {:>10.3} ms",
                "total application time",
                report["total_ms"].as_f64().unwrap()
            );
            if let Some(ms) = report["first_content_to_flush_ms"].as_f64() {
                eprintln!("  first content → flush: {ms:.3} ms (overlaps phases)");
            }
            eprintln!(
                "  chunk writes / flushes: {:.3} ms (overlaps phases)",
                report["output_ms"].as_f64().unwrap()
            );
        }
    }

    fn finish(&mut self, end: Instant, success: bool) -> serde_json::Value {
        let state = self.state.take().unwrap();
        let mut phases: Vec<_> = state.phases.iter().map(|(name, duration)| {
            serde_json::json!({"name": name, "duration_ms": millis(*duration), "incomplete": false})
        }).collect();
        phases.push(serde_json::json!({
            "name": state.phase,
            "duration_ms": millis(end.duration_since(state.phase_start)),
            "incomplete": !success,
        }));
        serde_json::json!({
            "type": "pls_profile",
            "schema_version": 1,
            "outcome": if success { "success" } else { "error" },
            "failed_phase": if success { None } else { Some(state.phase) },
            "total_ms": millis(end.duration_since(state.start)),
            "phases": phases,
            "first_content_to_flush_ms": state.first_flush.map(millis),
            "output_ms": millis(state.output),
        })
    }
}

impl State {
    fn enter_at(&mut self, phase: &'static str, now: Instant) {
        self.phases
            .push((self.phase, now.duration_since(self.phase_start)));
        self.phase = phase;
        self.phase_start = now;
    }
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_phase_is_partial_and_durations_partition_total() {
        let start = Instant::now();
        let mut profile = Profile::new(true, start);
        profile
            .state
            .as_mut()
            .unwrap()
            .enter_at("config", start + Duration::from_millis(2));
        let report = profile.finish(start + Duration::from_millis(5), false);
        assert_eq!(report["total_ms"], 5.0);
        assert_eq!(report["phases"][0]["duration_ms"], 2.0);
        assert_eq!(report["phases"][1]["duration_ms"], 3.0);
        assert_eq!(report["phases"][1]["incomplete"], true);
        assert_eq!(report["failed_phase"], "config");
        assert!(report["first_content_to_flush_ms"].is_null());
    }

    #[test]
    fn streaming_phases_partition_time_and_output_overlaps() {
        let mut profile = Profile::new(true, Instant::now());
        profile.enter("request to first content");
        profile.content();
        profile.flushed(profile.clock(), true);
        let first_flush = profile.state.as_ref().unwrap().first_flush;
        profile.content();
        profile.flushed(profile.clock(), true);
        assert_eq!(profile.state.as_ref().unwrap().first_flush, first_flush);
        profile.stream_finished();
        let report = profile.finish(Instant::now(), true);
        let phases = report["phases"].as_array().unwrap();
        assert!(phases.iter().any(|p| p["name"] == "streaming content"));
        assert!(phases.iter().any(|p| p["name"] == "stream tail"));
        let sum: f64 = phases
            .iter()
            .map(|p| p["duration_ms"].as_f64().unwrap())
            .sum();
        assert!((sum - report["total_ms"].as_f64().unwrap()).abs() < 0.000001);
        assert!(report["failed_phase"].is_null());
        assert!(report["first_content_to_flush_ms"].is_number());
    }

    #[test]
    fn empty_stream_has_no_content_metrics() {
        let mut profile = Profile::new(true, Instant::now());
        profile.enter("request to first content");
        profile.flushed(profile.clock(), false);
        profile.stream_finished();
        let report = profile.finish(Instant::now(), true);
        assert!(report["first_content_to_flush_ms"].is_null());
        assert!(
            !report["phases"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["name"] == "stream tail" || p["name"] == "streaming content")
        );
    }

    #[test]
    fn disabled_profile_does_not_collect() {
        let mut profile = Profile::default();
        profile.enter("request");
        profile.content();
        profile.flushed(profile.clock(), true);
        profile.stream_finished();
        assert!(profile.clock().is_none());
        assert!(profile.state.is_none());
    }
}
