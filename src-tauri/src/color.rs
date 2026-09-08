// pub type Hsv = (u16, u16, u16);

pub struct Hsv {
    pub h: u16,
    pub s: u16,
    pub v: u16,
}

impl Hsv {
    pub fn hue_changed(&self, other: &Hsv, threshold: u16) -> bool {
        let diff = self.h.abs_diff(other.h);
        let circular_diff = diff.min(360 - diff);

        circular_diff >= threshold
    }
    pub fn saturation_changed(&self, other: &Hsv, threshold: u16) -> bool {
        let diff = self.s.abs_diff(other.h);
        diff >= threshold
    }
    pub fn v_changed(&self, other: &Hsv, threshold: u16) -> bool {
        let diff = self.v.abs_diff(other.h);
        diff >= threshold
    }
}

#[derive(Debug, Clone, Default)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub fn rgb_to_hsv(&self) -> Hsv {
        let rf = self.r as f64 / 255.0;
        let gf = self.g as f64 / 255.0;
        let bf = self.b as f64 / 255.0;

        let cmax = rf.max(gf).max(bf);
        let cmin = rf.min(gf).min(bf);
        let delta = cmax - cmin;

        let h = if delta == 0.0 {
            0.0
        } else if cmax == rf {
            60.0 * (((gf - bf) / delta).rem_euclid(6.0))
        } else if cmax == gf {
            60.0 * (((bf - rf) / delta) + 2.0)
        } else {
            60.0 * (((rf - gf) / delta) + 4.0)
        };

        let s = if cmax == 0.0 { 0.0 } else { delta / cmax };
        let v = cmax;
        Hsv {
            h: h.round() as u16,
            s: (s * 1000.0).round() as u16,
            v: (v * 1000.0).round() as u16,
        }
    }
}
