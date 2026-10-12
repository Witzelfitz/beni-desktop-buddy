//! Deterministic, headless Beni animation engine. No OS, network or GPU access.
pub mod notifications;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, error::Error, f64::consts::PI, fmt};

pub const ATLAS_PNG: &[u8] = include_bytes!("../../../assets/beni/atlas.png");
const ATLAS_JSON: &str = include_str!("../../../assets/beni/atlas.json");
const ANIMATIONS_JSON: &str = include_str!("../../../assets/beni/animations.json");
const ALIASES_JSON: &str = include_str!("../../../assets/beni/aliases.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyError(pub String);
impl fmt::Display for BuddyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl Error for BuddyError {}
type Result<T> = std::result::Result<T, BuddyError>;
fn error(message: impl Into<String>) -> BuddyError {
    BuddyError(message.into())
}

#[derive(Debug, Clone, Deserialize)]
pub struct Atlas {
    pub scale: f64,
    pub window: [f64; 2],
    pub anchor: [f64; 2],
    /// [source x, source y, width, height, anchor-relative x, anchor-relative y].
    pub frames: BTreeMap<String, [i32; 6]>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Segment {
    pub f: String,
    pub d: f64,
    #[serde(default)]
    pub pop: f64,
    #[serde(default)]
    pub hop: f64,
    #[serde(default)]
    pub shake: f64,
    #[serde(default)]
    pub mirror: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Motion {
    pub t: String,
    pub a: f64,
    pub c: f64,
    #[serde(default)]
    pub p: f64,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Animation {
    pub label: String,
    pub segs: Vec<Segment>,
    #[serde(default)]
    pub motion: Vec<Motion>,
    #[serde(default)]
    pub pivot: f64,
    pub group: String,
}
impl Animation {
    pub fn duration_ms(&self) -> f64 {
        self.segs.iter().map(|s| s.d).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pose {
    pub frame: String,
    pub mirror: bool,
    pub tx: f64,
    pub ty: f64,
    pub rotation: f64,
    pub sx: f64,
    pub sy: f64,
    pub index: usize,
    pub progress: f64,
}
#[derive(Debug, Clone)]
pub struct Engine {
    pub atlas: Atlas,
    pub animations: BTreeMap<String, Animation>,
    aliases: BTreeMap<String, String>,
    pub image_size: [u32; 2],
}
impl Engine {
    pub fn embedded() -> Result<Self> {
        let image_size = png_dimensions(ATLAS_PNG)?;
        Self::from_json(ATLAS_JSON, ANIMATIONS_JSON, ALIASES_JSON, image_size)
    }
    pub fn from_json(
        atlas: &str,
        animations: &str,
        aliases: &str,
        image_size: [u32; 2],
    ) -> Result<Self> {
        let engine = Self {
            atlas: serde_json::from_str(atlas).map_err(|e| error(format!("Atlas: {e}")))?,
            animations: serde_json::from_str(animations)
                .map_err(|e| error(format!("Animationen: {e}")))?,
            aliases: serde_json::from_str(aliases).map_err(|e| error(format!("Aliase: {e}")))?,
            image_size,
        };
        engine.validate()?;
        Ok(engine)
    }
    pub fn normalize(&self, state: &str) -> Result<String> {
        let name = state.trim().to_lowercase();
        let name = self.aliases.get(&name).unwrap_or(&name);
        if self.animations.contains_key(name) {
            Ok(name.clone())
        } else {
            Err(error(format!("Unbekannte Animation: {state}")))
        }
    }
    pub fn validate(&self) -> Result<()> {
        if !self.atlas.scale.is_finite()
            || self.atlas.scale <= 0.0
            || self
                .atlas
                .window
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0)
            || self.atlas.anchor.iter().any(|v| !v.is_finite())
            || self.image_size.contains(&0)
        {
            return Err(error("Ungültige Atlas-Geometrie"));
        }
        if self.atlas.frames.is_empty() || !self.animations.contains_key("warten") {
            return Err(error("Atlas-Posen oder Idle-Animation fehlen"));
        }
        for (name, [x, y, w, h, _, _]) in &self.atlas.frames {
            if *x < 0
                || *y < 0
                || *w <= 0
                || *h <= 0
                || i64::from(*x) + i64::from(*w) > i64::from(self.image_size[0])
                || i64::from(*y) + i64::from(*h) > i64::from(self.image_size[1])
            {
                return Err(error(format!("Pose ausserhalb des Atlas: {name}")));
            }
        }
        for (name, animation) in &self.animations {
            if animation.segs.is_empty() || !animation.pivot.is_finite() {
                return Err(error(format!("Leere oder ungültige Animation: {name}")));
            }
            for s in &animation.segs {
                if !self.atlas.frames.contains_key(&s.f)
                    || !s.d.is_finite()
                    || s.d <= 0.0
                    || [s.pop, s.hop, s.shake].iter().any(|v| !v.is_finite())
                {
                    return Err(error(format!("Ungültiges Segment in {name}")));
                }
            }
            if !animation.duration_ms().is_finite() {
                return Err(error(format!("Ungültige Dauer in {name}")));
            }
            for m in &animation.motion {
                if !["breathe", "sway", "bob", "bobs", "nudgex", "shakex"].contains(&m.t.as_str())
                    || [m.a, m.c, m.p].iter().any(|v| !v.is_finite())
                {
                    return Err(error(format!("Ungültige Bewegung in {name}")));
                }
            }
        }
        for (alias, target) in &self.aliases {
            if !self.animations.contains_key(target) {
                return Err(error(format!("Ungültiger Alias: {alias}")));
            }
        }
        Ok(())
    }
    /// Matches Beni Allrounder's motion.mjs, including negative timeline wrapping.
    pub fn pose_at(&self, state: &str, time_ms: f64, energy: f64) -> Result<Pose> {
        if !time_ms.is_finite() || !energy.is_finite() || !(0.0..=1.0).contains(&energy) {
            return Err(error(
                "Zeit muss endlich sein; Energie muss zwischen 0 und 1 liegen",
            ));
        }
        let state = self.normalize(state)?;
        let animation = &self.animations[&state];
        let length = animation.duration_ms();
        let t = time_ms.rem_euclid(length);
        let mut start = 0.0;
        let mut index = 0;
        while index < animation.segs.len() - 1 && t >= start + animation.segs[index].d {
            start += animation.segs[index].d;
            index += 1;
        }
        let s = &animation.segs[index];
        let u = t - start;
        let mut pose = Pose {
            frame: s.f.clone(),
            mirror: s.mirror,
            tx: 0.0,
            ty: 0.0,
            rotation: 0.0,
            sx: 1.0,
            sy: 1.0,
            index,
            progress: t / length,
        };
        for m in &animation.motion {
            let phase = 2.0 * PI * (m.c * t / length + m.p);
            let a = m.a * energy;
            match m.t.as_str() {
                "breathe" => {
                    pose.sy *= 1.0 + a * phase.sin();
                    pose.sx *= 1.0 - a * 0.5 * phase.sin();
                }
                "sway" => pose.rotation += a * phase.sin(),
                "bob" => pose.ty -= a * (1.0 - phase.cos()) / 2.0,
                "bobs" => pose.ty -= a * phase.sin(),
                "nudgex" => pose.tx += a * (1.0 - phase.cos()) / 2.0,
                "shakex" => pose.tx += a * phase.sin(),
                _ => unreachable!("motions validated on import"),
            }
        }
        if s.pop != 0.0 {
            let v = energy * s.pop * (-u / 80.0).exp() * (2.0 * PI * u / 200.0).cos();
            pose.sy *= 1.0 - v;
            pose.sx *= 1.0 + 0.7 * v;
        }
        if s.hop != 0.0 {
            pose.ty -= energy * s.hop * (PI * u / s.d).sin();
        }
        if s.shake != 0.0 {
            pose.tx += energy * s.shake * (2.0 * PI * u / 80.0).sin();
        }
        Ok(pose)
    }
    /// Four logical canvas corners, transformed around the original atlas anchor.
    pub fn corners(&self, state: &str, pose: &Pose, mirror: bool) -> Result<[[f64; 2]; 4]> {
        let state = self.normalize(state)?;
        let pivot = self.animations[&state].pivot;
        let [_, _, w, h, ox, oy] = *self
            .atlas
            .frames
            .get(&pose.frame)
            .ok_or_else(|| error("Unbekannte Pose"))?;
        let scale = self.atlas.scale;
        let x = f64::from(ox) / scale;
        let y = f64::from(oy) / scale + pivot;
        let w = f64::from(w) / scale;
        let h = f64::from(h) / scale;
        let angle = pose.rotation.to_radians();
        let sx = pose.sx * if pose.mirror { -1.0 } else { 1.0 };
        let (sin, cos) = angle.sin_cos();
        Ok(
            [[x, y], [x + w, y], [x + w, y + h], [x, y + h]].map(|[x, y]| {
                let local_x = x * sx;
                let local_y = y * pose.sy;
                let world_x = self.atlas.anchor[0] + pose.tx + local_x * cos - local_y * sin;
                let world_y =
                    self.atlas.anchor[1] - pivot + pose.ty + local_x * sin + local_y * cos;
                [
                    if mirror {
                        self.atlas.window[0] - world_x
                    } else {
                        world_x
                    },
                    world_y,
                ]
            }),
        )
    }
}

pub fn png_dimensions(bytes: &[u8]) -> Result<[u32; 2]> {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return Err(error("Ungültiger PNG-Header"));
    }
    let width = u32::from_be_bytes(
        bytes[16..20]
            .try_into()
            .map_err(|_| error("PNG-Breite fehlt"))?,
    );
    let height = u32::from_be_bytes(
        bytes[20..24]
            .try_into()
            .map_err(|_| error("PNG-Höhe fehlt"))?,
    );
    if width == 0 || height == 0 {
        return Err(error("Leerer PNG-Atlas"));
    }
    Ok([width, height])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub energy: f64,
    pub speed: f64,
    pub reduced_motion: bool,
    pub mirrored: bool,
    pub always_on_top: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            energy: 0.7,
            speed: 1.0,
            reduced_motion: false,
            mirrored: false,
            always_on_top: true,
        }
    }
}
impl Settings {
    pub fn sanitize(mut self) -> Self {
        self.energy = if self.energy.is_finite() {
            self.energy.clamp(0.0, 1.0)
        } else {
            0.7
        };
        self.speed = if self.speed.is_finite() {
            self.speed.clamp(0.25, 3.0)
        } else {
            1.0
        };
        self
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    state: String,
    elapsed_ms: f64,
    once: bool,
    pub paused: bool,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            state: "warten".into(),
            elapsed_ms: 0.0,
            once: false,
            paused: false,
        }
    }
}
impl Player {
    pub fn state(&self) -> &str {
        &self.state
    }
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed_ms
    }
    pub fn play(&mut self, engine: &Engine, state: &str, once: bool) -> Result<()> {
        let state = engine.normalize(state)?;
        self.state = state;
        self.elapsed_ms = 0.0;
        self.once = once;
        self.paused = false;
        Ok(())
    }
    pub fn advance(&mut self, engine: &Engine, delta_ms: f64, speed: f64) -> Result<()> {
        if !delta_ms.is_finite()
            || delta_ms < 0.0
            || !speed.is_finite()
            || !(0.25..=3.0).contains(&speed)
        {
            return Err(error("Ungültiger Zeitschritt oder Tempo"));
        }
        if self.paused {
            return Ok(());
        }
        let next = self.elapsed_ms + delta_ms * speed;
        if !next.is_finite() {
            return Err(error("Zeitüberschreitung"));
        }
        if self.once && next >= engine.animations[&self.state].duration_ms() {
            self.state = "warten".into();
            self.elapsed_ms = 0.0;
            self.once = false;
        } else {
            self.elapsed_ms = next.rem_euclid(engine.animations[&self.state].duration_ms());
        }
        Ok(())
    }
    pub fn pose(&self, engine: &Engine, settings: &Settings) -> Result<Pose> {
        engine.pose_at(
            &self.state,
            if settings.reduced_motion {
                0.0
            } else {
                self.elapsed_ms
            },
            if settings.reduced_motion {
                0.0
            } else {
                settings.energy
            },
        )
    }
}
