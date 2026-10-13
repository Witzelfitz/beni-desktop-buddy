//! Session-only, bounded receipt history. Display TTL does not erase receipts.
use buddy_ipc::{MAX_RECENT_MESSAGES, ReceivedNotification};
use eframe::egui::{self, Color32};
use std::{collections::VecDeque, time::Instant};

#[derive(Default)]
pub(crate) struct History {
    entries: VecDeque<ReceivedNotification>,
}

#[derive(Default)]
pub(crate) struct HistoryResponse {
    pub close: bool,
    pub clear: bool,
}

impl History {
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn extend(&mut self, received: impl IntoIterator<Item = ReceivedNotification>) {
        for entry in received {
            if self.entries.len() == MAX_RECENT_MESSAGES {
                self.entries.pop_front();
            }
            self.entries.push_back(entry);
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn ui(&self, ui: &mut egui::Ui, now: Instant) -> (egui::Rect, HistoryResponse) {
        let mut action = HistoryResponse::default();
        let response = egui::Frame::new()
            .fill(Color32::from_rgba_unmultiplied(19, 29, 39, 245))
            .corner_radius(14)
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width((ui.available_width() - 16.0).max(100.0));
                ui.horizontal(|ui| {
                    ui.strong(format!("Meldungsverlauf ({})", self.len()));
                    action.clear = ui
                        .add_enabled(
                            !self.entries.is_empty(),
                            egui::Button::new("Leeren").small(),
                        )
                        .clicked();
                    let close = ui.button("×");
                    close.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            close.enabled(),
                            "Verlauf schliessen",
                        )
                    });
                    action.close = close.clicked();
                });
                egui::ScrollArea::vertical()
                    .id_salt("status-history")
                    .max_height(120.0)
                    .min_scrolled_height(120.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.entries.is_empty() {
                            ui.add(
                                egui::Label::new("Hier erscheinen deine Statusmeldungen.").wrap(),
                            );
                        }
                        for (index, entry) in self.entries.iter().enumerate().rev() {
                            ui.push_id((entry.received_at, index), |ui| {
                                let seconds =
                                    now.saturating_duration_since(entry.received_at).as_secs();
                                let age = if seconds < 60 {
                                    format!("vor {seconds} Sek.")
                                } else {
                                    format!("vor {} Min.", seconds / 60)
                                };
                                let important = if entry.message.priority
                                    == buddy_core::notifications::Priority::High
                                {
                                    " · Wichtig"
                                } else {
                                    ""
                                };
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(format!(
                                            "{} · {age}{important}",
                                            entry.message.source
                                        ))
                                        .small(),
                                    )
                                    .wrap(),
                                );
                                ui.add(
                                    egui::Label::new(&entry.message.text)
                                        .wrap()
                                        .selectable(true),
                                );
                                ui.separator();
                            });
                        }
                    });
            });
        (response.response.rect, action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_the_latest_hundred_in_receipt_order_and_clears_them() {
        let mut history = History::default();
        let now = Instant::now();
        history.extend((0..125).map(|i| ReceivedNotification {
            message: buddy_core::notifications::Notification::new(format!("Meldung {i}")),
            received_at: now,
        }));
        assert_eq!(history.entries.len(), 100);
        assert_eq!(history.entries.front().unwrap().message.text, "Meldung 25");
        assert_eq!(history.entries.back().unwrap().message.text, "Meldung 124");
        history.clear();
        assert!(history.entries.is_empty());
    }
}
