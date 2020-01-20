use rusttype::{Font, point, Scale};

pub fn texture_data_from_str(font: &Font, scale: f32, text: &str) -> (usize, Vec<f32>) {
    let scale = Scale {x: scale, y: scale};
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);
    let glyphs: Vec<_> = font.layout(text, scale, offset).collect();
    let width = (glyphs.last().unwrap().position().x + glyphs.last().unwrap().unpositioned().h_metrics().advance_width).ceil() as usize;
    let mut texture = vec![0.0; width * scale.y.ceil() as usize * 4];
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                if x >= 0 && x < width as i32 && y >= 0 && y < scale.y.ceil() as i32 {
                    let x = x as usize;
                    let y = y as usize;
                    texture[(x + y * width) * 4 + 3] = v;
                }
            });
        }
    }
    (width, texture)
}