use std::collections::HashMap;

use crate::models::{ImportedFile, PasswordEntry};

pub fn parse_csv(path: &str) -> Result<ImportedFile, String> {
    let p = std::path::Path::new(path);
    let name = p
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| format!("Bad CSV headers: {}", e))?
        .iter()
        .map(|h| h.trim().to_lowercase())
        .collect();

    let field_map = build_field_map(&headers);

    let mut entries = Vec::new();

    for (row_idx, result) in reader.records().enumerate() {
        let record = result.map_err(|e| format!("Row {} error: {}", row_idx + 1, e))?;
        let values: Vec<String> = record.iter().map(|v| v.trim().to_string()).collect();
        let raw: HashMap<String, String> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.clone(), values.get(i).cloned().unwrap_or_default()))
            .collect();

        entries.push(PasswordEntry {
            title: get_field(&values, &field_map, "title"),
            username: get_field(&values, &field_map, "username"),
            password: get_field(&values, &field_map, "password"),
            url: get_field(&values, &field_map, "url"),
            notes: get_field(&values, &field_map, "notes"),
            created_at: get_field(&values, &field_map, "created"),
            updated_at: get_field(&values, &field_map, "updated"),
            raw,
        });
    }

    Ok(ImportedFile { name, entries })
}

fn normalize(name: &str) -> String {
    name.to_lowercase().replace([' ', '_', '-'], "")
}

fn classify_header(name: &str) -> String {
    let n = normalize(name);

    if n.contains("loginpassword") { return "password".to_string(); }
    if n.contains("loginusername") { return "username".to_string(); }
    if n.contains("loginuri") { return "url".to_string(); }
    if n.contains("loginnname") { return "title".to_string(); }
    if n.contains("title") { return "title".to_string(); }
    if n.contains("username") { return "username".to_string(); }
    if n.contains("password") || n.contains("passwd") { return "password".to_string(); }
    if n.contains("website") { return "url".to_string(); }
    if n.contains("notes") || n.contains("note") { return "notes".to_string(); }
    if n.contains("extra") { return "notes".to_string(); }
    if n.contains("timepasswordchanged") { return "updated".to_string(); }
    if n.contains("timelastused") { return "updated".to_string(); }
    if n.contains("modifytime") { return "updated".to_string(); }
    if n.contains("lastmodified") { return "updated".to_string(); }
    if n.contains("updated_at") || n.contains("updated") { return "updated".to_string(); }
    if n.contains("timecreated") { return "created".to_string(); }
    if n.contains("createtime") { return "created".to_string(); }
    if n.contains("creation") { return "created".to_string(); }
    if n.contains("created_at") || n.contains("created") { return "created".to_string(); }
    if n.contains("uri") { return "url".to_string(); }
    if n.contains("url") { return "url".to_string(); }
    if n.contains("email") { return "username".to_string(); }
    if n.contains("user") { return "username".to_string(); }
    if n.contains("site") { return "title".to_string(); }
    if n.contains("loginname") { return "title".to_string(); }
    if n.contains("name") { return "title".to_string(); }
    if n.contains("favourite") || n.contains("favorite") || n.contains("fav") { return "fav".to_string(); }
    if n.contains("grouping") || n.contains("folder") || n.contains("group") { return "group".to_string(); }
    if n.contains("pass") { return "password".to_string(); }

    "other".to_string()
}

fn build_field_map(headers: &[String]) -> HashMap<String, usize> {
    let wanted = ["title", "username", "password", "url", "notes", "created", "updated"];
    let mut map: HashMap<String, usize> = HashMap::new();
    for (i, h) in headers.iter().enumerate() {
        let key = classify_header(h);
        if wanted.contains(&key.as_str()) {
            map.entry(key).or_insert(i);
        }
    }
    map
}

fn get_field(values: &[String], field_map: &HashMap<String, usize>, field: &str) -> String {
    if let Some(&idx) = field_map.get(field) {
        values.get(idx).cloned().unwrap_or_default()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_csv_str(csv_data: &str) -> (Vec<String>, Vec<String>, HashMap<String, usize>) {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(std::io::Cursor::new(csv_data));
        let headers: Vec<String> = reader.headers().unwrap().iter()
            .map(|h| h.trim().to_lowercase()).collect();
        let field_map = build_field_map(&headers);
        let record = reader.records().next().unwrap().unwrap();
        let values: Vec<String> = record.iter().map(|v| v.trim().to_string()).collect();
        (headers, values, field_map)
    }

    #[test]
    fn test_bitwarden() {
        let csv = "name,login_username,login_password,login_uri,notes\n\
                    GitHub,john@example.com,secret123,https://github.com,notes here";
        let (_h, values, fm) = parse_csv_str(csv);
        assert_eq!(get_field(&values, &fm, "title"), "GitHub");
        assert_eq!(get_field(&values, &fm, "username"), "john@example.com");
        assert_eq!(get_field(&values, &fm, "password"), "secret123");
        assert_eq!(get_field(&values, &fm, "url"), "https://github.com");
        assert_eq!(get_field(&values, &fm, "notes"), "notes here");
    }

    #[test]
    fn test_lastpass() {
        let csv = "url,username,password,extra,name,grouping\n\
                    https://github.com,john@example.com,mypass,note here,GitHub,Dev";
        let (_h, values, fm) = parse_csv_str(csv);
        assert_eq!(get_field(&values, &fm, "title"), "GitHub");
        assert_eq!(get_field(&values, &fm, "username"), "john@example.com");
        assert_eq!(get_field(&values, &fm, "password"), "mypass");
        assert_eq!(get_field(&values, &fm, "url"), "https://github.com");
        assert_eq!(get_field(&values, &fm, "notes"), "note here");
    }

    #[test]
    fn test_onepassword() {
        let csv = "title,website,username,password,notes,created_at,updated_at\n\
                    GitHub,https://github.com,john@example.com,secret456,my notes,2023-01-01,2024-06-01";
        let (_h, values, fm) = parse_csv_str(csv);
        assert_eq!(get_field(&values, &fm, "title"), "GitHub");
        assert_eq!(get_field(&values, &fm, "username"), "john@example.com");
        assert_eq!(get_field(&values, &fm, "password"), "secret456");
        assert_eq!(get_field(&values, &fm, "url"), "https://github.com");
        assert_eq!(get_field(&values, &fm, "notes"), "my notes");
        assert_eq!(get_field(&values, &fm, "created"), "2023-01-01");
        assert_eq!(get_field(&values, &fm, "updated"), "2024-06-01");
    }

    #[test]
    fn test_chrome() {
        let csv = "name,url,username,password\n\
                    GitHub,https://github.com,user1,chrome_pass";
        let (_h, values, fm) = parse_csv_str(csv);
        assert_eq!(get_field(&values, &fm, "title"), "GitHub");
        assert_eq!(get_field(&values, &fm, "username"), "user1");
        assert_eq!(get_field(&values, &fm, "password"), "chrome_pass");
        assert_eq!(get_field(&values, &fm, "url"), "https://github.com");
    }

    #[test]
    fn test_protonpass() {
        // Proton Pass export: name,url,email,password,note,createTime,modifyTime,vault
        // Two Reddit entries with different emails - should be separate matched groups
        let csv = "name,url,email,password,note,createTime,modifyTime,vault\n\
                    Reddit,,alice@proton.me,pass1,Reddit account,1772382589,1773373277,Personal";
        let (_h, values, fm) = parse_csv_str(csv);
        assert_eq!(get_field(&values, &fm, "title"), "Reddit");
        assert_eq!(get_field(&values, &fm, "username"), "alice@proton.me");
        assert_eq!(get_field(&values, &fm, "password"), "pass1");
        assert_eq!(get_field(&values, &fm, "created"), "1772382589");
        assert_eq!(get_field(&values, &fm, "updated"), "1773373277");
    }
}
