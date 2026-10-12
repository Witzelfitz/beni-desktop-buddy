//! Pure placement policy: Cocoa points on macOS, physical coordinates elsewhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl Bounds {
    pub fn valid(self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|n| n.is_finite())
            && self.width > 0.0
            && self.height > 0.0
    }
    pub fn contains(self, w: Self) -> bool {
        w.x >= self.x
            && w.y >= self.y
            && w.x + w.width <= self.x + self.width
            && w.y + w.height <= self.y + self.height
    }
    fn intersection(self, w: Self) -> f64 {
        ((self.x + self.width).min(w.x + w.width) - self.x.max(w.x)).max(0.0)
            * ((self.y + self.height).min(w.y + w.height) - self.y.max(w.y)).max(0.0)
    }
}
/// First screen is primary. Preserve valid positions; explicit recovery uses primary.
pub fn recovered_position(window: Bounds, screens: &[Bounds], reset: bool) -> Option<[f64; 2]> {
    if !window.valid() {
        return None;
    }
    let screens: Vec<_> = screens.iter().copied().filter(|s| s.valid()).collect();
    let mut target = *screens.first()?;
    if !reset && screens.iter().any(|s| s.contains(window)) {
        return None;
    }
    if !reset {
        let mut best = 0.0;
        for screen in &screens {
            let overlap = screen.intersection(window);
            if overlap > best {
                best = overlap;
                target = *screen;
            }
        }
    }
    let room_x = (target.width - window.width).max(0.0);
    let room_y = (target.height - window.height).max(0.0);
    let mx = 16.0_f64.min(room_x / 2.0);
    let my = 16.0_f64.min(room_y / 2.0);
    let x = if reset {
        target.x + room_x - mx
    } else {
        window.x.clamp(target.x + mx, target.x + room_x - mx)
    };
    let y = if reset {
        target.y + room_y - my
    } else {
        window.y.clamp(target.y + my, target.y + room_y - my)
    };
    Some([x, y])
}
#[cfg(test)]
mod tests {
    use super::*;
    fn screen(x: f64, y: f64) -> Bounds {
        Bounds {
            x,
            y,
            width: 1440.0,
            height: 900.0,
        }
    }
    fn buddy(x: f64, y: f64) -> Bounds {
        Bounds {
            x,
            y,
            width: 340.0,
            height: 440.0,
        }
    }
    #[test]
    fn valid_position_is_preserved() {
        assert_eq!(
            recovered_position(buddy(80.0, 60.0), &[screen(0.0, 0.0)], false),
            None
        );
    }
    #[test]
    fn disconnected_monitor_is_recovered() {
        let w = buddy(1700.0, 140.0);
        assert_eq!(
            recovered_position(w, &[screen(0.0, 0.0), screen(1440.0, 0.0)], false),
            None
        );
        let [x, y] = recovered_position(w, &[screen(0.0, 0.0)], false).unwrap();
        assert!(screen(0.0, 0.0).contains(buddy(x, y)));
    }
    #[test]
    fn negative_origin_and_partial_overlap() {
        let s = screen(-1440.0, -120.0);
        let [x, y] =
            recovered_position(buddy(-1500.0, 20.0), &[screen(0.0, 0.0), s], false).unwrap();
        assert!(s.contains(buddy(x, y)));
    }
    #[test]
    fn recovery_uses_primary() {
        let s = screen(0.0, 40.0);
        let [x, y] =
            recovered_position(buddy(-1000.0, 40.0), &[s, screen(-1440.0, 0.0)], true).unwrap();
        assert!(s.contains(buddy(x, y)));
    }
    #[test]
    fn small_screen_does_not_panic() {
        let s = Bounds {
            x: 0.0,
            y: 20.0,
            width: 280.0,
            height: 300.0,
        };
        assert_eq!(
            recovered_position(buddy(5000.0, 5000.0), &[s], false),
            Some([0.0, 20.0])
        );
    }
    #[test]
    fn missing_and_invalid_geometry() {
        assert_eq!(recovered_position(buddy(0.0, 0.0), &[], false), None);
        assert_eq!(
            recovered_position(buddy(f64::NAN, 0.0), &[screen(0.0, 0.0)], false),
            None
        );
        assert_eq!(
            recovered_position(buddy(9000.0, 0.0), &[screen(f64::INFINITY, 0.0)], false),
            None
        );
    }
}
