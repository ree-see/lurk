use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::models::keycode::KeyCode;
use crate::storage::Database;

fn validate_export_path<P: AsRef<Path>>(output_path: P) -> Result<std::path::PathBuf> {
    let path = output_path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let canonical_parent = absolute_path
        .parent()
        .ok_or_else(|| anyhow!("Invalid path: no parent directory"))?;

    if !canonical_parent.exists() {
        return Err(anyhow!(
            "Parent directory does not exist: {}",
            canonical_parent.display()
        ));
    }

    let canonical_parent = canonical_parent.canonicalize()?;

    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;

    if !canonical_parent.starts_with(&home_dir) {
        return Err(anyhow!(
            "Security: export path must be within user's home directory. \
             Attempted path: {}",
            canonical_parent.display()
        ));
    }

    let filename = absolute_path
        .file_name()
        .ok_or_else(|| anyhow!("Invalid path: no filename"))?;

    Ok(canonical_parent.join(filename))
}

pub fn export_csv<P: AsRef<Path>>(db: &Database, output_path: P) -> Result<()> {
    let safe_path = validate_export_path(&output_path)?;
    let events = db.get_all_events()?;
    let mut file = File::create(&safe_path)?;

    writeln!(
        file,
        "timestamp,key_code,key_name,event_type,modifiers,application"
    )?;

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
            csv_field(&key_name),
            event.event_type,
            csv_field(&modifiers_str),
            csv_field(&event.application)
        )?;
    }

    println!(
        "Exported {} events to {}",
        events.len(),
        safe_path.display()
    );

    Ok(())
}

/// Escape a CSV field per RFC 4180: quote when it contains a comma, quote,
/// or line break, doubling any embedded quotes.
fn csv_field(s: &str) -> std::borrow::Cow<'_, str> {
    if s.contains([',', '"', '\n', '\r']) {
        std::borrow::Cow::Owned(format!("\"{}\"", s.replace('"', "\"\"")))
    } else {
        std::borrow::Cow::Borrowed(s)
    }
}

pub fn export_json<P: AsRef<Path>>(db: &Database, output_path: P) -> Result<()> {
    let safe_path = validate_export_path(&output_path)?;
    let events = db.get_all_events()?;
    let date_range = db.get_date_range(None)?;

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

    let file = File::create(&safe_path)?;
    serde_json::to_writer_pretty(file, &export_data)?;

    println!(
        "Exported {} events to {}",
        events.len(),
        safe_path.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EventType, KeystrokeEvent};

    #[test]
    fn test_csv_field_plain_passthrough() {
        assert_eq!(csv_field("Backspace"), "Backspace");
        assert_eq!(csv_field("com.apple.Safari"), "com.apple.Safari");
        assert_eq!(csv_field(""), "");
    }

    #[test]
    fn test_csv_field_comma_is_quoted() {
        assert_eq!(csv_field(","), "\",\"");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
    }

    #[test]
    fn test_csv_field_quote_is_doubled_and_quoted() {
        assert_eq!(csv_field("\""), "\"\"\"\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_csv_field_newline_is_quoted() {
        assert_eq!(csv_field("a\nb"), "\"a\nb\"");
        assert_eq!(csv_field("a\rb"), "\"a\rb\"");
    }

    /// Minimal RFC 4180 line parser for assertions.
    fn parse_csv_line(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '"' if in_quotes && chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => fields.push(std::mem::take(&mut field)),
                _ => field.push(c),
            }
        }
        fields.push(field);
        fields
    }

    #[test]
    fn test_export_csv_comma_key_produces_six_fields() {
        // key_code 43 renders as a literal "," — the field must be quoted
        // so the row still parses to exactly 6 fields.
        let db = Database::new(":memory:").unwrap();
        db.insert_event(&KeystrokeEvent {
            timestamp: 1000,
            key_code: 43,
            event_type: EventType::Press,
            modifiers: vec![],
            application: "com.test.app".to_string(),
        })
        .unwrap();

        // validate_export_path requires the file to live under $HOME.
        let home = dirs::home_dir().expect("home dir required for this test");
        let dir = home.join(format!(".lurk-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("export.csv");

        // Drop guard so the dir is removed even if an assert panics.
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(dir.clone());

        export_csv(&db, &out).unwrap();
        let contents = std::fs::read_to_string(&out).unwrap();

        let data_line = contents.lines().nth(1).expect("one data line");
        let fields = parse_csv_line(data_line);
        assert_eq!(fields.len(), 6, "line: {:?}", data_line);
        assert_eq!(fields[2], ",");
        assert_eq!(fields[5], "com.test.app");
    }
}
