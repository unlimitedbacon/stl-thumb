extern crate cgmath;
#[macro_use]
extern crate glium;
extern crate image;
extern crate mint;

pub mod config;
mod mesh;

use std::borrow::Cow;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};
use config::Config;
use cgmath::Rotation;
use glium::{glutin, Surface};
use mesh::Mesh;

// TODO: Move this stuff to config module
const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const BACKGROUND_COLOR: (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 0.0);
const CAM_AZIMUTH_DEG: f32 = -60.0;
const CAM_ELEVATION_DEG: f32 = 30.0;
const CAM_FOV_DEG: f32 = 30.0;

fn locate_camera(bounds: &mesh::BoundingBox) -> mint::Point3<f32> {
    // Transform bounding box into camera space
    let p1 = bounds.max - bounds.center();
    let rot: cgmath::Basis2<f32> = cgmath::Rotation2::from_angle(cgmath::Deg(-CAM_AZIMUTH_DEG));
    let p2 = rot.rotate_vector(cgmath::Vector2{
        x: p1.x,
        y: p1.y,
    });
    // TODO: three-rs uses vertical FOV but we are using horizontal FOV here.
    // Adjust accordingly.
    let d = p2.y / (CAM_FOV_DEG.to_radians()/2.0).tan() + p2.x;
    mint::Point3 {
        x: d * CAM_AZIMUTH_DEG.to_radians().cos() + bounds.center().x,
        y: d * CAM_AZIMUTH_DEG.to_radians().sin() + bounds.center().y,
        z: d * CAM_ELEVATION_DEG.to_radians().tan() + bounds.center().z,
    }
    // TODO: Account for object that are taller than wide
}

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}


pub fn run(config: &Config) -> Result<(), Box<Error>> {
    // Create geometry from STL file
    // =========================

    let stl_file = File::open(&config.stl_filename)?;
    let mesh = Mesh::from_stl(stl_file)?;
    let center = mesh.bounds.center();


    // Graphics Stuff
    // ==============

    // Create GL context
    // -----------------

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("stl-thumb")
        .with_dimensions(WIDTH, HEIGHT)
        .with_min_dimensions(WIDTH, HEIGHT)
        .with_max_dimensions(WIDTH, HEIGHT);
    let context = glutin::ContextBuilder::new()
        .with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    //let context = glutin::HeadlessRendererBuilder::new(WIDTH, HEIGHT)
    //    //.with_depth_buffer(24)
    //    .build().unwrap();
    //let display = glium::HeadlessRenderer::new(context).unwrap();

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
        .. Default::default()
    };

    // Load and compile shaders
    // ------------------------

    let mut vertex_shader_file = File::open("src/model.vert")
        .expect("Error opening vertex shader file");
    let mut vertex_shader_src = String::new();
    vertex_shader_file.read_to_string(&mut vertex_shader_src)
        .expect("Error reading vertex shader file");
    let mut pixel_shader_file = File::open("src/model.frag")
        .expect("Error opening pixel shader file");
    let mut pixel_shader_src = String::new();
    pixel_shader_file.read_to_string(&mut pixel_shader_src)
        .expect("Error reading pixel shader file");

    // TODO: Cache program binary
    let program = glium::Program::from_source(&display, &vertex_shader_src, &pixel_shader_src, None);
    let program = match program {
        Ok(p) => p,
        Err(glium::CompilationError(err)) => {
            eprintln!("{}",err);
            panic!("Compiling shaders");
        },
        Err(err) => panic!("{}",err),
    };

    // Send mesh data to GPU
    // ---------------------

    let vertex_buf = glium::VertexBuffer::new(&display, &mesh.vertices).unwrap();
    let normal_buf = glium::VertexBuffer::new(&display, &mesh.normals).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    // Setup uniforms
    // --------------

    // Transformation matrix (positions, scales and rotates model)
    let model = [
        [0.01, 0.0, 0.0, 0.0],
        [0.0, 0.01, 0.0, 0.0],
        [0.0, 0.0, 0.01, 0.0],
        [0.0, 0.0, 2.0, 1.0f32],
    ];

    // View matrix (convert to positions relative to camera)
    let view = view_matrix(&[2.0, 1.0, 1.0], &[-2.0, -1.0, 1.0], &[0.0, 1.0, 0.0]);

    // Perspective matrix (give illusion of depth)
    let perspective = {
        let (width, height) = (WIDTH, HEIGHT);
        let aspect_ratio = height as f32 / width as f32;

        let fov = CAM_FOV_DEG.to_radians();
        let zfar = 1024.0;
        let znear = 0.1;

        let f = 1.0 / (fov / 2.0).tan();

        [
            [f * aspect_ratio, 0.0,                            0.0, 0.0],
            [             0.0,   f,                            0.0, 0.0],
            [             0.0, 0.0,      (zfar+znear)/(zfar-znear), 1.0],
            [             0.0, 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],
        ]
    };

    // Direction of light source
    let light = [-1.4, 0.4, -0.7f32];

    let uniforms = uniform! {
        model: model,
        view: view,
        perspective: perspective,
        u_light: light,
    };

    // Draw
    // ----

    {
        let mut target = display.draw();
        // Fills background color and clears depth buffer
        target.clear_color_and_depth(BACKGROUND_COLOR, 1.0);
        // Can use NoIndices here because STLs are dumb
        target.draw((&vertex_buf, &normal_buf), &indices, &program, &uniforms, &params)
            .unwrap();
        target.finish().unwrap();
    }

    // Save Image
    // ==========

    let (width, height) = display.get_framebuffer_dimensions();
    let pixels: glium::texture::RawImage2d<u8> = display.read_front_buffer();
    let img = image::ImageBuffer::from_raw(width, height, pixels.data.into_owned()).unwrap();
    let img = image::DynamicImage::ImageRgba8(img).flipv();
    img.save(&config.img_filename)
        .expect("Error saving image");

    // Wait until window is closed
    // ===========================

    //let mut closed = false;
    //let sleep_time = time::Duration::from_millis(10);
    //while !closed {
    //    thread::sleep(sleep_time);
    //    // Listing the events produced by the application and waiting to be received
    //    events_loop.poll_events(|ev| {
    //        match ev {
    //            glutin::Event::WindowEvent { event, .. } => match event {
    //                glutin::WindowEvent::Closed => closed = true,
    //                _ => (),
    //            },
    //            _ => (),
    //        }
    //    });
    //}

    //let mut window = three::Window::new(env!("CARGO_PKG_NAME"));
    //window.scene.background = three::Background::Color(0xFFFFFF);

    //println!("== STL ==");
    //mesh.debug();
    //let material = three::material::Phong {
    //    color: 0x00A0ff,
    //    glossiness: 80.0,
    //};
    //let mesh = window.factory.mesh(geometry, material);
    //window.scene.add(&mesh);

    ////let camera = window.factory.orthographic_camera(cam_center, yextent, zrange);
    //let camera = window.factory.perspective_camera(CAM_FOV_DEG, 1.0..500.0);
    ////let cam_pos = [150.0, -150.0, 150.0];
    //let cam_pos = locate_camera(&bounds);
    //camera.set_position(cam_pos);
    //camera.look_at(cam_pos, center, None);

    // Plane
    //let plane = {
    //    let geometry = Geometry::plane(
    //        (bounds.max.x - bounds.min.x) * 3.0,
    //        (bounds.max.y - bounds.min.y) * 3.0,
    //    );
    //    let material = three::material::Lambert {
    //        //color: 0xA0ffA0,
    //        color: 0xffffff,
    //        flat: false,
    //    };
    //    window.factory.mesh(geometry, material)
    //};
    //plane.set_position([center[0], center[1], bounds.min.z]);
    //window.scene.add(&plane);

    // Test sphere
    //let sphere = {
    //    let geometry = Geometry::uv_sphere(40.0, 20, 20);
    //    println!("== Sphere ==");
    //    debug_geo(&geometry);
    //    let material = three::material::Phong {
    //        color: 0xffA0A0,
    //        glossiness: 80.0,
    //    };
    //    window.factory.mesh(geometry, material)
    //};
    //sphere.set_position([30.0, 40.0, 2.5]);
    //window.scene.add(&sphere);

    // Lights
    //let hemisphere_light = window.factory.hemisphere_light(0xffffff, 0x8080ff, 0.5);
    //window.scene.add(&hemisphere_light);
    // TODO: Change dir_light position and shadow map size based on size of object
    //let mut dir_light = window.factory.directional_light(0xffffff, 0.9);
    //dir_light.look_at([-100.0, -400.0, 350.0], [0.0, 0.0, 0.0], None);
    //let shadow_map = window.factory.shadow_map(2048, 2048);
    //dir_light.set_shadow(shadow_map, 400.0, 1.0..1000.0);
    //window.scene.add(&dir_light);
    //let ambient_light = window.factory.ambient_light(0xdc8874, 0.5);
    //window.scene.add(ambient_light);
    //let point_light = window.factory.point_light(0xffffff, 0.5);
    //point_light.set_position([150.0, 350.0, 350.0]);
    //window.scene.add(point_light);

    //while window.update() {
    //    window.render(&camera);
    //}

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::ErrorKind;
    use super::*;

    #[test]
    fn cube() {
        let config = Config {
            stl_filename: "cube.stl".to_string(),
            img_filename: "cube.png".to_string(),
        };

        match fs::remove_file(&config.img_filename) {
            Ok(_) => (),
            Err(ref error) if error.kind() == ErrorKind::NotFound => (),
            Err(_) => {
                panic!("Couldn't clean files before testing");
            }
        }

        run(&config).expect("Error in run function");

        let size = fs::metadata(config.img_filename)
            .expect("No file created")
            .len();

        assert_ne!(0, size);
    }
}
