use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::models::keycode::KeyCode;
use crate::storage::Database;

pub fn export_csv<P: AsRef<Path>>(db: &Database, output_path: P) -> Result<()> {
    let events = db.get_all_events()?;
    let mut file = File::create(output_path.as_ref())?;

    writeln!(file, "timestamp,key_code,key_name,event_type,modifiers,application")?;

    for event in &events {
        let key_name = KeyCode(event.key_code).to_name();
        let modifiers_str = event
            .modifiers
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(";");

        writeln!(
            file,
            "{},{},{},{},{},{}",
            event.timestamp,
            event.key_code,
            key_name,
            event.event_type,
            modifiers_str,
            event.application.replace(',', ";")
        )?;
    }

    println!(
        "Exported {} events to {}",
        events.len(),
        output_path.as_ref().display()
    );

    Ok(())
}

pub fn export_json<P: AsRef<Path>>(db: &Database, output_path: P) -> Result<()> {
    let events = db.get_all_events()?;
    let date_range = db.get_date_range()?;

    let export_data = serde_json::json!({
        "metadata": {
            "export_date": chrono::Utc::now().to_rfc3339(),
            "total_events": events.len(),
            "date_range": date_range.map(|(start, end)| {
                serde_json::json!({
                    "start": start,
                    "end": end
                })
            })
        },
        "events": events.iter().map(|e| {
            serde_json::json!({
                "timestamp": e.timestamp,
                "key_code": e.key_code,
                "key_name": KeyCode(e.key_code).to_name(),
                "event_type": e.event_type,
                "modifiers": e.modifiers,
                "application": e.application
            })
        }).collect::<Vec<_>>()
    });

    let file = File::create(output_path.as_ref())?;
    serde_json::to_writer_pretty(file, &export_data)?;

    println!(
        "Exported {} events to {}",
        events.len(),
        output_path.as_ref().display()
    );

    Ok(())
}
