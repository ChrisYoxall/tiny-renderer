use std::error::Error;
use tiny_renderer::{Color, Format, TgaImage};

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 64;
    const HEIGHT: usize = 64;

    let mut framebuffer = TgaImage::new(WIDTH, HEIGHT, Format::Rgb);

    let (ax, ay) = (7, 3);
    let (bx, by) = (12, 37);
    let (cx, cy) = (62, 53);

    framebuffer.set(ax, ay, Color::WHITE);
    framebuffer.set(bx, by, Color::WHITE);
    framebuffer.set(cx, cy, Color::WHITE);

    framebuffer.write_tga_file("framebuffer.tga")?;
    println!("Generated framebuffer.tga ({}x{}) successfully.", WIDTH, HEIGHT);

    Ok(())
}
