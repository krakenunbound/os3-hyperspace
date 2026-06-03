//! Command Palette — "Spotlight for OS/3 Hyperspace".
//!
//! A single, keyboard-driven, fuzzy-searchable overlay (⌘K / Ctrl+K) that can do
//! *anything* in the shell: spawn objects, switch/create dimensions, fit the view,
//! save, toggle panels, talk to the local agent.
//!
//! Design goals (see docs/ux-vision.md):
//! - macOS-grade feel: centered glass card, dim backdrop, instant focus, fuzzy match.
//! - Linux-grade power: keyboard-complete. Every command is one entry in a registry,
//!   so new features become discoverable + reachable for free.
//!
//! Architecture: the palette does NOT hold callbacks (that fights Rust's borrow checker
//! against `&mut HyperspaceApp`). Instead each `Command` carries a plain-data
//! [`CommandAction`]; `show()` returns the chosen action and the app interprets it.

use eframe::egui;
use hyperspace_core::{DimensionId, ObjectKind};

/// A plain-data description of "what to do" — interpreted by the app (see
/// `HyperspaceApp::run_command_action`). Cloneable so the palette can hand one back.
#[derive(Clone)]
pub enum CommandAction {
    Spawn(ObjectKind),
    CreateDimension,
    SwitchDimension(DimensionId),
    FitToContent,
    Save,
    ToggleHud,
    AskAiDimension,
    PingAgent,
    DeleteSelected,
    About,
}

/// One searchable, executable entry in the palette.
#[derive(Clone)]
pub struct Command {
    pub title: String,
    /// Short right-aligned category badge (e.g. "Object", "Workspace", "View").
    pub badge: String,
    /// Leading glyph for quick visual scanning.
    pub icon: String,
    pub action: CommandAction,
}

impl Command {
    pub fn new(
        icon: impl Into<String>,
        title: impl Into<String>,
        badge: impl Into<String>,
        action: CommandAction,
    ) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
            badge: badge.into(),
            action,
        }
    }
}

#[derive(Default)]
pub struct CommandPalette {
    pub open: bool,
    query: String,
    selected: usize,
    just_opened: bool,
}

impl CommandPalette {
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open = true;
            self.query.clear();
            self.selected = 0;
            self.just_opened = true;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.just_opened = false;
    }

    /// Render the palette. Returns `Some(action)` if the user executed a command this frame.
    pub fn show(&mut self, ctx: &egui::Context, commands: &[Command]) -> Option<CommandAction> {
        if !self.open {
            return None;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.close();
            return None;
        }

        // --- Filter + rank ---
        let mut scored: Vec<(i32, usize)> = commands
            .iter()
            .enumerate()
            .filter_map(|(idx, c)| fuzzy_score(&self.query, &c.title).map(|s| (s, idx)))
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        let results: Vec<&Command> = scored.iter().map(|&(_, idx)| &commands[idx]).collect();

        // Keep selection in range.
        if results.is_empty() {
            self.selected = 0;
        } else if self.selected >= results.len() {
            self.selected = results.len() - 1;
        }

        // --- Keyboard navigation (works while the search field has focus) ---
        ctx.input(|i| {
            if !results.is_empty() {
                if i.key_pressed(egui::Key::ArrowDown) {
                    self.selected = (self.selected + 1) % results.len();
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    self.selected = (self.selected + results.len() - 1) % results.len();
                }
            }
        });
        let mut chosen: Option<CommandAction> = None;
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(c) = results.get(self.selected) {
                chosen = Some(c.action.clone());
            }
        }

        // --- Dim backdrop (click outside to dismiss) ---
        let screen = ctx.screen_rect();
        egui::Area::new(egui::Id::new("palette_backdrop"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let resp = ui.allocate_rect(screen, egui::Sense::click());
                ui.painter()
                    .rect_filled(screen, 0.0, egui::Color32::from_rgba_unmultiplied(2, 3, 8, 180));
                if resp.clicked() {
                    self.close();
                }
            });

        // --- The palette card ---
        let frame = egui::Frame::default()
            .fill(egui::Color32::from_rgb(13, 15, 28))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(128, 96, 210)))
            .corner_radius(egui::CornerRadius::same(14))
            .inner_margin(egui::Margin::same(12))
            .shadow(egui::epaint::Shadow {
                offset: [0, 10],
                blur: 28,
                spread: 2,
                color: egui::Color32::from_black_alpha(180),
            });

        egui::Window::new("command_palette")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 110.0))
            .fixed_size(egui::vec2(560.0, 0.0))
            .frame(frame)
            .show(ctx, |ui| {
                // Search field
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⌘").size(18.0).color(egui::Color32::from_rgb(180, 150, 255)));
                    let edit = egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search commands… (↑↓ to move, ↵ to run, esc to close)")
                        .desired_width(f32::INFINITY)
                        .frame(false)
                        .font(egui::FontId::proportional(17.0));
                    let resp = ui.add(edit);
                    if self.just_opened {
                        resp.request_focus();
                        self.just_opened = false;
                    }
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(2.0);

                // Results
                if results.is_empty() {
                    ui.add_space(14.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("No matching commands")
                                .color(egui::Color32::from_rgb(120, 130, 160)),
                        );
                    });
                    ui.add_space(14.0);
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (i, cmd) in results.iter().enumerate() {
                                let selected = i == self.selected;
                                let resp = row(ui, cmd, selected);
                                if selected {
                                    resp.scroll_to_me(Some(egui::Align::Center));
                                }
                                if resp.clicked() {
                                    chosen = Some(cmd.action.clone());
                                }
                            }
                        });
                }
            });

        if chosen.is_some() {
            self.close();
        }
        chosen
    }
}

/// Render a single result row with selection/hover highlight. Returns its click response.
fn row(ui: &mut egui::Ui, cmd: &Command, selected: bool) -> egui::Response {
    let height = 34.0;
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::click());
    let painter = ui.painter_at(rect);

    if selected {
        painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(44, 40, 86));
        painter.rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 120, 230)),
            egui::StrokeKind::Inside,
        );
    } else if resp.hovered() {
        painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(28, 30, 54));
    }

    // Icon
    painter.text(
        rect.left_center() + egui::vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        &cmd.icon,
        egui::FontId::proportional(16.0),
        egui::Color32::from_rgb(230, 225, 255),
    );
    // Title
    painter.text(
        rect.left_center() + egui::vec2(40.0, 0.0),
        egui::Align2::LEFT_CENTER,
        &cmd.title,
        egui::FontId::proportional(14.5),
        if selected {
            egui::Color32::from_rgb(245, 245, 255)
        } else {
            egui::Color32::from_rgb(205, 210, 230)
        },
    );
    // Badge (right-aligned)
    if !cmd.badge.is_empty() {
        painter.text(
            rect.right_center() + egui::vec2(-12.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            &cmd.badge,
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgb(130, 140, 175),
        );
    }

    resp
}

/// Lightweight fuzzy subsequence matcher. Returns `None` if `query` is not a subsequence
/// of `text` (case-insensitive), otherwise a score where higher is better. Empty query
/// matches everything with a neutral score (preserves the registry's natural order).
fn fuzzy_score(query: &str, text: &str) -> Option<i32> {
    if query.trim().is_empty() {
        return Some(0);
    }
    let q: Vec<char> = query.to_lowercase().chars().filter(|c| !c.is_whitespace()).collect();
    let t: Vec<char> = text.to_lowercase().chars().collect();

    let mut qi = 0usize;
    let mut score = 0i32;
    let mut last_match: Option<usize> = None;

    for (ti, &tc) in t.iter().enumerate() {
        if qi < q.len() && tc == q[qi] {
            score += 1;
            if ti == 0 {
                score += 15; // matches very start
            } else if matches!(t[ti - 1], ' ' | '-' | '/' | '_') {
                score += 10; // start of a word
            }
            if let Some(lm) = last_match {
                if lm + 1 == ti {
                    score += 5; // consecutive run
                }
            }
            last_match = Some(ti);
            qi += 1;
        }
    }

    if qi == q.len() {
        // Slightly prefer shorter titles among equal matches.
        Some(score - (t.len() as i32) / 12)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_anything() {
        assert_eq!(fuzzy_score("", "Spawn Note"), Some(0));
        assert_eq!(fuzzy_score("   ", "Anything"), Some(0));
    }

    #[test]
    fn subsequence_matches_and_nonmatch_rejected() {
        assert!(fuzzy_score("sn", "Spawn Note").is_some());
        assert!(fuzzy_score("note", "Spawn Note").is_some());
        assert!(fuzzy_score("xyz", "Spawn Note").is_none());
    }

    #[test]
    fn word_start_outscores_buried_match() {
        // "fit" as a leading word should beat the same letters buried mid-word.
        let lead = fuzzy_score("fit", "Fit to content").unwrap();
        let buried = fuzzy_score("fit", "Backfit settings").unwrap();
        assert!(lead > buried, "lead {lead} should beat buried {buried}");
    }
}
