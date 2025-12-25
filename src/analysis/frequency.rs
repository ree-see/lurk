use std::collections::HashMap;

use crate::models::keycode::KeyCode;
use crate::models::{EventType, KeystrokeEvent};

#[derive(Debug, Clone)]
pub struct KeyCount {
    pub key_code: u32,
    pub key_name: String,
    pub count: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct BigramCount {
    pub first_key: u32,
    pub second_key: u32,
    pub display: String,
    pub count: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct TrigramCount {
    pub keys: (u32, u32, u32),
    pub display: String,
    pub count: u64,
    pub percentage: f64,
}

#[derive(Debug)]
pub struct FrequencyAnalysis {
    pub total_presses: u64,
    pub key_frequencies: Vec<KeyCount>,
    pub bigram_frequencies: Vec<BigramCount>,
    pub trigram_frequencies: Vec<TrigramCount>,
}

impl FrequencyAnalysis {
    pub fn from_events(events: &[KeystrokeEvent]) -> Self {
        let press_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Press))
            .collect();

        let total_presses = press_events.len() as u64;

        let key_frequencies = Self::calculate_key_frequencies(&press_events, total_presses);
        let bigram_frequencies = Self::calculate_bigram_frequencies(&press_events, total_presses);
        let trigram_frequencies = Self::calculate_trigram_frequencies(&press_events, total_presses);

        Self {
            total_presses,
            key_frequencies,
            bigram_frequencies,
            trigram_frequencies,
        }
    }

    fn calculate_key_frequencies(events: &[&KeystrokeEvent], total: u64) -> Vec<KeyCount> {
        let mut counts: HashMap<u32, u64> = HashMap::new();

        for event in events {
            *counts.entry(event.key_code).or_insert(0) += 1;
        }

        let mut result: Vec<_> = counts
            .into_iter()
            .map(|(key_code, count)| KeyCount {
                key_code,
                key_name: KeyCode(key_code).to_name(),
                count,
                percentage: if total > 0 {
                    (count as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        result.sort_by(|a, b| b.count.cmp(&a.count));
        result
    }

    fn calculate_bigram_frequencies(events: &[&KeystrokeEvent], _total: u64) -> Vec<BigramCount> {
        let mut counts: HashMap<(u32, u32), u64> = HashMap::new();

        for window in events.windows(2) {
            let gap = window[1].timestamp - window[0].timestamp;
            if gap < 5000 {
                let bigram = (window[0].key_code, window[1].key_code);
                *counts.entry(bigram).or_insert(0) += 1;
            }
        }

        let bigram_total: u64 = counts.values().sum();

        let mut result: Vec<_> = counts
            .into_iter()
            .map(|((first, second), count)| BigramCount {
                first_key: first,
                second_key: second,
                display: format!("{} -> {}", KeyCode(first).to_name(), KeyCode(second).to_name()),
                count,
                percentage: if bigram_total > 0 {
                    (count as f64 / bigram_total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        result.sort_by(|a, b| b.count.cmp(&a.count));
        result
    }

    fn calculate_trigram_frequencies(events: &[&KeystrokeEvent], _total: u64) -> Vec<TrigramCount> {
        let mut counts: HashMap<(u32, u32, u32), u64> = HashMap::new();

        for window in events.windows(3) {
            let gap1 = window[1].timestamp - window[0].timestamp;
            let gap2 = window[2].timestamp - window[1].timestamp;
            if gap1 < 5000 && gap2 < 5000 {
                let trigram = (window[0].key_code, window[1].key_code, window[2].key_code);
                *counts.entry(trigram).or_insert(0) += 1;
            }
        }

        let trigram_total: u64 = counts.values().sum();

        let mut result: Vec<_> = counts
            .into_iter()
            .map(|(keys, count)| TrigramCount {
                keys,
                display: format!(
                    "{} -> {} -> {}",
                    KeyCode(keys.0).to_name(),
                    KeyCode(keys.1).to_name(),
                    KeyCode(keys.2).to_name()
                ),
                count,
                percentage: if trigram_total > 0 {
                    (count as f64 / trigram_total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        result.sort_by(|a, b| b.count.cmp(&a.count));
        result
    }

    pub fn top_keys(&self, n: usize) -> &[KeyCount] {
        &self.key_frequencies[..n.min(self.key_frequencies.len())]
    }

    pub fn top_bigrams(&self, n: usize) -> &[BigramCount] {
        &self.bigram_frequencies[..n.min(self.bigram_frequencies.len())]
    }

    pub fn top_trigrams(&self, n: usize) -> &[TrigramCount] {
        &self.trigram_frequencies[..n.min(self.trigram_frequencies.len())]
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
        let analysis = FrequencyAnalysis::from_events(&[]);
        assert_eq!(analysis.total_presses, 0);
        assert!(analysis.key_frequencies.is_empty());
    }

    #[test]
    fn test_key_frequency_count() {
        let events = vec![
            make_press(100, 0x00),
            make_press(200, 0x00),
            make_press(300, 0x01),
        ];

        let analysis = FrequencyAnalysis::from_events(&events);
        assert_eq!(analysis.total_presses, 3);
        
        let top = analysis.top_keys(10);
        assert_eq!(top[0].key_code, 0x00);
        assert_eq!(top[0].count, 2);
        assert_eq!(top[1].key_code, 0x01);
        assert_eq!(top[1].count, 1);
    }

    #[test]
    fn test_only_counts_presses() {
        let events = vec![
            make_press(100, 0x00),
            make_release(150, 0x00),
            make_press(200, 0x01),
            make_release(250, 0x01),
        ];

        let analysis = FrequencyAnalysis::from_events(&events);
        assert_eq!(analysis.total_presses, 2);
    }

    #[test]
    fn test_bigram_detection() {
        let events = vec![
            make_press(100, 0x00),
            make_press(200, 0x01),
            make_press(300, 0x00),
            make_press(400, 0x01),
        ];

        let analysis = FrequencyAnalysis::from_events(&events);
        let bigrams = analysis.top_bigrams(10);
        
        assert!(!bigrams.is_empty());
        let ab_bigram = bigrams.iter().find(|b| b.first_key == 0x00 && b.second_key == 0x01);
        assert!(ab_bigram.is_some());
        assert_eq!(ab_bigram.unwrap().count, 2);
    }

    #[test]
    fn test_bigram_filters_large_gaps() {
        let events = vec![
            make_press(100, 0x00),
            make_press(10000, 0x01),
        ];

        let analysis = FrequencyAnalysis::from_events(&events);
        assert!(analysis.bigram_frequencies.is_empty());
    }

    #[test]
    fn test_trigram_detection() {
        let events = vec![
            make_press(100, 0x00),
            make_press(200, 0x01),
            make_press(300, 0x02),
        ];

        let analysis = FrequencyAnalysis::from_events(&events);
        let trigrams = analysis.top_trigrams(10);
        
        assert_eq!(trigrams.len(), 1);
        assert_eq!(trigrams[0].keys, (0x00, 0x01, 0x02));
    }

    #[test]
    fn test_percentage_calculation() {
        let events = vec![
            make_press(100, 0x00),
            make_press(200, 0x00),
            make_press(300, 0x01),
            make_press(400, 0x02),
        ];

        let analysis = FrequencyAnalysis::from_events(&events);
        let top = analysis.top_keys(10);
        
        assert!((top[0].percentage - 50.0).abs() < 0.01);
        assert!((top[1].percentage - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_top_keys_limit() {
        let events = vec![
            make_press(100, 0x00),
            make_press(200, 0x01),
            make_press(300, 0x02),
            make_press(400, 0x03),
            make_press(500, 0x04),
        ];

        let analysis = FrequencyAnalysis::from_events(&events);
        let top2 = analysis.top_keys(2);
        assert_eq!(top2.len(), 2);
    }
}
