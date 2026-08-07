use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PasswordEntry {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    pub raw: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ImportedFile {
    pub name: String,
    pub entries: Vec<PasswordEntry>,
}

#[derive(Debug, Clone)]
pub struct EntryGroup {
    pub title: String,
    pub username: String,
    pub entries: Vec<(usize, PasswordEntry)>,
    pub resolved_source: Option<usize>,
}

impl EntryGroup {
    pub fn has_conflicts(&self) -> bool {
        if self.entries.len() < 2 {
            return false;
        }
        let base = &self.entries[0].1;
        for e in &self.entries[1..] {
            if base.password != e.1.password
                || base.url != e.1.url
                || base.notes != e.1.notes
                || base.created_at != e.1.created_at
                || base.updated_at != e.1.updated_at
            {
                return true;
            }
        }
        false
    }

    pub fn get_conflicting_fields(&self) -> Vec<String> {
        if self.entries.len() < 2 {
            return vec![];
        }
        let mut conflicts = vec![];
        let base = &self.entries[0].1;
        let fields = ["Username", "Password", "URL", "Notes", "Created", "Updated"];
        let getters: Vec<fn(&PasswordEntry) -> &str> = vec![
            |e| &e.username,
            |e| &e.password,
            |e| &e.url,
            |e| &e.notes,
            |e| &e.created_at,
            |e| &e.updated_at,
        ];
        for (i, field) in fields.iter().enumerate() {
            let base_val = getters[i](base);
            for e in &self.entries[1..] {
                if getters[i](&e.1) != base_val {
                    conflicts.push(field.to_string());
                    break;
                }
            }
        }
        conflicts
    }
}
