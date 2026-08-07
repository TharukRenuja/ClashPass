use chrono::Utc;

use crate::models::EntryGroup;

fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
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
