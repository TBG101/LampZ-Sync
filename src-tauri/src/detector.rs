use serde::{Deserialize, Serialize};

use crate::color::Rgb;
use std::collections::HashMap;

/// Normalized sub-rectangle of a frame. All values in [0.0, 1.0].
/// `left < right` and `top < bottom` are expected; we clamp defensively.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CaptureRegion {
    pub left:   f32,
    pub right:  f32,
    pub top:    f32,
    pub bottom: f32,
}

impl CaptureRegion {
    pub const FULL: Self = Self { left: 0.0, right: 1.0, top: 0.0, bottom: 1.0 };

    pub fn clamped(self) -> Self {
        let (mut l, mut r) = (self.left.clamp(0.0, 1.0), self.right.clamp(0.0, 1.0));
        let (mut t, mut b) = (self.top.clamp(0.0, 1.0), self.bottom.clamp(0.0, 1.0));
        if l > r { std::mem::swap(&mut l, &mut r); }
        if t > b { std::mem::swap(&mut t, &mut b); }
        Self { left: l, right: r, top: t, bottom: b }
    }
}

impl Default for CaptureRegion {
    fn default() -> Self { Self::FULL }
}

/// Returns the single most dominant (R, G, B) color across the **union**
/// of all supplied `regions`. Pixels outside any region are ignored.
///
/// Expects DXGI-style BGRA8 buffer (4 bytes/pixel: B, G, R, A).
pub fn detect_color(
    buffer: &[u8],
    width: u32,
    height: u32,
    regions: &[CaptureRegion],
) -> Option<Rgb> {
    let width_us  = width  as usize;
    let height_us = height as usize;
    let expected_len = width_us * height_us * 4;
    if buffer.len() < expected_len {
        eprintln!("buffer too small: got {}, expected {}", buffer.len(), expected_len);
        return None;
    }
    if regions.is_empty() {
        return None;
    }

    const SHIFT: u32   = 4;
    const BUCKETS: usize = 1 << (8 - SHIFT); // 16

    const TARGET_SAMPLES_PER_REGION: usize = 2500;

    let mut counts: HashMap<u16, u32>              = HashMap::with_capacity(64);
    let mut sums:   HashMap<u16, (u32, u32, u32)>  = HashMap::with_capacity(64);

    let row_stride_bytes = width_us * 4;

    for region in regions {
        let r = region.clamped();

        // Convert normalized -> pixel bounds (inclusive-exclusive).
        let x_start = (r.left   * width  as f32) as usize;
        let x_end   = ((r.right  * width  as f32).ceil() as usize).clamp(x_start + 1, width_us);
        let y_start = (r.top    * height as f32) as usize;
        let y_end   = ((r.bottom * height as f32).ceil() as usize).clamp(y_start + 1, height_us);

        let rw = x_end - x_start;
        let rh = y_end - y_start;
        let region_pixels = rw.checked_mul(rh)?;
        if region_pixels == 0 { continue; }

        // Spread samples evenly in 2D: pick sqrt(stride) per axis.
        let stride_pixels = (region_pixels / TARGET_SAMPLES_PER_REGION).max(1);
        let stride_y = ((stride_pixels as f32).sqrt().round() as usize).max(1);
        let stride_x = (stride_pixels / stride_y).max(1);

        let mut y = y_start;
        while y < y_end {
            let row_offset = y * row_stride_bytes;
            let mut x = x_start;
            while x < x_end {
                let i = row_offset + x * 4;

                // BGRA layout
                let b = buffer[i];
                let g = buffer[i + 1];
                let r_val = buffer[i + 2];

                let key = pack_key(r_val >> SHIFT, g >> SHIFT, b >> SHIFT, BUCKETS as u8);

                *counts.entry(key).or_insert(0) += 1;
                let entry = sums.entry(key).or_insert((0, 0, 0));
                entry.0 += r_val as u32;
                entry.1 += g as u32;
                entry.2 += b as u32;

                x += stride_x;
            }
            y += stride_y;
        }
    }

    let (best_key, best_count) = counts.into_iter().max_by_key(|&(_, n)| n)?;
    if best_count == 0 { return None; }

    let (sr, sg, sb) = sums[&best_key];
    let n = best_count as f32;

    // NOTE: original code had a bug — used `sg` for both r and g.
    Some(Rgb {
        r: sr as f32 / n,
        g: sg as f32 / n,
        b: sb as f32 / n,
    })
}

#[inline]
fn pack_key(r: u8, g: u8, b: u8, buckets: u8) -> u16 {
    let b_sz = buckets as u16;
    (r as u16) * b_sz * b_sz + (g as u16) * b_sz + (b as u16)
}