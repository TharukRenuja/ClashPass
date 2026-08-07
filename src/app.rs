use std::collections::HashMap;

use eframe::egui::*;

use crate::export::export_to_csv;
use crate::models::EntryGroup;
use crate::parser::parse_csv;

const CARD_BG: Color32 = Color32::from_rgb(28, 28, 40);
const CARD_BG_SELECTED: Color32 = Color32::from_rgb(40, 40, 60);
const ACCENT: Color32 = Color32::from_rgb(233, 69, 96);
const GREEN: Color32 = Color32::from_rgb(76, 175, 80);
const YELLOW: Color32 = Color32::from_rgb(255, 212, 96);
const GRAY: Color32 = Color32::from_rgb(120, 120, 140);
const SURFACE: Color32 = Color32::from_rgb(20, 20, 30);
const CONFLICT_BG: Color32 = Color32::from_rgb(70, 25, 25);
const SELECTED_BG: Color32 = Color32::from_rgb(35, 70, 45);

#[derive(Default)]
pub struct PasswordComparerApp {
    files: Vec<crate::models::ImportedFile>,
    groups: Vec<EntryGroup>,
    selected_group: usize,
    show_conflicts_only: bool,
    search_query: String,
    status_message: String,
    editing_field: Option<String>,
    editing_entry: Option<usize>,
    edit_buffer: String,
}

impl PasswordComparerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn import_file(&mut self) {
        let files = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .add_filter("All", &["*"])
            .set_title("Select password export CSV file")
            .pick_file();

        if let Some(path) = files {
            match parse_csv(path.to_str().unwrap_or("")) {
                Ok(file) => {
                    let name = file.name.clone();
                    let count = file.entries.len();
                    self.files.push(file);
                    self.status_message = format!("Loaded '{}' ({} entries)", name, count);
                    self.match_groups();
                }
                Err(e) => self.status_message = format!("Error: {}", e),
            }
        }
    }

    fn remove_file(&mut self, idx: usize) {
        if idx < self.files.len() {
            let name = self.files[idx].name.clone();
            self.files.remove(idx);
            self.status_message = format!("Removed '{}'", name);
            self.match_groups();
        }
    }

    fn match_groups(&mut self) {
        let mut groups: Vec<EntryGroup> = Vec::new();
        let mut used: Vec<Vec<bool>> =
            self.files.iter().map(|f| vec![false; f.entries.len()]).collect();
        let mut key_map: HashMap<String, Vec<(usize, usize)>> = HashMap::new();

        for (fi, file) in self.files.iter().enumerate() {
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
            let first = &self.files[matches[0].0].entries[matches[0].1];
            let mut group = EntryGroup {
                title: first.title.clone(),
                username: first.username.clone(),
                entries: Vec::new(),
                resolved_source: None,
            };
            for &(fi, ei) in matches {
                used[fi][ei] = true;
                group.entries.push((fi, self.files[fi].entries[ei].clone()));
            }
            groups.push(group);
        }

        for (fi, file) in self.files.iter().enumerate() {
            for (ei, entry) in file.entries.iter().enumerate() {
                if !used[fi][ei] {
                    groups.push(EntryGroup {
                        title: entry.title.clone(),
                        username: entry.username.clone(),
                        entries: vec![(fi, entry.clone())],
                        resolved_source: None,
                    });
                }
            }
        }

        self.groups = groups;
        self.selected_group = self.selected_group.min(self.groups.len().saturating_sub(1));
    }

    fn resolve_group(&mut self, group_idx: usize, file_idx: usize) {
        if group_idx < self.groups.len() {
            self.groups[group_idx].resolved_source = Some(file_idx);
        }
    }

    fn export_csv(&mut self) {
        if self.groups.is_empty() {
            self.status_message = "No data to export".to_string();
            return;
        }
        let file = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_title("Save merged CSV")
            .save_file();
        if let Some(path) = file {
            self.commit_edit();
            match export_to_csv(&self.groups, path.to_str().unwrap_or("")) {
                Ok(_) => self.status_message = "Exported successfully!".to_string(),
                Err(e) => self.status_message = format!("Export error: {}", e),
            }
        }
    }

    fn begin_edit(&mut self, field: &str, entry_idx: usize, value: &str) {
        self.commit_edit();
        self.editing_field = Some(field.to_string());
        self.editing_entry = Some(entry_idx);
        self.edit_buffer = value.to_string();
    }

    fn commit_edit(&mut self) {
        if let (Some(field), Some(entry_idx)) = (&self.editing_field, self.editing_entry) {
            let sel = self.selected_group.min(self.groups.len().saturating_sub(1));
            if let Some(group) = self.groups.get_mut(sel) {
                if let Some((_, entry)) = group.entries.get_mut(entry_idx) {
                    let val = &self.edit_buffer;
                    match field.as_str() {
                        "Username" => entry.username = val.clone(),
                        "Password" => entry.password = val.clone(),
                        "URL" => entry.url = val.clone(),
                        "Notes" => entry.notes = val.clone(),
                        "Created" => entry.created_at = val.clone(),
                        "Updated" => entry.updated_at = val.clone(),
                        _ => {}
                    }
                }
            }
        }
        self.editing_field = None;
        self.editing_entry = None;
        self.edit_buffer.clear();
    }

    fn status_badge(conflicts: bool, resolved: bool) -> (Color32, &'static str) {
        if resolved {
            (Color32::from_rgb(40, 80, 60), "Resolved")
        } else if conflicts {
            (Color32::from_rgb(80, 30, 30), "Conflicts")
        } else {
            (Color32::from_rgb(30, 60, 40), "Synced")
        }
    }
}

impl eframe::App for PasswordComparerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_style(egui::Style {
            visuals: egui::Visuals {
                dark_mode: true,
                window_rounding: Rounding::same(8.0),
                menu_rounding: Rounding::same(6.0),
                panel_fill: SURFACE,
                faint_bg_color: CARD_BG,
                extreme_bg_color: SURFACE,
                window_fill: SURFACE,
                override_text_color: Some(Color32::from_gray(210)),
                ..Default::default()
            },
            spacing: egui::Spacing {
                item_spacing: Vec2::new(8.0, 6.0),
                button_padding: Vec2::new(10.0, 4.0),
                indent: 12.0,
                ..Default::default()
            },
            ..Default::default()
        });

        // ═══════════════════ TOOLBAR ═══════════════════
        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            Frame::none()
                .fill(SURFACE)
                .inner_margin(Margin::symmetric(12.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚔ ClashPass").strong().size(16.0).color(ACCENT));

                        ui.separator();

                        let btn_import = Button::new("📂  Import")
                            .fill(Color32::from_rgb(40, 40, 55))
                            .rounding(Rounding::same(6.0));
                        if ui.add(btn_import).clicked() {
                            self.import_file();
                        }

                        let can_export = !self.groups.is_empty();
                        let btn_export = Button::new("💾  Export")
                            .fill(Color32::from_rgb(40, 40, 55))
                            .rounding(Rounding::same(6.0));
                        if ui.add_enabled(can_export, btn_export).clicked() {
                            self.export_csv();
                        }

                        ui.separator();

                        ui.toggle_value(&mut self.show_conflicts_only, "🔍  Conflicts only");

                        if !self.groups.is_empty() {
                            ui.add(
                                TextEdit::singleline(&mut self.search_query)
                                    .hint_text("🔎  Filter entries...")
                                    .desired_width(180.0)
                                    .margin(Margin::symmetric(6.0, 2.0)),
                            );
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if !self.status_message.is_empty() {
                                ui.label(
                                    RichText::new(&self.status_message)
                                        .size(12.0)
                                        .color(GRAY),
                                );
                            }
                            if !self.files.is_empty() {
                                let txt = format!("📄 {} file(s)", self.files.len());
                                Frame::none()
                                    .fill(Color32::from_rgb(40, 40, 55))
                                    .rounding(Rounding::same(12.0))
                                    .inner_margin(Margin::symmetric(8.0, 3.0))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(txt).size(12.0).color(GRAY));
                                    });
                            }
                        });
                    });
                });
        });

        // ═══════════════════ FILE CHIPS ═══════════════════
        if !self.files.is_empty() {
            TopBottomPanel::top("file_chips").show(ctx, |ui| {
                Frame::none()
                    .fill(Color32::from_rgb(16, 16, 26))
                    .inner_margin(Margin::symmetric(12.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("📂  Sources:").size(12.0).color(GRAY));
                            let mut remove_idx = None;
                            for (i, file) in self.files.iter().enumerate() {
                                let chip_color = Color32::from_rgb(35, 35, 50);
                                let resp = Frame::none()
                                    .fill(chip_color)
                                    .rounding(Rounding::same(14.0))
                                    .inner_margin(Margin::symmetric(8.0, 3.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&file.name).size(12.0).strong(),
                                            );
                                            ui.label(
                                                RichText::new(format!("({})", file.entries.len()))
                                                    .size(11.0)
                                                    .color(GRAY),
                                            );
                                            let close = Button::new("✕")
                                                .small()
                                                .fill(Color32::TRANSPARENT);
                                            if ui.add(close).clicked() {
                                                remove_idx = Some(i);
                                            }
                                        });
                                    });
                                let _ = resp.response.on_hover_text("Click to remove");
                            }
                            if let Some(idx) = remove_idx {
                                self.remove_file(idx);
                            }
                        });
                    });
            });
        }

        // ═══════════════════ EMPTY STATE ═══════════════════
        if self.files.is_empty() {
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.25);
                    ui.label(RichText::new("⚔ ClashPass").size(36.0).color(ACCENT).strong());
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Password Conflict Resolver")
                            .size(18.0)
                            .color(GRAY),
                    );
                    ui.add_space(24.0);
                    let btn =
                        Button::new("📂  Import a password export CSV")
                            .min_size(Vec2::new(220.0, 40.0))
                            .fill(Color32::from_rgb(50, 35, 40))
                            .rounding(Rounding::same(8.0));
                    if ui.add(btn).clicked() {
                        self.import_file();
                    }
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("Supports Bitwarden · LastPass · 1Password · Proton Pass · Chrome · Firefox · more")
                            .size(13.0)
                            .color(GRAY),
                    );
                });
            });
            return;
        }

        // ═══════════════════ SIDE PANEL: Entry List ═══════════════════
        SidePanel::left("entry_list")
            .resizable(true)
            .default_width(240.0)
            .min_width(180.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Entries").strong().size(14.0).color(ACCENT));
                    ui.separator();

                    if self.groups.is_empty() {
                        ui.label(RichText::new("No entries found").size(12.0).color(GRAY));
                        return;
                    }

                    let sel = self.selected_group.min(self.groups.len().saturating_sub(1));
                    let query = self.search_query.to_lowercase();
                    let filtered: Vec<usize> = self
                        .groups
                        .iter()
                        .enumerate()
                        .filter(|(_, g)| {
                            if self.show_conflicts_only && !g.has_conflicts() {
                                return false;
                            }
                            if !query.is_empty() {
                                let matches = g.title.to_lowercase().contains(&query)
                                    || g.username.to_lowercase().contains(&query);
                                if !matches {
                                    return false;
                                }
                            }
                            true
                        })
                        .map(|(i, _)| i)
                        .collect();

                    ScrollArea::vertical()
                        .id_salt("entry_scroll")
                        .show(ui, |ui| {
                            for &gi in &filtered {
                                let group = &self.groups[gi];
                                let conflicts = group.has_conflicts();
                                let resolved = group.resolved_source.is_some();
                                let is_selected = gi == sel;
                                let (badge_bg, badge_txt) = Self::status_badge(conflicts, resolved);

                                let card_color = if is_selected {
                                    CARD_BG_SELECTED
                                } else {
                                    CARD_BG
                                };

                                let card = Frame::none()
                                    .fill(card_color)
                                    .rounding(Rounding::same(6.0))
                                    .stroke(if is_selected {
                                        Stroke::new(1.5, ACCENT)
                                    } else {
                                        Stroke::new(1.0, Color32::from_rgb(40, 40, 55))
                                    })
                                    .inner_margin(Margin::symmetric(10.0, 8.0));

                                let resp = card
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(&group.title)
                                                        .size(13.0)
                                                        .strong(),
                                                );
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    Frame::none()
                                                        .fill(badge_bg)
                                                        .rounding(Rounding::same(4.0))
                                                        .inner_margin(Margin::symmetric(5.0, 2.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                RichText::new(badge_txt)
                                                                    .size(10.0)
                                                                    .color(Color32::WHITE),
                                                            );
                                                        });
                                                });
                                            });
                                            if !group.username.is_empty() {
                                                ui.label(
                                                    RichText::new(&group.username)
                                                        .size(11.0)
                                                        .color(GRAY),
                                                );
                                            }
                                            if conflicts {
                                                let fields = group.get_conflicting_fields();
                                                ui.label(
                                                    RichText::new(format!(
                                                        "⚠ {} differ{}",
                                                        fields.len(),
                                                        if fields.len() == 1 { "s" } else { "" }
                                                    ))
                                                    .size(11.0)
                                                    .color(YELLOW),
                                                );
                                            } else if resolved {
                                                ui.label(
                                                    RichText::new("✔ Resolved")
                                                        .size(11.0)
                                                        .color(GREEN),
                                                );
                                            }
                                        });
                                    })
                                    .response;

                                let click_resp = ui.interact(resp.rect, ui.auto_id_with("card_sel"), Sense::click());
                                if click_resp.clicked() {
                                    self.commit_edit();
                                    self.selected_group = gi;
                                }
                                ui.add_space(4.0);
                            }
                        });
                });
            });

        // ═══════════════════ COMPARISON TABLE ═══════════════════
        CentralPanel::default().show(ctx, |ui| {
            if self.groups.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.label(RichText::new("📭 No entries found").size(20.0).color(GRAY));
                });
                return;
            }

            let sel = self.selected_group.min(self.groups.len().saturating_sub(1));

            // Clone group data upfront to avoid borrow conflicts
            let group_title = self.groups[sel].title.clone();
            let group_username = self.groups[sel].username.clone();
            let group_entries: Vec<(usize, crate::models::PasswordEntry)> =
                self.groups[sel].entries.clone();
            let group_conflicts = self.groups[sel].has_conflicts();
            let group_conflict_fields = self.groups[sel].get_conflicting_fields();
            let group_num_files = group_entries.len();
            let group_has_resolved = self.groups[sel].resolved_source.is_some();
            let group_resolved_file = self.groups[sel].resolved_source.unwrap_or(0);

            if group_entries.is_empty() {
                ui.label("No entry data");
                return;
            }

            let file_labels: Vec<String> = {
                let mut seen: HashMap<usize, usize> = HashMap::new();
                group_entries
                    .iter()
                    .map(|(fi, _)| {
                        let base = if *fi < self.files.len() {
                            self.files[*fi].name.clone()
                        } else {
                            format!("File {}", fi)
                        };
                        let nth = seen.entry(*fi).or_insert(0);
                        *nth += 1;
                        let total = group_entries.iter().filter(|(i, _)| *i == *fi).count();
                        if total > 1 {
                            format!("{} ({})", base, nth)
                        } else {
                            base
                        }
                    })
                    .collect()
            };

            // ─── Header ───
            ui.horizontal(|ui| {
                ui.label(RichText::new(&group_title).size(18.0).strong());
                if !group_username.is_empty() {
                    ui.label(
                        RichText::new(format!("({})", group_username))
                            .size(14.0)
                            .color(GRAY),
                    );
                }
                if group_conflicts {
                    Frame::none()
                        .fill(Color32::from_rgb(80, 30, 30))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("⚠ Conflicts")
                                    .size(11.0)
                                    .strong(),
                            );
                        });
                } else {
                    Frame::none()
                        .fill(Color32::from_rgb(30, 60, 40))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("✔ Synced")
                                    .size(11.0)
                                    .strong(),
                            );
                        });
                }
            });
            ui.separator();

            // ─── Comparison Grid ───
            let fields = ["Username", "Password", "URL", "Notes", "Created", "Updated"];
            let getters: Vec<fn(&crate::models::PasswordEntry) -> &str> = vec![
                |e| &e.username,
                |e| &e.password,
                |e| &e.url,
                |e| &e.notes,
                |e| &e.created_at,
                |e| &e.updated_at,
            ];

            ScrollArea::vertical()
                .id_salt("comp_table")
                .show(ui, |ui| {
                    Grid::new("comp_grid")
                        .striped(false)
                        .min_col_width(90.0)
                        .max_col_width(250.0)
                        .show(ui, |ui| {
                            // Column headers
                            Frame::none()
                                .fill(Color32::from_rgb(30, 30, 45))
                                .inner_margin(Margin::symmetric(5.0, 3.0))
                                .show(ui, |ui| {
                                    ui.strong(RichText::new("Field").size(11.0));
                                });
                            for (vi, lbl) in file_labels.iter().enumerate() {
                                let sep = if vi > 0 {
                                    Stroke::new(1.0, Color32::from_rgb(50, 50, 65))
                                } else {
                                    Stroke::new(0.0, Color32::TRANSPARENT)
                                };
                                Frame::none()
                                    .fill(Color32::from_rgb(30, 30, 45))
                                    .stroke(sep)
                                    .inner_margin(Margin::symmetric(6.0, 3.0))
                                    .show(ui, |ui| {
                                        ui.strong(RichText::new(lbl).size(11.0));
                                    });
                            }
                            ui.end_row();

                            // Field rows
                            for (fi, field) in fields.iter().enumerate() {
                                let has_conflict = group_conflict_fields.contains(&field.to_string());
                                let row_color = if fi % 2 == 0 {
                                    Color32::from_rgb(26, 26, 38)
                                } else {
                                    Color32::from_rgb(22, 22, 34)
                                };

                                Frame::none()
                                    .fill(row_color)
                                    .inner_margin(Margin::symmetric(4.0, 2.0))
                                    .show(ui, |ui| {
                                        let icon = match *field {
                                            "Username" => "👤",
                                            "Password" => "🔑",
                                            "URL" => "🌐",
                                            "Notes" => "📝",
                                            "Created" => "📅",
                                            "Updated" => "🔄",
                                            _ => "",
                                        };
                                        ui.label(
                                            RichText::new(format!("{} {}", icon, field))
                                                .size(12.0)
                                                .strong(),
                                        );

                                        for (vi, (file_idx, entry)) in
                                            group_entries.iter().enumerate()
                                        {
                                            let val = getters[fi](entry);
                                            let val_str = if val.is_empty() { "—" } else { val };
                                            let is_selected = group_has_resolved
                                                && *file_idx == group_resolved_file;

                                            let bg = if has_conflict && is_selected {
                                                SELECTED_BG
                                            } else if has_conflict {
                                                CONFLICT_BG
                                            } else {
                                                Color32::TRANSPARENT
                                            };

                                            let sep = if vi > 0 {
                                                Stroke::new(1.0, Color32::from_rgb(50, 50, 65))
                                            } else {
                                                Stroke::new(0.0, Color32::TRANSPARENT)
                                            };

                                            Frame::none()
                                                .stroke(sep)
                                                .inner_margin(Margin::symmetric(6.0, 2.0))
                                                .show(ui, |ui| {
                                                    let is_editing = self.editing_field.as_deref() == Some(field)
                                                        && self.editing_entry == Some(vi);

                                                    if is_editing {
                                                        let resp = ui.add_sized(
                                                            Vec2::new(120.0, 22.0),
                                                            TextEdit::singleline(&mut self.edit_buffer)
                                                                .font(TextStyle::Monospace)
                                                                .margin(Margin::symmetric(2.0, 0.0)),
                                                        );
                                                        let enter = resp.lost_focus()
                                                            && ui.input(|i| i.key_pressed(Key::Enter));
                                                        if enter {
                                                            self.commit_edit();
                                                            ui.memory_mut(|mem| mem.surrender_focus(resp.id));
                                                        }
                                                    } else {
                                                        let resp = ui.add(
                                                            Label::new(
                                                                RichText::new(val_str)
                                                                    .background_color(bg)
                                                                    .size(12.0),
                                                            )
                                                            .sense(Sense::click()),
                                                        );
                                                        if has_conflict && resp.clicked() {
                                                            self.resolve_group(sel, *file_idx);
                                                        }
                                                        if has_conflict {
                                                            resp.on_hover_text("Select this");
                                                        }

                                                        let editable = matches!(*field, "Username" | "Password" | "URL" | "Notes");
                                                        if editable {
                                                            let edit_resp = ui.add(
                                                                Label::new(RichText::new("✏️").size(13.0))
                                                                    .sense(Sense::click()),
                                                            )
                                                            .on_hover_text("Edit");
                                                            if edit_resp.clicked() {
                                                                self.begin_edit(field, vi, val);
                                                            }
                                                        }
                                                    }
                                                });
                                        }

                                        if has_conflict && group_num_files > 1 {
                                            ui.horizontal(|ui| {
                                                for (vi, (file_idx, _)) in
                                                    group_entries.iter().enumerate()
                                                {
                                                    let selected =
                                                        group_has_resolved
                                                            && *file_idx == group_resolved_file;
                                                    let txt = format!(
                                                        "{} {}",
                                                        if selected { "●" } else { "○" },
                                                        file_labels[vi]
                                                    );
                                                    let c = if selected { ACCENT } else { GRAY };
                                                    let r = ui
                                                        .selectable_label(
                                                            selected,
                                                            RichText::new(txt)
                                                                .size(10.0)
                                                                .color(c),
                                                        );
                                                    if r.clicked() {
                                                        self.resolve_group(sel, *file_idx);
                                                    }
                                                }
                                            });
                                        } else {
                                            ui.label("");
                                        }
                                    });
                                ui.end_row();
                            }

                            // Extra fields collapsible
                            if let Some(first_entry) =
                                group_entries.first().map(|(_, e)| e)
                            {
                                let standard = [
                                    "title", "username", "password", "url",
                                    "notes", "created_at", "updated_at", "created",
                                    "updated", "login_username", "login_password",
                                    "login_uri", "login_name", "name", "website",
                                    "uri", "note", "extra", "grouping", "folder",
                                    "favorite", "favourite", "fav", "type",
                                    "fields", "reprompt", "totp", "timecreated",
                                    "timepasswordchanged", "timelastused",
                                    "last_modified", "group", "email", "createtime",
                                    "modifytime", "vault",
                                ];
                                let extras: Vec<&String> = first_entry
                                    .raw
                                    .keys()
                                    .filter(|k| !standard.contains(&k.as_str()))
                                    .collect();
                                if !extras.is_empty() {
                                    ui.end_row();
                                    ui.label("");
                                    CollapsingHeader::new("📎 Extra fields")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            for col in &extras {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new(col.as_str())
                                                            .italics()
                                                            .size(11.0)
                                                            .color(GRAY),
                                                    );
                                                    for (_fi, e) in
                                                        &group_entries
                                                    {
                                                        let v = e
                                                            .raw
                                                            .get(col.as_str())
                                                            .cloned()
                                                            .unwrap_or_default();
                                                        let v = if v.is_empty() {
                                                            "—"
                                                        } else {
                                                            &v
                                                        };
                                                        ui.label(
                                                            RichText::new(v)
                                                                .size(11.0),
                                                        );
                                                    }
                                                });
                                            }
                                        });
                                }
                            }
                        });
                });

            if group_has_resolved && group_num_files > 1 {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Resolved from ").size(11.0).color(GRAY));
                    if let Some((_, _)) = group_entries.iter().find(|(i, _)| *i == group_resolved_file) {
                        ui.label(RichText::new(&file_labels[group_entries.iter().position(|(i, _)| *i == group_resolved_file).unwrap_or(0)]).size(11.0).strong().color(GREEN));
                    }
                    if ui.button(RichText::new("✕").size(11.0).color(GRAY)).clicked() {
                        self.groups[sel].resolved_source = None;
                    }
                });
            }
        });
    }
}
