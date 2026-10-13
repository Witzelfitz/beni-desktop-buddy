//! Bounded, session-only reminders. Time is supplied explicitly by the caller.
use eframe::egui;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const MAX_REMINDERS: usize = 10;
const MAX_TEXT_CHARS: usize = 240;
const MAX_MINUTES: u16 = 1440;
const NOTICE_TTL: Duration = Duration::from_secs(120);

#[derive(Clone, Debug)]
pub struct Reminder {
    id: u64,
    pub text: String,
    deadline: Instant,
}

pub struct Reminders {
    draft: String,
    minutes: u16,
    next_id: u64,
    scheduled: Vec<Reminder>,
    pending: VecDeque<Reminder>,
    error: Option<&'static str>,
}
impl Default for Reminders {
    fn default() -> Self {
        Self {
            draft: String::new(),
            minutes: 10,
            next_id: 1,
            scheduled: vec![],
            pending: VecDeque::new(),
            error: None,
        }
    }
}
impl Reminders {
    fn prune(&mut self, now: Instant) {
        self.pending
            .retain(|r| now.saturating_duration_since(r.deadline) < NOTICE_TTL);
    }
    pub fn schedule(
        &mut self,
        text: &str,
        minutes: u16,
        now: Instant,
    ) -> Result<u64, &'static str> {
        self.prune(now);
        let text = text
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>()
            .trim()
            .to_owned();
        if text.is_empty() || text.chars().count() > MAX_TEXT_CHARS {
            return Err("Bitte 1–240 Zeichen für die Erinnerung eingeben.");
        }
        if !(1..=MAX_MINUTES).contains(&minutes) {
            return Err("Die Wartezeit muss 1–1440 Minuten betragen.");
        }
        if self.scheduled.len() + self.pending.len() >= MAX_REMINDERS {
            return Err("Es sind höchstens zehn Erinnerungen gleichzeitig möglich.");
        }
        let next = self
            .next_id
            .checked_add(1)
            .ok_or("Keine weiteren Erinnerungen in dieser Sitzung möglich.")?;
        let id = self.next_id;
        self.next_id = next;
        self.scheduled.push(Reminder {
            id,
            text,
            deadline: now + Duration::from_secs(u64::from(minutes) * 60),
        });
        self.scheduled.sort_by_key(|r| (r.deadline, r.id));
        Ok(id)
    }
    fn cancel(&mut self, id: u64) {
        self.scheduled.retain(|r| r.id != id);
        self.pending.retain(|r| r.id != id);
    }
    /// Every due reminder is returned once for history, even if its bubble is already stale.
    pub fn tick(&mut self, now: Instant) -> Vec<Reminder> {
        self.prune(now);
        let mut received = vec![];
        while self.scheduled.first().is_some_and(|r| r.deadline <= now) {
            let reminder = self.scheduled.remove(0);
            if now.saturating_duration_since(reminder.deadline) < NOTICE_TTL {
                self.pending.push_back(reminder.clone());
            }
            received.push(reminder);
        }
        received
    }
    pub fn take_notice(&mut self, now: Instant) -> Option<Reminder> {
        self.prune(now);
        self.pending.pop_front()
    }
    pub fn ui(&mut self, ui: &mut egui::Ui, now: Instant) {
        let section = egui::CollapsingHeader::new("Erinnerungen")
            .id_salt("local-reminders")
            .show(ui, |ui| {
                let text = ui.add(
                    egui::TextEdit::singleline(&mut self.draft)
                        .hint_text("Was soll Beni dir sagen?")
                        .char_limit(MAX_TEXT_CHARS),
                );
                ui.ctx()
                    .accesskit_node_builder(text.id, |node| node.set_label("Erinnerungstext"));
                ui.horizontal(|ui| {
                    let label = ui.label("Erinnern in (Minuten)");
                    ui.add(egui::DragValue::new(&mut self.minutes).range(1..=MAX_MINUTES))
                        .labelled_by(label.id);
                });
                let capacity = self.scheduled.len() + self.pending.len() < MAX_REMINDERS;
                let nonempty = self
                    .draft
                    .chars()
                    .any(|c| !c.is_control() && !c.is_whitespace());
                if ui
                    .add_enabled(capacity && nonempty, egui::Button::new("Erinnerung planen"))
                    .clicked()
                {
                    let draft = self.draft.clone();
                    match self.schedule(&draft, self.minutes, now) {
                        Ok(_) => {
                            self.draft.clear();
                            self.error = None;
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
                if !capacity {
                    ui.small("Zehn Erinnerungen aktiv. Entferne eine, um Platz zu machen.");
                }
                if let Some(error) = self.error {
                    ui.colored_label(egui::Color32::LIGHT_RED, error);
                }
                let mut cancel = None;
                for (reminder, waiting) in self
                    .scheduled
                    .iter()
                    .map(|r| (r, false))
                    .chain(self.pending.iter().map(|r| (r, true)))
                {
                    ui.push_id(reminder.id, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width((ui.available_width() - 90.0).max(100.0));
                                ui.add(egui::Label::new(&reminder.text).wrap());
                                if waiting {
                                    ui.small("Fällig · wartet auf Anzeige");
                                } else {
                                    let duration = reminder.deadline.saturating_duration_since(now);
                                    let seconds = duration.as_secs()
                                        + u64::from(duration.subsec_nanos() != 0);
                                    ui.small(format!(
                                        "Noch {:02}:{:02}",
                                        seconds / 60,
                                        seconds % 60
                                    ));
                                }
                            });
                            let remove = ui.button("Entfernen");
                            remove.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    remove.enabled(),
                                    format!("Erinnerung entfernen: {}", reminder.text),
                                )
                            });
                            if remove.clicked() {
                                cancel = Some(reminder.id);
                            }
                        });
                    });
                }
                if let Some(id) = cancel {
                    self.cancel(id);
                }
                ui.small(
                    "Nur in dieser Sitzung. App-Beenden verwirft Texte und geplante Erinnerungen.",
                );
            });
        let count = self.scheduled.len() + self.pending.len();
        if section.body_response.is_none() && count > 0 {
            ui.small(format!("{count} Erinnerungen aktiv"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orders_by_deadline_then_creation_and_delivers_each_once() {
        let mut reminders = Reminders::default();
        let now = Instant::now();
        reminders.schedule("Später", 2, now).unwrap();
        reminders.schedule("Zuerst", 1, now).unwrap();
        reminders.schedule("Auch bald", 1, now).unwrap();
        assert!(reminders.tick(now + Duration::from_secs(59)).is_empty());
        let due = reminders.tick(now + Duration::from_secs(60));
        assert_eq!(
            due.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
            ["Zuerst", "Auch bald"]
        );
        assert!(reminders.tick(now + Duration::from_secs(60)).is_empty());
        assert_eq!(
            reminders
                .take_notice(now + Duration::from_secs(60))
                .unwrap()
                .text,
            "Zuerst"
        );
        assert_eq!(
            reminders
                .take_notice(now + Duration::from_secs(60))
                .unwrap()
                .text,
            "Auch bald"
        );
        assert!(
            reminders
                .take_notice(now + Duration::from_secs(60))
                .is_none()
        );
        assert_eq!(
            reminders.tick(now + Duration::from_secs(120))[0].text,
            "Später"
        );
    }
    #[test]
    fn rejects_empty_oversized_or_invalid_delay_and_cleans_control_characters() {
        let mut reminders = Reminders::default();
        let now = Instant::now();
        for text in ["", " \n\t ", &"ä".repeat(241)] {
            assert!(reminders.schedule(text, 1, now).is_err());
        }
        for minutes in [0, 1441, u16::MAX] {
            assert!(reminders.schedule("Gültiger Text", minutes, now).is_err());
        }
        assert!(reminders.scheduled.is_empty());
        reminders.schedule("  Hoi\u{0007} Beni!  ", 1, now).unwrap();
        assert_eq!(reminders.scheduled[0].text, "Hoi Beni!");
        reminders.schedule(&"ä".repeat(240), 1440, now).unwrap();
        assert_eq!(reminders.scheduled[1].text.chars().count(), 240);
    }
    #[test]
    fn bounds_both_scheduled_and_waiting_items_and_cancels_either() {
        let mut reminders = Reminders::default();
        let now = Instant::now();
        for _ in 0..MAX_REMINDERS {
            reminders.schedule("Gleich", 1, now).unwrap();
        }
        assert!(reminders.schedule("Zu viel", 1, now).is_err());
        reminders.cancel(1);
        reminders.schedule("Ersatz", 1, now).unwrap();
        assert_eq!(
            reminders.tick(now + Duration::from_secs(60)).len(),
            MAX_REMINDERS
        );
        assert!(
            reminders
                .schedule("Noch zu viel", 1, now + Duration::from_secs(60))
                .is_err()
        );
        reminders.cancel(2);
        reminders
            .schedule("Neu", 1, now + Duration::from_secs(60))
            .unwrap();
        assert_eq!(reminders.pending.len(), 9);
        reminders.cancel(12);
        assert!(reminders.scheduled.is_empty());
    }
    #[test]
    fn late_poll_keeps_receipt_but_expires_notice_and_releases_capacity() {
        let mut reminders = Reminders::default();
        let now = Instant::now();
        reminders.schedule("Verpasst", 1, now).unwrap();
        assert_eq!(reminders.tick(now + Duration::from_secs(180)).len(), 1);
        assert!(
            reminders
                .take_notice(now + Duration::from_secs(180))
                .is_none()
        );
        for _ in 0..MAX_REMINDERS {
            reminders
                .schedule("Neu", 1, now + Duration::from_secs(180))
                .unwrap();
        }
        reminders.tick(now + Duration::from_secs(240));
        assert!(
            reminders
                .schedule("Begrenzt", 1, now + Duration::from_secs(359))
                .is_err()
        );
        assert!(
            reminders
                .schedule("Wieder Platz", 1, now + Duration::from_secs(360))
                .is_ok()
        );
    }
}
