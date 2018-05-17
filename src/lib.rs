extern crate cgmath;
#[macro_use]
extern crate glium;
extern crate image;
extern crate mint;

pub mod config;
mod mesh;

use std::error::Error;
use std::fs::File;
use config::Config;
use cgmath::Rotation;
use mesh::Mesh;

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

pub fn run(config: &Config) -> Result<(), Box<Error>> {
    // Create geometry from STL file
    // =========================

    let stl_file = File::open(&config.stl_filename)?;
    let mesh = Mesh::from_stl(stl_file)?;
    let center = mesh.bounds.center();

    // Graphics Stuff
    // ==============

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
