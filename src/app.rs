use std::collections::HashMap;

use eframe::egui::*;
use egui_phosphor::regular as ph;

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
    show_passwords: bool,
    logo: Option<TextureHandle>,
}

impl PasswordComparerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load the Phosphor icon font so icon glyphs actually render instead
        // of showing up as missing-glyph squares (egui's default font has no
        // emoji coverage at all).
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        // Load logo
        let logo_bytes = include_bytes!("../icons/clashpass_256.png");
        let decoder = png::Decoder::new(std::io::Cursor::new(logo_bytes));
        let mut reader = decoder.read_info().expect("Failed to decode logo PNG");
        let mut buf = vec![0u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).expect("Failed to read logo frame");
        buf.truncate(info.buffer_size());
        let image = ColorImage::from_rgba_unmultiplied(
            [info.width as usize, info.height as usize],
            &buf,
        );
        let logo = cc.egui_ctx.load_texture("logo", image, TextureOptions::LINEAR);

        Self {
            logo: Some(logo),
            ..Default::default()
        }
    }

    fn import_file(&mut self) {
        let files = native_dialog::DialogBuilder::file()
            .add_filter("CSV", &["csv"])
            .add_filter("All", &["*"])
            .set_title("Select password export CSV file")
            .open_single_file()
            .show();

        if let Ok(Some(path)) = files {
            self.import_path(path.to_str().unwrap_or(""));
        }
    }

    fn import_path(&mut self, path: &str) {
        match parse_csv(path) {
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
        let file = native_dialog::DialogBuilder::file()
            .add_filter("CSV", &["csv"])
            .set_title("Save merged CSV")
            .save_single_file()
            .show();
        if let Ok(Some(path)) = file {
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();
        ctx.request_repaint();

        // Handle drag and drop
        let dropped: Vec<std::path::PathBuf> = ctx.input(|i| i.raw.dropped_files.clone())
            .into_iter()
            .filter_map(|f| f.path)
            .collect();
        for path in dropped {
            if let Some(ext) = path.extension() {
                if ext == "csv" {
                    self.import_path(path.to_str().unwrap_or(""));
                }
            }
        }

        ctx.set_style_of(egui::Theme::Dark, egui::Style {
            visuals: egui::Visuals {
                dark_mode: true,
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
        Panel::top("toolbar").show(ui, |ui| {
            Frame::new()
                .fill(SURFACE)
                .inner_margin(Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("ClashPass").strong().size(16.0).color(ACCENT));

                        ui.separator();

                        let btn_import = Button::new(&format!("{}  Import", ph::FOLDER_OPEN))
                            .fill(Color32::from_rgb(40, 40, 55));
                        if ui.add(btn_import).clicked() {
                            self.import_file();
                        }

                        let can_export = !self.groups.is_empty();
                        let btn_export = Button::new(&format!("{}  Export", ph::FLOPPY_DISK))
                            .fill(Color32::from_rgb(40, 40, 55));
                        if ui.add_enabled(can_export, btn_export).clicked() {
                            self.export_csv();
                        }

                        ui.separator();

                        ui.toggle_value(&mut self.show_conflicts_only, format!("{}  Conflicts only", ph::MAGNIFYING_GLASS));

                        if !self.groups.is_empty() {
                            ui.add(
                                TextEdit::singleline(&mut self.search_query)
                                    .hint_text(format!("{}  Filter entries...", ph::MAGNIFYING_GLASS))
                                    .desired_width(180.0)
                                    .margin(Margin::symmetric(6, 2)),
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
                                let txt = format!("{} {} file(s)", ph::FILE, self.files.len());
                                Frame::new()
                                    .fill(Color32::from_rgb(40, 40, 55))
                                    .corner_radius(CornerRadius::same(12))
                                    .inner_margin(Margin::symmetric(8, 3))
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
            Panel::top("file_chips").show(ui, |ui| {
                Frame::new()
                    .fill(Color32::from_rgb(16, 16, 26))
                    .inner_margin(Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(&format!("{}  Sources:", ph::FOLDER)).size(12.0).color(GRAY));
                            let mut remove_idx = None;
                            for (i, file) in self.files.iter().enumerate() {
                                let chip_color = Color32::from_rgb(35, 35, 50);
                                let resp = Frame::new()
                                    .fill(chip_color)
                                    .corner_radius(CornerRadius::same(14))
                                    .inner_margin(Margin::symmetric(8, 3))
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
                                            let close = ui.add(
                                                Label::new(
                                                    RichText::new("x").size(12.0).color(GRAY),
                                                )
                                                .sense(Sense::click()),
                                            );
                                            if close.clicked() {
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
            CentralPanel::default().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.25);
                    if let Some(logo) = &self.logo {
                        ui.image((logo.id(), vec2(128.0, 128.0)));
                    }
                    ui.add_space(16.0);
                    ui.label(RichText::new("ClashPass").size(36.0).color(ACCENT).strong());
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Password Conflict Resolver")
                            .size(18.0)
                            .color(GRAY),
                    );
                    ui.add_space(24.0);
                    let btn =
                        Button::new(&format!("{}  Import a password export CSV", ph::FOLDER_OPEN))
                            .min_size(Vec2::new(220.0, 40.0))
                            .fill(Color32::from_rgb(50, 35, 40));
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
        Panel::left("entry_list")
            .resizable(true)
            .default_size(240.0)
            .min_size(200.0)
            .max_size(350.0)
            .show(ui, |ui| {
                ui.add_space(2.0);
                ui.label(RichText::new("Entries").strong().size(13.0).color(ACCENT));
                ui.separator();

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

                ui.vertical(|ui| {
                    if self.groups.is_empty() {
                        ui.label(RichText::new("No entries found").size(12.0).color(GRAY));
                    } else if filtered.is_empty() {
                        ui.add_space(20.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No conflicts found").size(12.0).color(GRAY));
                        });
                    } else {
                        ScrollArea::vertical()
                            .id_salt("entry_scroll")
                            .auto_shrink([false, false])
                            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
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

                                    let card = Frame::new()
                                        .fill(card_color)
                                        .corner_radius(CornerRadius::same(6))
                                        .stroke(if is_selected {
                                            Stroke::new(1.5f32, ACCENT)
                                        } else {
                                            Stroke::new(1.0f32, Color32::from_rgb(40, 40, 55))
                                        })
                                        .inner_margin(Margin::symmetric(8, 4));

                                    let resp = card
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new(&group.title)
                                                            .size(12.0)
                                                            .strong(),
                                                    );
                                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                        Frame::new()
                                                            .fill(badge_bg)
                                                            .corner_radius(CornerRadius::same(4))
                                                            .inner_margin(Margin::symmetric(4, 1))
                                                            .show(ui, |ui| {
                                                                ui.label(
                                                                    RichText::new(badge_txt)
                                                                        .size(9.0)
                                                                        .color(Color32::WHITE),
                                                                );
                                                            });
                                                    });
                                                });
                                                if !group.username.is_empty() {
                                                    ui.label(
                                                        RichText::new(&group.username)
                                                            .size(10.0)
                                                            .color(GRAY),
                                                    );
                                                }
                                                if conflicts {
                                                    let fields = group.get_conflicting_fields();
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{} {} differ{}",
                                                            ph::WARNING,
                                                            fields.len(),
                                                            if fields.len() == 1 { "s" } else { "" }
                                                        ))
                                                        .size(10.0)
                                                        .color(YELLOW),
                                                    );
                                                } else if resolved {
                                                    ui.label(
                                                        RichText::new(&format!("{} Resolved", ph::CHECK))
                                                            .size(10.0)
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
                                }
                            });
                    }
                });
            });

        // ═══════════════════ COMPARISON TABLE ═══════════════════
        CentralPanel::default().show(ui, |ui| {
            if self.groups.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.label(RichText::new(&format!("{} No entries found", ph::TRAY)).size(20.0).color(GRAY));
                });
                return;
            }

            let sel = self.selected_group.min(self.groups.len().saturating_sub(1));

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
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if let Some(logo) = &self.logo {
                    ui.image((logo.id(), vec2(24.0, 24.0)));
                }
                ui.label(RichText::new(&group_title).size(18.0).strong());
                if !group_username.is_empty() {
                    ui.label(
                        RichText::new(format!("({})", group_username))
                            .size(14.0)
                            .color(GRAY),
                    );
                }
                if group_conflicts {
                    Frame::new()
                        .fill(Color32::from_rgb(80, 30, 30))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(6, 2))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(&format!("{} Conflicts", ph::WARNING))
                                    .size(11.0)
                                    .strong(),
                            );
                        });
                } else if group_num_files > 1 {
                    Frame::new()
                        .fill(Color32::from_rgb(30, 60, 40))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(6, 2))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(&format!("{} Synced", ph::CHECK))
                                    .size(11.0)
                                    .strong(),
                            );
                        });
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if group_num_files > 1 {
                        let eye_icon = if self.show_passwords { ph::EYE_SLASH } else { ph::EYE };
                        let toggle_btn = Button::new(format!("{} Passwords", eye_icon))
                            .fill(Color32::from_rgb(40, 40, 55));
                        if ui.add(toggle_btn).clicked() {
                            self.show_passwords = !self.show_passwords;
                        }
                    }
                });
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // ─── Column Headers ───
            let header_bg = Color32::from_rgb(24, 24, 36);
            let cell_h = 28.0;
            let col_border = Color32::from_rgb(45, 45, 60);

            let total_width = ui.available_width();
            let field_col_width = total_width * 0.12;
            let values_area_width = total_width * 0.88;

            let fields = ["Username", "Password", "URL", "Notes", "Created", "Updated"];
            let field_icons = [ph::USER, ph::KEY, ph::GLOBE, ph::NOTE_PENCIL, ph::CALENDAR, ph::ARROWS_CLOCKWISE];
            let getters: Vec<fn(&crate::models::PasswordEntry) -> &str> = vec![
                |e| &e.username,
                |e| &e.password,
                |e| &e.url,
                |e| &e.notes,
                |e| &e.created_at,
                |e| &e.updated_at,
            ];

            // Split entries into pairs
            let pairs: Vec<&[(usize, crate::models::PasswordEntry)]> = group_entries.chunks(2).collect();

            ScrollArea::vertical()
                .id_salt("comp_table_v3")
                .show(ui, |ui| {
                    for (pair_idx, pair) in pairs.iter().enumerate() {
                        let num_in_pair = pair.len();
                        let col_width = (values_area_width - (num_in_pair as f32 - 1.0)) / num_in_pair as f32;

                        // Pair box
                        Frame::new()
                            .fill(CARD_BG)
                            .corner_radius(CornerRadius::same(8))
                            .stroke(Stroke::new(1.0, col_border))
                            .inner_margin(Margin::symmetric(8, 8))
                            .show(ui, |ui| {
                                // Pair header — file names
                                ui.horizontal(|ui| {
                                    for (vi, (file_idx, _)) in pair.iter().enumerate() {
                                        let is_resolved_col = group_has_resolved && *file_idx == group_resolved_file;
                                        Frame::new()
                                            .fill(header_bg)
                                            .corner_radius(CornerRadius::same(4))
                                            .inner_margin(Margin::symmetric(6, 4))
                                            .show(ui, |ui| {
                                                ui.set_min_width(col_width);
                                                ui.horizontal(|ui| {
                                                    if is_resolved_col {
                                                        ui.label(RichText::new(ph::CHECK).size(11.0).color(GREEN));
                                                    }
                                                    ui.label(RichText::new(&file_labels[*file_idx]).size(11.0).strong().color(Color32::from_rgb(180, 180, 200)));
                                                });
                                            });
                                        if vi == 0 {
                                            ui.add_space(4.0);
                                        }
                                    }
                                });
                                ui.add_space(4.0);

                                // Field rows for this pair
                                for (fi, field) in fields.iter().enumerate() {
                                    let has_conflict = group_conflict_fields.contains(&field.to_string());
                                    let row_bg = if fi % 2 == 0 {
                                        Color32::from_rgb(22, 22, 34)
                                    } else {
                                        Color32::from_rgb(26, 26, 38)
                                    };
                                    let is_password_row = *field == "Password";

                                    Frame::new()
                                        .fill(row_bg)
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::symmetric(0, 3))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Field label
                                                Frame::new()
                                                    .fill(Color32::from_rgb(18, 18, 28))
                                                    .corner_radius(CornerRadius::same(4))
                                                    .inner_margin(Margin::symmetric(8, 2))
                                                    .show(ui, |ui| {
                                                        ui.set_min_width(field_col_width);
                                                        ui.label(
                                                            RichText::new(format!("{} {}", field_icons[fi], field))
                                                                .size(12.0)
                                                                .strong()
                                                                .color(Color32::from_rgb(170, 170, 190)),
                                                        );
                                                    });

                                                // Value cells
                                                for (vi, (file_idx, entry)) in pair.iter().enumerate() {
                                                    let val = getters[fi](entry);
                                                    let is_selected = group_has_resolved && *file_idx == group_resolved_file;
                                                    let is_resolved_cell = is_selected && has_conflict;

                                                    let cell_bg = if is_resolved_cell {
                                                        SELECTED_BG
                                                    } else if has_conflict {
                                                        CONFLICT_BG
                                                    } else {
                                                        Color32::TRANSPARENT
                                                    };

                                                    let col_stroke = if vi > 0 {
                                                        Stroke::new(1.0f32, col_border)
                                                    } else {
                                                        Stroke::new(0.0f32, Color32::TRANSPARENT)
                                                    };

                                                    let cell_resp = Frame::new()
                                                        .fill(cell_bg)
                                                        .stroke(col_stroke)
                                                        .inner_margin(Margin::symmetric(8, 2))
                                                        .show(ui, |ui| {
                                                            ui.set_min_width(col_width);

                                                            // Global index for this entry in group_entries
                                                            let global_idx = pair_idx * 2 + vi;
                                                            let is_editing = self.editing_field.as_deref() == Some(*field)
                                                                && self.editing_entry == Some(global_idx);

                                                            if is_editing {
                                                                let resp = ui.add_sized(
                                                                    Vec2::new(col_width - 20.0, 22.0),
                                                                    TextEdit::singleline(&mut self.edit_buffer)
                                                                        .font(TextStyle::Monospace)
                                                                        .margin(Margin::symmetric(2, 0)),
                                                                );
                                                                let enter = resp.lost_focus()
                                                                    && ui.input(|i| i.key_pressed(Key::Enter));
                                                                if enter {
                                                                    self.commit_edit();
                                                                    ui.memory_mut(|mem| mem.surrender_focus(resp.id));
                                                                }
                                                            } else {
                                                                let display_val = if is_password_row && !self.show_passwords && !val.is_empty() {
                                                                    "••••••••"
                                                                } else if val.is_empty() {
                                                                    "—"
                                                                } else {
                                                                    val
                                                                };

                                                                let text_color = if val.is_empty() {
                                                                    GRAY
                                                                } else if is_resolved_cell {
                                                                    GREEN
                                                                } else {
                                                                    Color32::from_rgb(210, 210, 220)
                                                                };

                                                                ui.horizontal(|ui| {
                                                                    if has_conflict && group_num_files > 1 {
                                                                        let indicator = if is_resolved_cell { "●" } else { "○" };
                                                                        let ind_color = if is_resolved_cell { GREEN } else { GRAY };
                                                                        let ind = ui.add(
                                                                            Label::new(RichText::new(indicator).size(14.0).color(ind_color))
                                                                                .sense(Sense::click()),
                                                                        );
                                                                        if ind.clicked() {
                                                                            self.resolve_group(sel, *file_idx);
                                                                        }
                                                                        ind.on_hover_text(if is_resolved_cell { "Selected" } else { "Click to select" });
                                                                        ui.add_space(4.0);
                                                                    }

                                                                    if has_conflict {
                                                                        let stripe_rect = Rect::from_min_size(
                                                                            ui.cursor().min - Vec2::new(8.0, 0.0),
                                                                            Vec2::new(3.0, cell_h - 4.0),
                                                                        );
                                                                        let stripe_color = if is_resolved_cell { GREEN } else { ACCENT };
                                                                        ui.painter().rect_filled(stripe_rect, 1.0, stripe_color);
                                                                    }

                                                                    let resp = ui.add(
                                                                        Label::new(
                                                                            RichText::new(display_val)
                                                                                .size(12.0)
                                                                                .color(text_color),
                                                                        )
                                                                        .sense(Sense::click()),
                                                                    );

                                                                    if has_conflict && resp.clicked() {
                                                                        self.resolve_group(sel, *file_idx);
                                                                    }
                                                                    if has_conflict {
                                                                        resp.on_hover_text("Click to select this value");
                                                                    }

                                                                    let editable = matches!(*field, "Username" | "Password" | "URL" | "Notes");
                                                                    if editable && !val.is_empty() {
                                                                        let edit_resp = ui.add(
                                                                            Label::new(RichText::new(ph::PENCIL_SIMPLE).size(13.0).color(GRAY))
                                                                                .sense(Sense::click()),
                                                                        )
                                                                        .on_hover_text("Edit value");
                                                                        if edit_resp.clicked() {
                                                                            self.begin_edit(field, global_idx, val);
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        }).response;

                                                    let hover_rect = cell_resp.rect;
                                                    if ui.rect_contains_pointer(hover_rect) && has_conflict {
                                                        ui.painter().rect_stroke(
                                                            hover_rect,
                                                            CornerRadius::same(4),
                                                            Stroke::new(1.0f32, Color32::from_rgb(100, 100, 130)),
                                                            StrokeKind::Inside,
                                                        );
                                                    }
                                                }
                                            });
                                        });
                                }

                                // Keep buttons for this pair
                                if group_conflicts && !group_has_resolved {
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        for (_file_idx, _entry) in pair.iter() {
                                            let btn = Button::new(format!("{} Keep", ph::CHECK))
                                                .fill(Color32::from_rgb(30, 60, 45));
                                            if ui.add(btn).clicked() {
                                                self.resolve_group(sel, pair[0].0);
                                            }
                                        }
                                    });
                                }
                            });

                        ui.add_space(8.0);
                    }

                    // ─── Extra fields ───
                    if let Some(first_entry) = group_entries.first().map(|(_, e)| e) {
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
                            ui.add_space(8.0);
                            CollapsingHeader::new(format!("{} Extra fields ({})", ph::PAPERCLIP, extras.len()))
                                .default_open(false)
                                .show(ui, |ui| {
                                    for col in &extras {
                                        Frame::new()
                                            .fill(Color32::from_rgb(22, 22, 34))
                                            .corner_radius(CornerRadius::same(3))
                                            .inner_margin(Margin::symmetric(6, 3))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.allocate_ui(Vec2::new(140.0, 20.0), |ui| {
                                                        ui.label(
                                                            RichText::new(col.as_str())
                                                                .italics()
                                                                .size(11.0)
                                                                .color(GRAY),
                                                        );
                                                    });
                                                    for (_fi, e) in &group_entries {
                                                        let v = e.raw.get(col.as_str()).cloned().unwrap_or_default();
                                                        let display = if v.is_empty() { "—" } else { &v };
                                                        ui.label(RichText::new(display).size(11.0));
                                                    }
                                                });
                                            });
                                    }
                                });
                        }
                    }
                });

            // ─── Resolved footer ───
            if group_has_resolved && group_num_files > 1 {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Resolved from").size(11.0).color(GRAY));
                    if let Some(pos) = group_entries.iter().position(|(i, _)| *i == group_resolved_file) {
                        ui.label(
                            RichText::new(&file_labels[pos])
                                .size(11.0)
                                .strong()
                                .color(GREEN),
                        );
                    }
                    if ui.button(RichText::new("Clear").size(11.0).color(GRAY)).clicked() {
                        self.groups[sel].resolved_source = None;
                    }
                });
            }
        });
    }
}
