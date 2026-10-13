//! Input geometry in egui points; independent of native window APIs.
use eframe::egui::{self, Pos2, Rect};

#[derive(Clone, Default)]
pub struct HitMap {
    width: usize,
    height: usize,
    pixels: Vec<bool>,
}
impl HitMap {
    pub fn reset(&mut self, size: egui::Vec2) {
        self.width = size.x.ceil().clamp(0.0, 4096.0) as usize;
        self.height = size.y.ceil().clamp(0.0, 4096.0) as usize;
        self.pixels.resize(self.width * self.height, false);
        self.pixels.fill(false);
    }
    pub fn contains(&self, point: Pos2) -> bool {
        point.x.is_finite()
            && point.y.is_finite()
            && point.x >= 0.0
            && point.y >= 0.0
            && (point.x as usize) < self.width
            && (point.y as usize) < self.height
            && self.pixels[point.y as usize * self.width + point.x as usize]
    }
    fn fill(&mut self, rect: Rect, mut opaque: impl FnMut(Pos2) -> bool) {
        let min_x = rect.min.x.floor().max(0.0) as usize;
        let min_y = rect.min.y.floor().max(0.0) as usize;
        let max_x = (rect.max.x.ceil().max(0.0) as usize).min(self.width);
        let max_y = (rect.max.y.ceil().max(0.0) as usize).min(self.height);
        for y in min_y..max_y {
            for x in min_x..max_x {
                if opaque(egui::pos2(x as f32 + 0.5, y as f32 + 0.5)) {
                    self.pixels[y * self.width + x] = true;
                }
            }
        }
    }
    pub fn add_rect(&mut self, rect: Rect, radius: f32) {
        let r = radius
            .max(0.0)
            .min(rect.width() / 2.0)
            .min(rect.height() / 2.0);
        self.fill(rect, |p| {
            let center = egui::pos2(
                p.x.clamp(rect.min.x + r, rect.max.x - r),
                p.y.clamp(rect.min.y + r, rect.max.y - r),
            );
            p.distance_sq(center) <= r * r
        });
    }
    pub fn add_mesh(&mut self, mesh: &egui::Mesh, alpha: &[u8], atlas_width: usize) {
        if mesh.vertices.len() != 4 || atlas_width == 0 {
            return;
        }
        let vertices = &mesh.vertices;
        let origin = vertices[0].pos;
        let a = vertices[1].pos - origin;
        let b = vertices[3].pos - origin;
        let determinant = a.x * b.y - a.y * b.x;
        if determinant.abs() < 0.001 {
            return;
        }
        let atlas_height = alpha.len() / atlas_width;
        let uv_a = vertices[1].uv - vertices[0].uv;
        let uv_b = vertices[3].uv - vertices[0].uv;
        let rect = Rect::from_points(&vertices.iter().map(|v| v.pos).collect::<Vec<_>>());
        self.fill(rect, |p| {
            let d = p - origin;
            let u = (d.x * b.y - d.y * b.x) / determinant;
            let v = (a.x * d.y - a.y * d.x) / determinant;
            if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
                return false;
            }
            let uv = vertices[0].uv + uv_a * u + uv_b * v;
            let x = (uv.x * atlas_width as f32) as usize;
            let y = (uv.y * atlas_height as f32) as usize;
            x < atlas_width && y < atlas_height && alpha[y * atlas_width + x] > 0
        });
    }
    /// Non-overlapping scanline rectangles, ordered by y then x.
    #[cfg(any(not(target_os = "macos"), test))]
    pub fn rectangles(&self, scale: f32) -> Vec<[i32; 4]> {
        let mut rectangles = Vec::new();
        if !scale.is_finite() || scale <= 0.0 {
            return rectangles;
        }
        let width = (self.width as f32 * scale).ceil() as usize;
        let height = (self.height as f32 * scale).ceil() as usize;
        for y in 0..height {
            let mut x = 0;
            let inside = |x| {
                self.contains(egui::pos2(
                    (x as f32 + 0.5) / scale,
                    (y as f32 + 0.5) / scale,
                ))
            };
            while x < width {
                if !inside(x) {
                    x += 1;
                    continue;
                }
                let start = x;
                while x < width && inside(x) {
                    x += 1;
                }
                rectangles.push([start as i32, y as i32, x as i32, (y + 1) as i32]);
            }
        }
        rectangles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rounded_ui_regions_leave_corners_and_hidden_controls_clear() {
        let mut map = HitMap::default();
        map.reset(egui::vec2(40.0, 40.0));
        map.add_rect(
            Rect::from_min_max(egui::pos2(5.0, 5.0), egui::pos2(25.0, 25.0)),
            5.0,
        );
        assert!(map.contains(egui::pos2(15.0, 15.0)));
        assert!(!map.contains(egui::pos2(5.0, 5.0)));
        assert!(!map.contains(egui::pos2(35.0, 35.0)));
        assert!(!map.contains(egui::pos2(f32::NAN, 10.0)));
        assert!(!map.contains(egui::pos2(-1.0, 10.0)));
        map.reset(egui::vec2(40.0, 40.0));
        assert!(map.rectangles(1.0).is_empty());
    }
    #[test]
    fn sprite_alpha_and_rotation_determine_input_geometry() {
        let mut map = HitMap::default();
        map.reset(egui::vec2(40.0, 40.0));
        let mut mesh = egui::Mesh::default();
        for (pos, uv) in [
            (egui::pos2(20.0, 5.0), egui::pos2(0.0, 0.0)),
            (egui::pos2(30.0, 15.0), egui::pos2(1.0, 0.0)),
            (egui::pos2(20.0, 25.0), egui::pos2(1.0, 1.0)),
            (egui::pos2(10.0, 15.0), egui::pos2(0.0, 1.0)),
        ] {
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv,
                color: egui::Color32::WHITE,
            });
        }
        map.add_mesh(&mesh, &[255, 0, 0, 255], 2);
        assert!(map.contains(egui::pos2(20.0, 10.0)));
        assert!(map.contains(egui::pos2(20.0, 20.0)));
        assert!(!map.contains(egui::pos2(25.0, 15.0)));
        assert!(!map.contains(egui::pos2(5.0, 5.0)));
        let rects = map.rectangles(2.0);
        assert!(!rects.is_empty());
        assert!(rects.iter().all(|r| r[0] < r[2] && r[1] < r[3]));
    }
    #[test]
    fn fractional_dpi_regions_are_sorted_and_do_not_overlap() {
        let mut map = HitMap::default();
        map.reset(egui::vec2(20.0, 20.0));
        map.add_rect(
            Rect::from_min_max(egui::pos2(2.0, 2.0), egui::pos2(8.0, 12.0)),
            0.0,
        );
        map.add_rect(
            Rect::from_min_max(egui::pos2(12.0, 2.0), egui::pos2(18.0, 12.0)),
            0.0,
        );
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let rectangles = map.rectangles(scale);
            for pair in rectangles.windows(2) {
                assert!(
                    pair[0][3] <= pair[1][1]
                        || (pair[0][1] == pair[1][1] && pair[0][2] <= pair[1][0])
                );
            }
        }
    }
}
