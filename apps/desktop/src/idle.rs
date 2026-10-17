//! Quiet scheduling of existing SDK animations; never creates or edits a pose.
use buddy_core::{BuddyError, Engine, Player};

pub const REST_MS: f64 = 30_000.0;
const GESTURES: [&str; 2] = ["nachdenken", "drehen"];

#[derive(Debug, Default)]
pub struct Idle {
    rest_ms: f64,
    next: usize,
    gesture: Option<&'static str>,
}

impl Idle {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn owns(&self, player: &Player) -> bool {
        self.gesture == Some(player.state())
    }

    /// Call after Player::advance. It remains responsible for original one-shot
    /// timing and poses. Only quiet waiting time belongs to this scheduler.
    pub fn advance(
        &mut self,
        engine: &Engine,
        player: &mut Player,
        delta_ms: f64,
        speed: f64,
        enabled: bool,
    ) -> Result<(), BuddyError> {
        let step = delta_ms * speed;
        if !delta_ms.is_finite()
            || delta_ms < 0.0
            || !speed.is_finite()
            || !(0.25..=3.0).contains(&speed)
            || !step.is_finite()
        {
            return Err(BuddyError("Ungültige Zeit für Idle-Gesten".into()));
        }
        if self.owns(player) {
            return Ok(());
        }
        if self.gesture.take().is_some() {
            self.rest_ms = 0.0;
            return Ok(());
        }
        if player.state() != "warten" {
            self.rest_ms = 0.0;
            return Ok(());
        }
        if !enabled || player.paused {
            return Ok(());
        }
        self.rest_ms += step;
        if self.rest_ms >= REST_MS {
            let state = GESTURES[self.next];
            player.play(engine, state, true)?;
            self.gesture = Some(state);
            self.rest_ms = 0.0;
            self.next = (self.next + 1) % GESTURES.len();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternates_original_one_shots_with_long_rest_and_exact_sdk_poses() {
        let engine = Engine::embedded().unwrap();
        let mut player = Player::default();
        let mut idle = Idle::default();
        idle.advance(&engine, &mut player, REST_MS - 1.0, 1.0, true)
            .unwrap();
        assert_eq!(player.state(), "warten");
        for state in ["nachdenken", "drehen", "nachdenken"] {
            idle.advance(&engine, &mut player, 1.0, 1.0, true).unwrap();
            assert_eq!(player.state(), state);
            assert!(idle.owns(&player));
            player.advance(&engine, 537.0, 1.0).unwrap();
            let pose = player
                .pose(&engine, &buddy_core::Settings::default())
                .unwrap();
            assert_eq!(
                pose,
                engine
                    .pose_at(state, 537.0, buddy_core::Settings::default().energy)
                    .unwrap()
            );
            idle.advance(&engine, &mut player, REST_MS * 10.0, 1.0, true)
                .unwrap();
            assert_eq!(
                player.state(),
                state,
                "never replace an in-progress gesture"
            );
            player
                .advance(&engine, engine.animations[state].duration_ms(), 1.0)
                .unwrap();
            idle.advance(&engine, &mut player, 0.0, 1.0, true).unwrap();
            assert_eq!(player.state(), "warten");
            assert!(!idle.owns(&player));
            idle.advance(&engine, &mut player, REST_MS - 1.0, 1.0, true)
                .unwrap();
            assert_eq!(player.state(), "warten");
        }
    }

    #[test]
    fn pause_reduction_and_other_actions_never_trigger_or_replace_gestures() {
        let engine = Engine::embedded().unwrap();
        let mut player = Player::default();
        let mut idle = Idle::default();
        player.paused = true;
        idle.advance(&engine, &mut player, REST_MS, 1.0, true)
            .unwrap();
        assert!(player.paused);
        player.paused = false;
        idle.advance(&engine, &mut player, REST_MS, 1.0, false)
            .unwrap();
        assert_eq!(player.state(), "warten");
        player.play(&engine, "daumen", true).unwrap();
        idle.advance(&engine, &mut player, REST_MS, 1.0, true)
            .unwrap();
        assert_eq!(player.state(), "daumen");
        player.advance(&engine, 10_000.0, 1.0).unwrap();
        idle.advance(&engine, &mut player, REST_MS / 2.0, 2.0, true)
            .unwrap();
        assert_eq!(player.state(), "nachdenken", "speed applies to quiet time");
        idle.reset();
        assert!(!idle.owns(&player));
        player.play(&engine, "warten", false).unwrap();
        idle.advance(&engine, &mut player, REST_MS - 1.0, 1.0, true)
            .unwrap();
        assert_eq!(player.state(), "warten");
    }

    #[test]
    fn invalid_time_is_rejected_and_large_steps_start_at_most_one_existing_gesture() {
        let engine = Engine::embedded().unwrap();
        let mut player = Player::default();
        let mut idle = Idle::default();
        for (delta, speed) in [
            (f64::NAN, 1.0),
            (-1.0, 1.0),
            (1.0, f64::INFINITY),
            (1.0, 0.0),
            (f64::MAX, 3.0),
        ] {
            assert!(
                idle.advance(&engine, &mut player, delta, speed, true)
                    .is_err()
            );
            assert_eq!(player.state(), "warten");
        }
        idle.advance(&engine, &mut player, 1e20, 1.0, true).unwrap();
        assert_eq!(player.state(), "nachdenken");
        assert_eq!(player.elapsed_ms(), 0.0);
    }
}
