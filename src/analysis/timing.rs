use std::collections::HashMap;

use crate::analysis::filters::{calculate_percentiles, FilterConfig};
use crate::models::keycode::KeyCode;
use crate::models::{EventType, KeystrokeEvent};

#[derive(Debug, Clone)]
pub struct InterKeyInterval {
    pub from_key: u32,
    pub to_key: u32,
    pub intervals_ms: Vec<i64>,
    pub mean_ms: f64,
    pub median_ms: i64,
    pub p95_ms: i64,
}

#[derive(Debug, Clone)]
pub struct HoldDuration {
    pub key_code: u32,
    pub key_name: String,
    pub durations_ms: Vec<i64>,
    pub mean_ms: f64,
    pub median_ms: i64,
    pub p95_ms: i64,
    pub sample_count: usize,
}

#[derive(Debug)]
pub struct TimingAnalysis {
    pub overall_inter_key: InterKeyStats,
    pub per_key_inter_key: Vec<InterKeyInterval>,
    pub hold_durations: Vec<HoldDuration>,
    pub filter_config: FilterConfig,
}

#[derive(Debug, Clone)]
pub struct InterKeyStats {
    pub count: usize,
    pub mean_ms: f64,
    pub median_ms: i64,
    pub p90_ms: i64,
    pub p95_ms: i64,
    pub p99_ms: i64,
}

impl TimingAnalysis {
    pub fn from_events(events: &[KeystrokeEvent], config: FilterConfig) -> Self {
        let overall_inter_key = Self::calculate_overall_inter_key(events, &config);
        let per_key_inter_key = Self::calculate_per_key_inter_key(events, &config);
        let hold_durations = Self::calculate_hold_durations(events, &config);

        Self {
            overall_inter_key,
            per_key_inter_key,
            hold_durations,
            filter_config: config,
        }
    }

    fn calculate_per_key_inter_key(
        events: &[KeystrokeEvent],
        config: &FilterConfig,
    ) -> Vec<InterKeyInterval> {
        let press_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Press))
            .collect();

        let mut pair_intervals: HashMap<(u32, u32), Vec<i64>> = HashMap::new();

        for pair in press_events.windows(2) {
            let interval = pair[1].timestamp - pair[0].timestamp;
            if config.is_valid_interval(interval) {
                let key_pair = (pair[0].key_code, pair[1].key_code);
                pair_intervals.entry(key_pair).or_default().push(interval);
            }
        }

        let mut results: Vec<_> = pair_intervals
            .into_iter()
            .filter(|(_, intervals)| intervals.len() >= 3)
            .map(|((from_key, to_key), mut intervals)| {
                let count = intervals.len();
                let sum: i64 = intervals.iter().sum();
                let mean_ms = sum as f64 / count as f64;

                intervals.sort_unstable();
                let median_ms = intervals[count / 2];
                let p95_idx = ((count as f64 * 0.95) as usize).min(count.saturating_sub(1));
                let p95_ms = intervals[p95_idx];

                InterKeyInterval {
                    from_key,
                    to_key,
                    intervals_ms: intervals,
                    mean_ms,
                    median_ms,
                    p95_ms,
                }
            })
            .collect();

        results.sort_by(|a, b| b.intervals_ms.len().cmp(&a.intervals_ms.len()));
        results
    }

    fn calculate_overall_inter_key(events: &[KeystrokeEvent], config: &FilterConfig) -> InterKeyStats {
        let press_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Press))
            .collect();

        let mut intervals: Vec<i64> = press_events
            .windows(2)
            .filter_map(|pair| {
                let interval = pair[1].timestamp - pair[0].timestamp;
                if config.is_valid_interval(interval) {
                    Some(interval)
                } else {
                    None
                }
            })
            .collect();

        if intervals.is_empty() {
            return InterKeyStats {
                count: 0,
                mean_ms: 0.0,
                median_ms: 0,
                p90_ms: 0,
                p95_ms: 0,
                p99_ms: 0,
            };
        }

        let count = intervals.len();
        let sum: i64 = intervals.iter().sum();
        let mean_ms = sum as f64 / count as f64;

        let (median_ms, p90_ms, p95_ms, p99_ms) =
            calculate_percentiles(&mut intervals).unwrap_or((0, 0, 0, 0));

        InterKeyStats {
            count,
            mean_ms,
            median_ms,
            p90_ms,
            p95_ms,
            p99_ms,
        }
    }

    fn calculate_hold_durations(
        events: &[KeystrokeEvent],
        config: &FilterConfig,
    ) -> Vec<HoldDuration> {
        let mut press_times: HashMap<u32, Vec<i64>> = HashMap::new();
        let mut hold_data: HashMap<u32, Vec<i64>> = HashMap::new();

        for event in events {
            match event.event_type {
                EventType::Press => {
                    press_times
                        .entry(event.key_code)
                        .or_default()
                        .push(event.timestamp);
                }
                EventType::Release => {
                    if let Some(times) = press_times.get_mut(&event.key_code) {
                        if let Some(press_time) = times.pop() {
                            let duration = event.timestamp - press_time;
                            if config.is_valid_hold_duration(duration) {
                                hold_data.entry(event.key_code).or_default().push(duration);
                            }
                        }
                    }
                }
            }
        }

        let mut results: Vec<_> = hold_data
            .into_iter()
            .map(|(key_code, mut durations)| {
                let sample_count = durations.len();
                let sum: i64 = durations.iter().sum();
                let mean_ms = if sample_count > 0 {
                    sum as f64 / sample_count as f64
                } else {
                    0.0
                };

                durations.sort_unstable();
                let median_ms = durations.get(sample_count / 2).copied().unwrap_or(0);
                let p95_idx = ((sample_count as f64 * 0.95) as usize).min(sample_count.saturating_sub(1));
                let p95_ms = durations.get(p95_idx).copied().unwrap_or(0);

                HoldDuration {
                    key_code,
                    key_name: KeyCode(key_code).to_name(),
                    durations_ms: durations,
                    mean_ms,
                    median_ms,
                    p95_ms,
                    sample_count,
                }
            })
            .collect();

        results.sort_by(|a, b| b.sample_count.cmp(&a.sample_count));
        results
    }

    pub fn top_hold_durations(&self, n: usize) -> &[HoldDuration] {
        &self.hold_durations[..n.min(self.hold_durations.len())]
    }

    pub fn top_inter_key_pairs(&self, n: usize) -> &[InterKeyInterval] {
        &self.per_key_inter_key[..n.min(self.per_key_inter_key.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_press(timestamp: i64, key_code: u32) -> KeystrokeEvent {
        KeystrokeEvent {
            timestamp,
            key_code,
            event_type: EventType::Press,
            modifiers: vec![],
            application: "test".to_string(),
        }
    }

    fn make_release(timestamp: i64, key_code: u32) -> KeystrokeEvent {
        KeystrokeEvent {
            timestamp,
            key_code,
            event_type: EventType::Release,
            modifiers: vec![],
            application: "test".to_string(),
        }
    }

    #[test]
    fn test_empty_events() {
        let analysis = TimingAnalysis::from_events(&[], FilterConfig::default());
        assert_eq!(analysis.overall_inter_key.count, 0);
    }

    #[test]
    fn test_inter_key_interval() {
        let events = vec![
            make_press(100, 0x00),
            make_press(200, 0x01),
            make_press(300, 0x02),
        ];

        let analysis = TimingAnalysis::from_events(&events, FilterConfig::default());
        assert_eq!(analysis.overall_inter_key.count, 2);
        assert!((analysis.overall_inter_key.mean_ms - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_inter_key_filters_large_gaps() {
        let events = vec![
            make_press(100, 0x00),
            make_press(10000, 0x01),
        ];

        let analysis = TimingAnalysis::from_events(&events, FilterConfig::default());
        assert_eq!(analysis.overall_inter_key.count, 0);
    }

    #[test]
    fn test_hold_duration_calculation() {
        let events = vec![
            make_press(100, 0x00),
            make_release(200, 0x00),
            make_press(300, 0x00),
            make_release(400, 0x00),
        ];

        let analysis = TimingAnalysis::from_events(&events, FilterConfig::default());
        let hold = &analysis.hold_durations[0];
        
        assert_eq!(hold.key_code, 0x00);
        assert_eq!(hold.sample_count, 2);
        assert!((hold.mean_ms - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_hold_duration_filters_invalid() {
        let config = FilterConfig {
            max_gap_ms: 5000,
            min_hold_ms: 50,
            max_hold_ms: 500,
        };

        let events = vec![
            make_press(100, 0x00),
            make_release(110, 0x00),
            make_press(200, 0x01),
            make_release(1000, 0x01),
        ];

        let analysis = TimingAnalysis::from_events(&events, config);
        assert!(analysis.hold_durations.is_empty());
    }

    #[test]
    fn test_percentiles() {
        let events: Vec<KeystrokeEvent> = (0..100)
            .flat_map(|i| {
                vec![
                    make_press(i * 100, 0x00),
                ]
            })
            .collect();

        let analysis = TimingAnalysis::from_events(&events, FilterConfig::default());
        assert!(analysis.overall_inter_key.median_ms > 0);
        assert!(analysis.overall_inter_key.p95_ms >= analysis.overall_inter_key.median_ms);
    }

    #[test]
    fn test_multiple_keys_hold_duration() {
        let events = vec![
            make_press(100, 0x00),
            make_release(200, 0x00),
            make_press(100, 0x01),
            make_release(250, 0x01),
            make_press(100, 0x01),
            make_release(250, 0x01),
        ];

        let analysis = TimingAnalysis::from_events(&events, FilterConfig::default());
        assert_eq!(analysis.hold_durations.len(), 2);
        
        let key_01 = analysis.hold_durations.iter().find(|h| h.key_code == 0x01).unwrap();
        assert_eq!(key_01.sample_count, 2);
    }
}
