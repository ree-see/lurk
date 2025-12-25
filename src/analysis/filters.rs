use crate::models::KeystrokeEvent;

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub max_gap_ms: i64,
    pub min_hold_ms: i64,
    pub max_hold_ms: i64,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            max_gap_ms: 5000,
            min_hold_ms: 10,
            max_hold_ms: 2000,
        }
    }
}

impl FilterConfig {
    pub fn is_valid_interval(&self, interval_ms: i64) -> bool {
        interval_ms > 0 && interval_ms < self.max_gap_ms
    }

    pub fn is_valid_hold_duration(&self, duration_ms: i64) -> bool {
        duration_ms >= self.min_hold_ms && duration_ms <= self.max_hold_ms
    }

    pub fn filter_events_by_gap<'a>(
        &self,
        events: &'a [KeystrokeEvent],
    ) -> Vec<&'a [KeystrokeEvent]> {
        if events.is_empty() {
            return vec![];
        }

        let mut segments = vec![];
        let mut start_idx = 0;

        for i in 1..events.len() {
            let gap = events[i].timestamp - events[i - 1].timestamp;
            if gap > self.max_gap_ms {
                if start_idx < i {
                    segments.push(&events[start_idx..i]);
                }
                start_idx = i;
            }
        }

        if start_idx < events.len() {
            segments.push(&events[start_idx..]);
        }

        segments
    }
}

pub fn calculate_percentiles(values: &mut [i64]) -> Option<(i64, i64, i64, i64)> {
    if values.is_empty() {
        return None;
    }

    values.sort_unstable();

    let p50 = calculate_percentile_sorted(values, 0.50)?;
    let p90 = calculate_percentile_sorted(values, 0.90)?;
    let p95 = calculate_percentile_sorted(values, 0.95)?;
    let p99 = calculate_percentile_sorted(values, 0.99)?;

    Some((p50, p90, p95, p99))
}

fn calculate_percentile_sorted(sorted_values: &[i64], percentile: f64) -> Option<i64> {
    if sorted_values.is_empty() {
        return None;
    }
    let idx = ((sorted_values.len() as f64 - 1.0) * percentile) as usize;
    Some(sorted_values[idx.min(sorted_values.len() - 1)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EventType, KeystrokeEvent};

    fn make_event(timestamp: i64) -> KeystrokeEvent {
        KeystrokeEvent {
            timestamp,
            key_code: 0x00,
            event_type: EventType::Press,
            modifiers: vec![],
            application: "test".to_string(),
        }
    }

    #[test]
    fn test_filter_config_default() {
        let config = FilterConfig::default();
        assert_eq!(config.max_gap_ms, 5000);
        assert_eq!(config.min_hold_ms, 10);
        assert_eq!(config.max_hold_ms, 2000);
    }

    #[test]
    fn test_is_valid_interval() {
        let config = FilterConfig::default();
        
        assert!(config.is_valid_interval(100));
        assert!(config.is_valid_interval(4999));
        assert!(!config.is_valid_interval(5000));
        assert!(!config.is_valid_interval(10000));
        assert!(!config.is_valid_interval(0));
        assert!(!config.is_valid_interval(-1));
    }

    #[test]
    fn test_is_valid_hold_duration() {
        let config = FilterConfig::default();
        
        assert!(config.is_valid_hold_duration(50));
        assert!(config.is_valid_hold_duration(100));
        assert!(config.is_valid_hold_duration(2000));
        assert!(!config.is_valid_hold_duration(5));
        assert!(!config.is_valid_hold_duration(2001));
    }

    #[test]
    fn test_filter_events_by_gap_empty() {
        let config = FilterConfig::default();
        let events: Vec<KeystrokeEvent> = vec![];
        let segments = config.filter_events_by_gap(&events);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_filter_events_by_gap_single() {
        let config = FilterConfig::default();
        let events = vec![make_event(100)];
        let segments = config.filter_events_by_gap(&events);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].len(), 1);
    }

    #[test]
    fn test_filter_events_by_gap_continuous() {
        let config = FilterConfig::default();
        let events = vec![
            make_event(100),
            make_event(200),
            make_event(300),
        ];
        
        let segments = config.filter_events_by_gap(&events);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].len(), 3);
    }

    #[test]
    fn test_filter_events_by_gap_with_break() {
        let config = FilterConfig::default();
        let events = vec![
            make_event(100),
            make_event(200),
            make_event(10000),
            make_event(10100),
        ];
        
        let segments = config.filter_events_by_gap(&events);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].len(), 2);
        assert_eq!(segments[1].len(), 2);
    }



    #[test]
    fn test_calculate_percentiles() {
        let mut values: Vec<i64> = (1..=100).collect();
        
        let (p50, p90, p95, p99) = calculate_percentiles(&mut values).unwrap();
        
        assert!(p50 >= 49 && p50 <= 51);
        assert!(p90 >= 89 && p90 <= 91);
        assert!(p95 >= 94 && p95 <= 96);
        assert!(p99 >= 98 && p99 <= 100);
    }

    #[test]
    fn test_calculate_percentiles_empty() {
        let mut values: Vec<i64> = vec![];
        assert!(calculate_percentiles(&mut values).is_none());
    }

    #[test]
    fn test_calculate_percentiles_single() {
        let mut values = vec![42];
        let (p50, p90, p95, p99) = calculate_percentiles(&mut values).unwrap();
        assert_eq!(p50, 42);
        assert_eq!(p90, 42);
        assert_eq!(p95, 42);
        assert_eq!(p99, 42);
    }
}
