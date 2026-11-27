use std::error::Error;
#[allow(unused_imports)]
use tiny_renderer::{Color, CullMode, Format, Mesh, ScreenPoint, TgaImage, rasterizer};

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 800;

    // Swap any of the sample Wavefront meshes below to render a different model:
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

    // Swap render mode below:
    //rasterizer::mesh_random_colors(&mesh, &mut framebuffer, 42);
    // rasterizer::wireframe(&mesh, &mut framebuffer, Color::WHITE);
    rasterizer::triangle_interpolated(
        [
            ScreenPoint::new(100, 100),
            ScreenPoint::new(700, 150),
            ScreenPoint::new(350, 700),
        ],
        [Color::RED, Color::GREEN, Color::BLUE],
        CullMode::None,
        &mut framebuffer,
    );

    framebuffer.write_tga_file("framebuffer.tga")?;
    println!(
        "Rendered '{}' ({} vertices, {} faces) to framebuffer.tga ({}x{}).",
        model_path,
        mesh.vertices().len(),
        mesh.faces().len(),
        WIDTH,
        HEIGHT
    );

    Ok(())
}
