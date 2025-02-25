use macroquad::color::Color;

pub fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> Color {
    let mut r = lightness;
    let mut g = lightness;
    let mut b = lightness;

    if saturation != 0.0 {
        let q = if lightness > 0.5 {
            lightness * (1.0 + saturation)
        } else {
            1.0 + saturation - lightness * saturation
        };
        let p = 2.0 * lightness - q;
        r = hue_to_rgb(p, q, hue + 1.0/3.0);
        g = hue_to_rgb(p, q, hue);
        b = hue_to_rgb(p, q, hue - 1.0/3.0);
    }

    Color::new(r, g, b, 1.0)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0/6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0/2.0 { return q; }
    if t < 2.0/3.0 { return p + (q - p) * (2.0/3.0 - t) * 6.0; }
    p
}
