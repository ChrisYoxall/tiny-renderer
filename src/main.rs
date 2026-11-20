use std::error::Error;
use tiny_renderer::{Color, Format, Mesh, TgaImage, rasterizer};

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 800;

    // Swap any of the sample Wavefront meshes below to render a different wireframe:
    let model_path = "wavefront/african_head/african_head.obj";
    // let model_path = "wavefront/african_head/african_head_eye_inner.obj";
    // let model_path = "wavefront/african_head/african_head_eye_outer.obj";
    // let model_path = "wavefront/diablo3_pose/diablo3_pose.obj";
    // let model_path = "wavefront/boggie/head.obj";
    // let model_path = "wavefront/boggie/body.obj";
    // let model_path = "wavefront/boggie/eyes.obj";
    // let model_path = "wavefront/floor.obj";

    let mesh = Mesh::from_obj_file(model_path)?;
    let mut framebuffer = TgaImage::new(WIDTH, HEIGHT, Format::Rgb);

    rasterizer::wireframe(&mesh, &mut framebuffer, Color::WHITE);

    framebuffer.write_tga_file("framebuffer.tga")?;
    println!(
        "Rendered wireframe for '{}' ({} vertices, {} faces) to framebuffer.tga ({}x{}).",
        model_path,
        mesh.vertices().len(),
        mesh.faces().len(),
        WIDTH,
        HEIGHT
    );

    Ok(())
}
