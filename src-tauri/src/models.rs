use std::collections::HashMap;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PasswordEntry {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip)]
    #[allow(dead_code)]
    pub raw: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportedFile {
    pub name: String,
    pub entries: Vec<PasswordEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryGroup {
    pub title: String,
    pub username: String,
    pub entries: Vec<(usize, PasswordEntry)>,
    pub resolved_source: Option<usize>,
    pub has_conflicts: bool,
    pub conflict_count: usize,
}

impl EntryGroup {
    pub fn compute_conflicts(&mut self) {
        if self.entries.len() < 2 {
            self.has_conflicts = false;
            self.conflict_count = 0;
            return;
        }
        let base = &self.entries[0].1;
        let mut count = 0;
        let getters: Vec<fn(&PasswordEntry) -> &str> = vec![
            |e| &e.username,
            |e| &e.password,
            |e| &e.url,
            |e| &e.notes,
            |e| &e.created_at,
            |e| &e.updated_at,
        ];
        for getter in &getters {
            let base_val = getter(base);
            for e in &self.entries[1..] {
                if getter(&e.1) != base_val {
                    count += 1;
                    break;
                }
            }
        }
        self.conflict_count = count;
        self.has_conflicts = count > 0;
    }
}
