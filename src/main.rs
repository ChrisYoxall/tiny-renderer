use std::error::Error;
use tiny_renderer::{Color, CullMode, Format, Mesh, ScreenPoint, TgaImage, rasterizer};

#[allow(dead_code)]
enum RenderMode {
    MeshRandomColors,
    Wireframe,
    BarycentricTriangle,
}

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 800;

    // Swap render mode below:
    let mode = RenderMode::MeshRandomColors;
    // let mode = RenderMode::Wireframe;
    // let mode = RenderMode::BarycentricTriangle;

    // Swap any of the sample Wavefront meshes below when using a mesh render mode:
    let model_path = "wavefront/african_head/african_head.obj";
    // let model_path = "wavefront/african_head/african_head_eye_inner.obj";
    // let model_path = "wavefront/african_head/african_head_eye_outer.obj";
    // let model_path = "wavefront/diablo3_pose/diablo3_pose.obj";
    // let model_path = "wavefront/boggie/head.obj";
    // let model_path = "wavefront/boggie/body.obj";
    // let model_path = "wavefront/boggie/eyes.obj";
    // let model_path = "wavefront/floor.obj";

    let mut framebuffer = TgaImage::new(WIDTH, HEIGHT, Format::Rgb);

    let summary = match mode {
        RenderMode::MeshRandomColors => {
            let mesh = Mesh::from_obj_file(model_path)?;
            rasterizer::mesh_random_colors(&mesh, &mut framebuffer, 42);
            format!(
                "'{}' ({} vertices, {} faces)",
                model_path,
                mesh.vertices().len(),
                mesh.faces().len()
            )
        }
        RenderMode::Wireframe => {
            let mesh = Mesh::from_obj_file(model_path)?;
            rasterizer::wireframe(&mesh, &mut framebuffer, Color::WHITE);
            format!(
                "wireframe for '{}' ({} vertices, {} faces)",
                model_path,
                mesh.vertices().len(),
                mesh.faces().len()
            )
        }
        RenderMode::BarycentricTriangle => {
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
            "barycentric RGB triangle".to_string()
        }
    };

    framebuffer.write_tga_file("framebuffer.tga")?;
    println!("Rendered {summary} to framebuffer.tga ({WIDTH}x{HEIGHT}).");

    Ok(())
}
