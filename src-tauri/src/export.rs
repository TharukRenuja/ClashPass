use std::time::SystemTime;

use crate::models::EntryGroup;

fn now_iso() -> String {
    let Ok(dur) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
        return "1970-01-01T00:00:00Z".into();
    };
    let secs = dur.as_secs();
    let days = secs / 86400;
    let time = secs % 86400;
    let h = time / 3600;
    let m = (time % 3600) / 60;
    let s = time % 60;
    let mut y = 1970;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days: [u32; 12] = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u64;
    for &md in &month_days {
        if remaining < md as u64 {
            break;
        }
        remaining -= md as u64;
        month += 1;
    }
    let day = remaining + 1;
    format!("{y:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0) && (y % 100 != 0 || y % 400 == 0)
}

pub fn export_to_csv(groups: &[EntryGroup], path: &str) -> Result<(), String> {
    let mut wtr = csv::WriterBuilder::new()
        .from_path(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    wtr.write_record([
        "Title",
        "Username",
        "Password",
        "URL",
        "Notes",
        "Created At",
        "Updated At",
    ])
    .map_err(|e| format!("Write error: {}", e))?;

    let ts = now_iso();

    for group in groups {
        let entry = match group.resolved_source {
            Some(file_idx) => {
                let found = group.entries.iter().find(|(i, _)| *i == file_idx);
                found.map(|(_, e)| e)
            }
            None => group.entries.first().map(|(_, e)| e),
        };

        if let Some(e) = entry {
            let created = if e.created_at.is_empty() { &ts } else { &e.created_at };
            let updated = if e.updated_at.is_empty() { &ts } else { &e.updated_at };
            wtr.write_record([
                &group.title,
                &group.username,
                &e.password,
                &e.url,
                &e.notes,
                created,
                updated,
            ])
            .map_err(|e| format!("Write error: {}", e))?;
        }
    }

    wtr.flush().map_err(|e| format!("Flush error: {}", e))?;
    Ok(())
}
