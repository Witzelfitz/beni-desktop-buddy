//! Local, opt-in session timer. The caller supplies monotonic time.
use eframe::egui;
use std::time::{Duration, Instant};

const DEFAULT_MINUTES: [u16; 2] = [25, 5];
const MAX_MINUTES: u16 = 180;
const NOTICE_TTL: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Focus,
    Break,
}
impl Phase {
    fn index(self) -> usize {
        match self {
            Self::Focus => 0,
            Self::Break => 1,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Focus => "Fokus",
            Self::Break => "Pause",
        }
    }
    pub fn message(self) -> &'static str {
        match self {
            Self::Focus => "Fokuszeit geschafft! Zeit für eine Pause.",
            Self::Break => "Pause vorbei. Bereit für die nächste Fokuszeit?",
        }
    }
    pub fn animation(self) -> &'static str {
        match self {
            Self::Focus => "daumen",
            Self::Break => "idee",
        }
    }
}

struct Session {
    phase: Phase,
    remaining: Duration,
    deadline: Option<Instant>,
}

pub struct Timer {
    minutes: [u16; 2],
    session: Option<Session>,
    finished: Option<Phase>,
    pending: Option<(Phase, Instant)>,
}
impl Default for Timer {
    fn default() -> Self {
        Self {
            minutes: DEFAULT_MINUTES,
            session: None,
            finished: None,
            pending: None,
        }
    }
}
impl Timer {
    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        let mut timer = Self::default();
        if let Some(minutes) = storage.and_then(|s| eframe::get_value(s, "timer-minutes")) {
            timer.minutes = Self::sanitize(minutes);
        }
        timer
    }
    fn sanitize(minutes: [u16; 2]) -> [u16; 2] {
        std::array::from_fn(|i| {
            if (1..=MAX_MINUTES).contains(&minutes[i]) {
                minutes[i]
            } else {
                DEFAULT_MINUTES[i]
            }
        })
    }
    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "timer-minutes", &Self::sanitize(self.minutes));
    }
    pub fn start(&mut self, phase: Phase, now: Instant) {
        self.minutes = Self::sanitize(self.minutes);
        let remaining = Duration::from_secs(u64::from(self.minutes[phase.index()]) * 60);
        self.session = Some(Session {
            phase,
            remaining,
            deadline: Some(now + remaining),
        });
        self.finished = None;
        self.pending = None;
    }
    fn toggle_pause(&mut self, now: Instant) {
        if let Some(session) = &mut self.session {
            if let Some(deadline) = session.deadline.take() {
                session.remaining = deadline.saturating_duration_since(now);
            } else {
                session.deadline = Some(now + session.remaining);
            }
        }
    }
    pub fn stop(&mut self) {
        self.session = None;
        self.finished = None;
        self.pending = None;
    }
    /// Returns a completion exactly once, including its actual deadline for history.
    pub fn tick(&mut self, now: Instant) -> Option<(Phase, Instant)> {
        let session = self.session.as_ref()?;
        let deadline = session.deadline?;
        if now < deadline {
            return None;
        }
        let phase = session.phase;
        self.session = None;
        self.finished = Some(phase);
        self.pending = Some((phase, deadline));
        Some((phase, deadline))
    }
    /// Called only when the UI is ready for a notice. Old notices expire, history stays.
    pub fn take_notice(&mut self, now: Instant) -> Option<Phase> {
        let (phase, at) = self.pending.take()?;
        (now.saturating_duration_since(at) < NOTICE_TTL).then_some(phase)
    }
    pub fn ui(&mut self, ui: &mut egui::Ui, now: Instant) {
        self.show(ui, now, false);
    }
    pub fn panel(&mut self, ui: &mut egui::Ui, now: Instant) {
        self.show(ui, now, true);
    }
    pub fn status(&self, now: Instant) -> String {
        if let Some(session) = &self.session {
            let remaining = session
                .deadline
                .map_or(session.remaining, |d| d.saturating_duration_since(now));
            let seconds = remaining.as_secs() + u64::from(remaining.subsec_nanos() != 0);
            let paused = if session.deadline.is_none() {
                " · Unterbrochen"
            } else {
                ""
            };
            format!(
                "{} · {:02}:{:02}{paused}",
                session.phase.label(),
                seconds / 60,
                seconds % 60
            )
        } else if let Some(phase) = self.finished {
            format!("{} beendet", phase.label())
        } else {
            "Kein Timer läuft".into()
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, now: Instant, expanded: bool) {
        let status = self.status(now);
        if expanded {
            ui.heading("Fokus & Pause");
            self.contents(ui, now, &status);
        } else {
            egui::CollapsingHeader::new("Fokus & Pause")
                .id_salt("focus-timer")
                .show(ui, |ui| self.contents(ui, now, &status));
            if self.session.is_some() || self.finished.is_some() {
                ui.small(status);
            }
        }
    }
    fn contents(&mut self, ui: &mut egui::Ui, now: Instant, status: &str) {
        ui.label(status);
        if self.session.is_none() {
            for phase in [Phase::Focus, Phase::Break] {
                ui.horizontal(|ui| {
                    let label = ui.label(format!("{} (Minuten)", phase.label()));
                    ui.add(
                        egui::DragValue::new(&mut self.minutes[phase.index()])
                            .range(1..=MAX_MINUTES),
                    )
                    .labelled_by(label.id);
                });
            }
            ui.horizontal(|ui| {
                if ui.button("Fokus starten").clicked() {
                    self.start(Phase::Focus, now);
                }
                if ui.button("Pause starten").clicked() {
                    self.start(Phase::Break, now);
                }
            });
            if self.finished.is_some() && ui.button("Timer zurücksetzen").clicked() {
                self.stop();
            }
        } else {
            ui.horizontal(|ui| {
                let paused = self.session.as_ref().unwrap().deadline.is_none();
                if ui
                    .button(if paused {
                        "Timer fortsetzen"
                    } else {
                        "Timer unterbrechen"
                    })
                    .clicked()
                {
                    self.toggle_pause(now);
                }
                if ui.button("Timer abbrechen").clicked() {
                    self.stop();
                }
            });
        }
        ui.small("Läuft lokal. Kein automatischer Wechsel; App-Beenden stoppt den Timer.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaches_each_deadline_once_and_requires_explicit_next_start() {
        let now = Instant::now();
        let mut timer = Timer::default();
        assert!(!timer.has_session());
        assert!(timer.tick(now).is_none());
        for (phase, seconds) in [(Phase::Focus, 1500), (Phase::Break, 300)] {
            timer.start(phase, now);
            assert!(timer.has_session());
            let deadline = now + Duration::from_secs(seconds);
            assert!(timer.tick(deadline - Duration::from_nanos(1)).is_none());
            assert_eq!(timer.tick(deadline), Some((phase, deadline)));
            assert!(timer.tick(deadline + Duration::from_secs(1)).is_none());
            assert!(timer.session.is_none());
            assert!(!timer.has_session());
            assert_eq!(timer.take_notice(deadline), Some(phase));
            assert!(timer.take_notice(deadline).is_none());
        }
    }
    #[test]
    fn pause_preserves_remaining_time_and_cancel_or_restart_clears_old_notices() {
        let now = Instant::now();
        let mut timer = Timer::default();
        timer.start(Phase::Break, now);
        timer.toggle_pause(now + Duration::from_secs(100));
        assert!(
            timer.has_session(),
            "paused timers also need a restart warning"
        );
        assert!(timer.tick(now + Duration::from_secs(1000)).is_none());
        timer.toggle_pause(now + Duration::from_secs(1000));
        assert!(timer.tick(now + Duration::from_secs(1199)).is_none());
        assert!(timer.tick(now + Duration::from_secs(1200)).is_some());
        timer.start(Phase::Focus, now + Duration::from_secs(1200));
        assert!(timer.take_notice(now + Duration::from_secs(1200)).is_none());
        timer.stop();
        assert!(timer.tick(now + Duration::from_secs(9999)).is_none());
        assert!(timer.session.is_none() && timer.finished.is_none());
    }
    #[test]
    fn late_poll_records_original_deadline_and_drops_stale_bubble() {
        let now = Instant::now();
        let mut timer = Timer::default();
        timer.start(Phase::Break, now);
        let late = now + Duration::from_secs(420);
        assert_eq!(
            timer.tick(late),
            Some((Phase::Break, now + Duration::from_secs(300)))
        );
        assert!(timer.take_notice(late).is_none());
        assert_eq!(timer.finished, Some(Phase::Break));
    }
    #[test]
    fn invalid_minutes_fall_back_per_field_and_only_durations_are_saved() {
        struct Store(std::collections::HashMap<String, String>);
        impl eframe::Storage for Store {
            fn get_string(&self, key: &str) -> Option<String> {
                self.0.get(key).cloned()
            }
            fn set_string(&mut self, key: &str, value: String) {
                self.0.insert(key.into(), value);
            }
            fn remove_string(&mut self, key: &str) {
                self.0.remove(key);
            }
            fn flush(&mut self) {}
        }
        let mut store = Store(Default::default());
        eframe::set_value(&mut store, "timer-minutes", &[0u16, 7]);
        let mut timer = Timer::load(Some(&store));
        assert_eq!(timer.minutes, [25, 7]);
        timer.start(Phase::Focus, Instant::now());
        timer.save(&mut store);
        let restored = Timer::load(Some(&store));
        assert_eq!(restored.minutes, [25, 7]);
        assert!(restored.session.is_none() && restored.pending.is_none());
        assert_eq!(store.0.len(), 1);
        eframe::set_value(&mut store, "timer-minutes", &[180u16, 181]);
        assert_eq!(Timer::load(Some(&store)).minutes, [180, 5]);
        store.0.insert("timer-minutes".into(), "broken".into());
        assert_eq!(Timer::load(Some(&store)).minutes, DEFAULT_MINUTES);
    }
}
