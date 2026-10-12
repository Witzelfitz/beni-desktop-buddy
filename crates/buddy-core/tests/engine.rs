use buddy_core::{ATLAS_PNG, Engine, Player, Pose, Settings, png_dimensions};
use serde::Deserialize;

#[test]
fn embedded_assets_are_complete_and_consistent() {
    let engine = Engine::embedded().unwrap();
    assert_eq!(engine.atlas.frames.len(), 44);
    assert_eq!(engine.animations.len(), 20);
    for (name, animation) in &engine.animations {
        let pose = engine
            .pose_at(name, animation.duration_ms() - 0.1, 1.0)
            .unwrap();
        assert!(engine.atlas.frames.contains_key(&pose.frame));
        assert_eq!(
            engine.pose_at(name, animation.duration_ms(), 1.0).unwrap(),
            engine.pose_at(name, 0.0, 1.0).unwrap()
        );
    }
}
#[derive(Deserialize)]
struct Reference {
    state: String,
    time: f64,
    energy: f64,
    pose: Pose,
}
#[test]
fn rust_matches_original_sdk_reference_poses() {
    let engine = Engine::embedded().unwrap();
    let references: Vec<Reference> =
        serde_json::from_str(include_str!("fixtures/sdk-poses.json")).unwrap();
    assert_eq!(references.len(), 480);
    for r in references {
        let p = engine.pose_at(&r.state, r.time, r.energy).unwrap();
        assert_eq!(p.frame, r.pose.frame, "{} @ {}", r.state, r.time);
        assert_eq!(p.mirror, r.pose.mirror);
        assert_eq!(p.index, r.pose.index);
        for (actual, expected) in [p.tx, p.ty, p.rotation, p.sx, p.sy, p.progress]
            .into_iter()
            .zip([
                r.pose.tx,
                r.pose.ty,
                r.pose.rotation,
                r.pose.sx,
                r.pose.sy,
                r.pose.progress,
            ])
        {
            assert!(
                (actual - expected).abs() < 1e-10,
                "{} @ {}: {actual} != {expected}",
                r.state,
                r.time
            );
        }
    }
}
#[test]
fn aliases_and_unknown_states_are_handled() {
    let engine = Engine::embedded().unwrap();
    assert_eq!(engine.normalize("  WAVE ").unwrap(), "hallo");
    assert_eq!(engine.normalize("erklären").unwrap(), "erklaeren");
    assert!(engine.normalize("does-not-exist").is_err());
}
#[test]
fn one_shot_finishes_at_idle_and_can_be_interrupted() {
    let engine = Engine::embedded().unwrap();
    let mut player = Player::default();
    player.play(&engine, "hallo", true).unwrap();
    player.advance(&engine, 1599.0, 1.0).unwrap();
    assert_eq!(player.state(), "hallo");
    player.advance(&engine, 1.0, 1.0).unwrap();
    assert_eq!(player.state(), "warten");
    player.play(&engine, "jubeln", true).unwrap();
    player.play(&engine, "nachdenken", false).unwrap();
    player.advance(&engine, 5000.0, 1.0).unwrap();
    assert_eq!(player.state(), "nachdenken");
}
#[test]
fn pause_and_speed_control_time_without_jumps() {
    let engine = Engine::embedded().unwrap();
    let mut player = Player::default();
    player.advance(&engine, 100.0, 2.0).unwrap();
    assert_eq!(player.elapsed_ms(), 200.0);
    player.paused = true;
    player.advance(&engine, 5000.0, 1.0).unwrap();
    assert_eq!(player.elapsed_ms(), 200.0);
    player.paused = false;
    player.advance(&engine, 100.0, 0.5).unwrap();
    assert_eq!(player.elapsed_ms(), 250.0);
}
#[test]
fn invalid_input_does_not_change_player_state() {
    let engine = Engine::embedded().unwrap();
    let mut player = Player::default();
    for delta in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(player.advance(&engine, delta, 1.0).is_err());
    }
    for speed in [0.0, 4.0, f64::NAN] {
        assert!(player.advance(&engine, 1.0, speed).is_err());
    }
    assert!(player.play(&engine, "unknown", true).is_err());
    assert_eq!(player.state(), "warten");
    assert_eq!(player.elapsed_ms(), 0.0);
    assert!(engine.pose_at("warten", f64::NAN, 1.0).is_err());
    assert!(engine.pose_at("warten", 0.0, 1.1).is_err());
}
#[test]
fn reduced_motion_freezes_pose_but_finishes_one_shots() {
    let engine = Engine::embedded().unwrap();
    let mut player = Player::default();
    let settings = Settings {
        reduced_motion: true,
        ..Default::default()
    };
    player.play(&engine, "hallo", true).unwrap();
    let first = player.pose(&engine, &settings).unwrap();
    player.advance(&engine, 700.0, 1.0).unwrap();
    assert_eq!(player.pose(&engine, &settings).unwrap(), first);
    player.advance(&engine, 900.0, 1.0).unwrap();
    assert_eq!(player.state(), "warten");
}
#[test]
fn malformed_assets_and_out_of_bounds_frames_are_rejected() {
    assert!(png_dimensions(&[]).is_err());
    assert!(png_dimensions(&ATLAS_PNG[..23]).is_err());
    let mut png = ATLAS_PNG[..24].to_vec();
    png[16..20].fill(0);
    assert!(png_dimensions(&png).is_err());
    let engine = Engine::embedded().unwrap();
    let mut broken = engine.clone();
    broken.atlas.frames.get_mut("front").unwrap()[0] = i32::MAX;
    assert!(broken.validate().is_err());
    let mut broken = engine.clone();
    broken.animations.get_mut("warten").unwrap().segs[0].d = 0.0;
    assert!(broken.validate().is_err());
    let mut broken = engine;
    broken.animations.get_mut("warten").unwrap().motion[0].t = "unknown".into();
    assert!(broken.validate().is_err());
}
#[test]
fn mirroring_preserves_the_ground_anchor_and_rotation_geometry() {
    let engine = Engine::embedded().unwrap();
    let pose = engine.pose_at("warten", 0.0, 0.0).unwrap();
    let corners = engine.corners("warten", &pose, false).unwrap();
    let flipped = engine.corners("warten", &pose, true).unwrap();
    for (a, b) in corners.iter().zip(flipped) {
        assert!((a[0] + b[0] - 560.0).abs() < 1e-10);
        assert_eq!(a[1], b[1]);
    }
    let mut rotated = pose;
    rotated.rotation = 90.0;
    let turned = engine.corners("warten", &rotated, false).unwrap();
    assert!((turned[0][0] - (280.0 - (corners[0][1] - 490.0))).abs() < 1e-10);
    assert!((turned[0][1] - (490.0 + (corners[0][0] - 280.0))).abs() < 1e-10);
}
#[test]
fn damaged_preferences_are_sanitized() {
    let settings = Settings {
        energy: f64::NAN,
        speed: f64::INFINITY,
        ..Default::default()
    }
    .sanitize();
    assert_eq!(settings.energy, 0.7);
    assert_eq!(settings.speed, 1.0);
    let settings = Settings {
        energy: 2.0,
        speed: 0.0,
        ..Default::default()
    }
    .sanitize();
    assert_eq!(settings.energy, 1.0);
    assert_eq!(settings.speed, 0.25);
    let settings: Settings = serde_json::from_str("{}").unwrap();
    assert!(settings.always_on_top);
}
