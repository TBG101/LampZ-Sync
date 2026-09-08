use crate::color::Rgb;
use std::collections::HashMap;

/// Returns the single most dominant (R, G, B) color in the frame.
/// Expects DXGI-style BGRA8 buffer (4 bytes/pixel: B, G, R, A).
///
/// Uses a coarse histogram over quantized RGB buckets instead of
/// clustering — O(n) single pass, no iteration, no convergence loop.
pub fn detect_color(buffer: &[u8], width: u32, height: u32) -> Option<Rgb> {
    let expected_len = (width as usize) * (height as usize) * 4;
    if buffer.len() < expected_len {
        eprintln!(
            "buffer too small: got {}, expected {}",
            buffer.len(),
            expected_len
        );
        return None;
    }

    let total_pixels = (width as usize) * (height as usize);
    if total_pixels == 0 {
        return None;
    }

    // Fixed sample budget regardless of resolution. 2-3k samples is plenty
    // to find the dominant color reliably.
    const TARGET_SAMPLES: usize = 2500;
    let stride_pixels = (total_pixels / TARGET_SAMPLES).max(1);
    let stride_bytes = stride_pixels * 4;

    // Quantize each channel into 16 buckets (>> 4) — coarse enough to be
    // fast and robust to noise, fine enough to distinguish real colors.
    const SHIFT: u32 = 4;
    const BUCKETS: usize = 1 << (8 - SHIFT); // 16

    // Track bucket counts AND the running sum of raw values per bucket,
    // so we can return the actual average color within the winning
    // bucket rather than just the bucket's quantized midpoint.
    let mut counts: HashMap<u16, u32> = HashMap::with_capacity(64);
    let mut sums: HashMap<u16, (u32, u32, u32)> = HashMap::with_capacity(64);

    let mut i = 0usize;
    while i < buffer.len() - 3 {
        let b = buffer[i];
        let g = buffer[i + 1];
        let r = buffer[i + 2];

        let key = pack_key(r >> SHIFT, g >> SHIFT, b >> SHIFT, BUCKETS as u8);

        *counts.entry(key).or_insert(0) += 1;
        let entry = sums.entry(key).or_insert((0, 0, 0));
        entry.0 += r as u32;
        entry.1 += g as u32;
        entry.2 += b as u32;

        i += stride_bytes;
    }

    let (best_key, best_count) = counts.into_iter().max_by_key(|&(_, n)| n)?;
    if best_count == 0 {
        return None;
    }

    let (sr, sg, sb) = sums[&best_key];
    let n = best_count as f32;

    Some(Rgb {
        r: sg as f32 / n,
        g: sg as f32 / n,
        b: sb as f32 / n,
    })
    // Some((sr as f32 / n, sg as f32 / n, sb as f32 / n))
}

#[inline]
fn pack_key(r: u8, g: u8, b: u8, buckets: u8) -> u16 {
    // buckets is 16 for an 8-bit>>4 quantization, fits in 4 bits/channel
    let b_sz = buckets as u16;
    (r as u16) * b_sz * b_sz + (g as u16) * b_sz + (b as u16)
}
