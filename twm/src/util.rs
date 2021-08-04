use crate::Display;
use num_traits::{PrimInt, AsPrimitive, FromPrimitive, signum};

pub fn bytes_to_string(buffer: &[i8]) -> String {
    buffer
        .iter()
        .take_while(|b| **b != 0)
        .map(|byte| char::from(*byte as u8))
        .collect::<String>()
}

pub fn to_widestring(string: &str) -> Vec<u16> {
    string.encode_utf16().chain(std::iter::once(0)).collect()
}

pub type RGB = (i32, i32, i32);

pub fn rgb_to_hex(rgb: RGB) -> i32 {
    ((rgb.0 & 0xff) << 16) + ((rgb.1 & 0xff) << 8) + (rgb.2 & 0xff)
}

pub fn hex_to_rgb(hex: i32) -> RGB {
    ((hex >> 16) & 0xFF, (hex >> 8) & 0xFF, hex & 0xFF)
}

pub fn scale_color(color: i32, factor: f64) -> i32 {
    let (mut red, mut green, mut blue) = hex_to_rgb(color);

    red = (red as f64 * factor).round() as i32;
    green = (green as f64 * factor).round() as i32;
    blue = (blue as f64 * factor).round() as i32;

    rgb_to_hex((red, green, blue))
}

pub fn points_to_pixels<T: PrimInt + AsPrimitive<i64> + FromPrimitive>(points: T, display: &Display) -> T {
    let signed_p: i64 = points.as_();
    let unsigned_p = signed_p.abs() as u64;
    const points_per_inch: u64 = 72;
    let dpi = display.dpi as u64;
    // Note the (points_per_inch << 1) rounds the result to the nearest integer
    let res = ((unsigned_p * dpi + (points_per_inch << 1)) / points_per_inch) as i64;
    FromPrimitive::from_i64(res * signum(signed_p)).unwrap()
}
