use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::models::keycode::KeyCode;
use crate::models::shorten_bundle_id;
use crate::storage::Database;

const MS_PER_DAY: i64 = 24 * 60 * 60 * 1000;

pub fn show_stats(db: &Database, days: Option<u32>) -> Result<()> {
    let since = days.map(|d| Utc::now().timestamp_millis() - d as i64 * MS_PER_DAY);

    let total = db.get_total_count(since)?;
    let presses = db.get_press_count(since)?;

    match days {
        Some(d) => println!("=== Lurk Statistics (last {} days) ===\n", d),
        None => println!("=== Lurk Statistics ===\n"),
    }

    if total == 0 {
        if days.is_some() {
            println!("No keystroke data recorded in this period.");
        } else {
            println!("No keystroke data recorded yet.");
            println!("\nMake sure the daemon is running:");
            println!("  launchctl list | grep lurk");
        }
        return Ok(());
    }

    println!("Total Events:     {}", total);
    println!("Key Presses:      {}", presses);
    println!("Key Releases:     {}", total - presses);

    if let Some((start, end)) = db.get_date_range(since)? {
        let start_dt = DateTime::from_timestamp_millis(start).unwrap_or_else(|| Utc::now());
        let end_dt = DateTime::from_timestamp_millis(end).unwrap_or_else(|| Utc::now());

        // Denominator is the span of actual data, not the requested --days
        // window, so avg/day stays honest when the window is sparsely filled.
        let duration = end_dt - start_dt;
        let days_recorded = duration.num_days().max(1);

        println!("\nDate Range:");
        println!("  Start: {}", start_dt.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("  End:   {}", end_dt.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("  Duration: {} days", days_recorded);

        let avg_per_day = presses / days_recorded;
        println!("\nAverage: {} presses/day", avg_per_day);
    }

    println!("\n--- Top 10 Keys ---");
    let top_keys = db.get_top_keys(10, since)?;
    for (i, (key_code, count)) in top_keys.iter().enumerate() {
        let key_name = KeyCode(*key_code).to_name();
        let pct = (*count as f64 / presses as f64) * 100.0;
        println!("{:2}. {:15} {:>8} ({:.1}%)", i + 1, key_name, count, pct);
    }

    println!("\n--- Top 5 Applications ---");
    let top_apps = db.get_top_applications(5, since)?;
    for (i, (app, count)) in top_apps.iter().enumerate() {
        let app_short = shorten_bundle_id(app);
        let pct = (*count as f64 / presses as f64) * 100.0;
        println!("{:2}. {:25} {:>8} ({:.1}%)", i + 1, app_short, count, pct);
    }

    Ok(())
}
