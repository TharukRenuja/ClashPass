use std::collections::HashMap;
use std::sync::Mutex;

use crate::models::{EntryGroup, ImportedFile};
use crate::parser::parse_csv;
use crate::export::export_to_csv;

pub struct AppState {
    pub files: Vec<ImportedFile>,
    pub groups: Vec<EntryGroup>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            groups: Vec::new(),
        }
    }
}

#[tauri::command]
pub fn import_files(paths: Vec<String>, state: tauri::State<'_, Mutex<AppState>>) -> Result<Vec<ImportedFile>, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let mut imported = Vec::new();

    for path in &paths {
        match parse_csv(path) {
            Ok(file) => {
                imported.push(file.clone());
                state.files.push(file);
            }
            Err(e) => return Err(format!("Error parsing {}: {}", path, e)),
        }
    }

    match_groups(&mut state);
    Ok(imported)
}

#[tauri::command]
pub fn remove_file(idx: usize, state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if idx < state.files.len() {
        state.files.remove(idx);
        match_groups(&mut state);
    }
    Ok(())
}

#[tauri::command]
pub fn get_files(state: tauri::State<'_, Mutex<AppState>>) -> Result<Vec<ImportedFile>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.files.clone())
}

#[tauri::command]
pub fn get_groups(state: tauri::State<'_, Mutex<AppState>>) -> Result<Vec<EntryGroup>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.groups.clone())
}

#[tauri::command]
pub fn resolve_group(group_idx: usize, entry_idx: usize, state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if let Some(group) = state.groups.get_mut(group_idx) {
        group.resolved_source = Some(entry_idx);
    }
    Ok(())
}

#[tauri::command]
pub fn clear_resolve(group_idx: usize, state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if let Some(group) = state.groups.get_mut(group_idx) {
        group.resolved_source = None;
    }
    Ok(())
}

#[tauri::command]
pub fn edit_field(
    group_idx: usize,
    entry_file_idx: usize,
    field: String,
    value: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if let Some(group) = state.groups.get_mut(group_idx) {
        if let Some((_, entry)) = group.entries.iter_mut().find(|(i, _)| *i == entry_file_idx) {
            match field.as_str() {
                "Username" => entry.username = value,
                "Password" => entry.password = value,
                "URL" => entry.url = value,
                "Notes" => entry.notes = value,
                "Created" => entry.created_at = value,
                "Updated" => entry.updated_at = value,
                _ => {}
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn export_csv(path: String, state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    export_to_csv(&state.groups, &path)
}

fn match_groups(state: &mut AppState) {
    let mut groups: Vec<EntryGroup> = Vec::new();
    let mut used: Vec<Vec<bool>> =
        state.files.iter().map(|f| vec![false; f.entries.len()]).collect();
    let mut key_map: HashMap<String, Vec<(usize, usize)>> = HashMap::new();

    for (fi, file) in state.files.iter().enumerate() {
        for (ei, entry) in file.entries.iter().enumerate() {
            let title = entry.title.to_lowercase().trim().to_string();
            let username = entry.username.to_lowercase().trim().to_string();
            if title.is_empty() || username.is_empty() {
                continue;
            }
            key_map
                .entry(format!("{}|{}", title, username))
                .or_default()
                .push((fi, ei));
        }
    }

    for matches in key_map.values() {
        if matches.is_empty() {
            continue;
        }
        let first = &state.files[matches[0].0].entries[matches[0].1];
        let mut group = EntryGroup {
            title: first.title.clone(),
            username: first.username.clone(),
            entries: Vec::new(),
            resolved_source: None,
            has_conflicts: false,
            conflict_count: 0,
        };
        for &(fi, ei) in matches {
            used[fi][ei] = true;
            group.entries.push((fi, state.files[fi].entries[ei].clone()));
        }
        group.compute_conflicts();
        groups.push(group);
    }

    for (fi, file) in state.files.iter().enumerate() {
        for (ei, entry) in file.entries.iter().enumerate() {
            if !used[fi][ei] {
                let mut group = EntryGroup {
                    title: entry.title.clone(),
                    username: entry.username.clone(),
                    entries: vec![(fi, entry.clone())],
                    resolved_source: None,
                    has_conflicts: false,
                    conflict_count: 0,
                };
                group.compute_conflicts();
                groups.push(group);
            }
        }
    }

    state.groups = groups;
}
