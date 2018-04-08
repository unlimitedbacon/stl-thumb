extern crate cgmath;
extern crate clap;
extern crate mint;
extern crate stl_io;
extern crate three;

use std::error::Error;
use std::fs::File;
use clap::{App, Arg};
use cgmath::Rotation;
use stl_io::{Triangle, Vertex};
use three::{Geometry, Object};

const CAM_AZIMUTH_DEG: f32 = -60.0;
const CAM_ELEVATION_DEG: f32 = 30.0;
const CAM_FOV_DEG: f32 = 30.0;

pub struct Config {
    pub stl_filename: String,
    pub img_filename: String,
}

impl Config {
    pub fn new() -> Config {
        // Define command line arguments
        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .arg(
                Arg::with_name("STL_FILE")
                    .help("STL file")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("IMG_FILE")
                    .help("Thumbnail image")
                    .required(true)
                    .index(2),
            )
            .get_matches();

        let stl_filename = matches.value_of("STL_FILE").unwrap().to_string();
        let img_filename = matches.value_of("IMG_FILE").unwrap().to_string();

        Config {
            stl_filename,
            img_filename,
        }
    }
}

struct BoundingBox {
    min: cgmath::Point3<f32>,
    max: cgmath::Point3<f32>,
}

impl BoundingBox {
    fn new(vert: &Vertex) -> BoundingBox {
        BoundingBox {
            min: cgmath::Point3 {
                x: vert[0],
                y: vert[1],
                z: vert[2],
            },
            max: cgmath::Point3 {
                x: vert[0],
                y: vert[1],
                z: vert[2],
            },
        }
    }
    fn expand(&mut self, vert: &Vertex) {
        if vert[0] < self.min.x {
            self.min.x = vert[0];
        } else if vert[0] > self.max.x {
            self.max.x = vert[0];
        }
        if vert[1] < self.min.y {
            self.min.y = vert[1];
        } else if vert[1] > self.max.y {
            self.max.y = vert[1];
        }
        if vert[2] < self.min.z {
            self.min.z = vert[2];
        } else if vert[2] > self.max.z {
            self.max.z = vert[2];
        }
    }
    fn center(&self) -> cgmath::Point3<f32> {
        cgmath::Point3 {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
            z: (self.min.z + self.max.z) / 2.0,
        }
    }
}

// Calculate surface normal of triangle using cross product
// TODO: The GPU can probably do this a lot faster than we can.
// See if there is an option for offloading this.
fn normal(tri: &Triangle) -> mint::Vector3<f32> {
    let p1: cgmath::Vector3<f32> = tri.vertices[0].into();
    let p2: cgmath::Vector3<f32> = tri.vertices[1].into();
    let p3: cgmath::Vector3<f32> = tri.vertices[2].into();
    let v = p2 - p1;
    let w = p3 - p1;
    let n = v.cross(w);
    let mag = n.x.abs() + n.y.abs() + n.z.abs();
    mint::Vector3 {
        x: n.x / mag,
        y: n.y / mag,
        z: n.z / mag,
    }
}

fn process_tri(tri: &Triangle, geo: &mut Geometry, bounds: &mut BoundingBox) {
    for v in tri.vertices.iter() {
        bounds.expand(&v);
        // TODO: Should figure out how to do this with into() instead
        geo.base.vertices.push(mint::Point3 {
            x: v[0],
            y: v[1],
            z: v[2],
        });
        //println!("{:?}", v);
    }
    // Use normal from STL file if it is provided, otherwise calculate it ourselves
    let n: mint::Vector3<f32>;
    if tri.normal == [0.0, 0.0, 0.0] {
        println!("Calculating surface normal");
        n = normal(&tri);
    } else {
        n = mint::Vector3 {
            x: tri.normal[0],
            y: tri.normal[1],
            z: tri.normal[2],
        };
    }
    //println!("{:?}",tri.normal);
    for _ in 0..3 {
        geo.base.normals.push(n);
    }
}

fn debug_geo(geometry: &Geometry) {
    println!("Verts: {}", geometry.base.vertices.len());
    println!("Norms: {}", geometry.base.normals.len());
    println!("Tangents: {}", geometry.base.tangents.len());
    println!("Tex Coords: {:?}", geometry.tex_coords);
    println!("Faces: {:?}", geometry.faces.len());
    println!("Joints: {:?}", geometry.joints);
    println!("Shapes: {:?}", geometry.shapes);
    println!();
}

fn load_mesh(mut stl_file: File) -> Result<(Geometry, BoundingBox), Box<Error>> {
    //let stl = stl_io::read_stl(&mut stl_file)?;
    //println!("{:?}", stl);
    let mut stl_iter = stl_io::create_stl_reader(&mut stl_file).unwrap();

    // Get starting point for finding bounding box
    let t1 = stl_iter.next().unwrap().unwrap();
    let v1 = t1.vertices[0];
    let mut bounds = BoundingBox::new(&v1);

    let mut face_count = 0;
    let mut geometry = Geometry {
        ..Geometry::default()
    };

    process_tri(&t1, &mut geometry, &mut bounds);
    face_count += 1;

    for triangle in stl_iter {
        process_tri(&triangle.unwrap(), &mut geometry, &mut bounds);
        face_count += 1;
        //println!("{:?}",triangle);
    }

    println!("Bounds:");
    println!("X: {}, {}", bounds.min.x, bounds.max.x);
    println!("Y: {}, {}", bounds.min.y, bounds.max.y);
    println!("Z: {}, {}", bounds.min.z, bounds.max.z);
    println!("Center:");
    println!("{:?}", bounds.center());
    println!("Triangles processed:");
    println!("{}", face_count);
    println!();

    Ok((geometry, bounds))
}

fn locate_camera(bounds: &BoundingBox) -> mint::Point3<f32> {
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
    let (geometry, bounds) = load_mesh(stl_file)?;
    let center = bounds.center();

    // Graphics Stuff
    // ==============

    let mut window = three::Window::new(env!("CARGO_PKG_NAME"));
    window.scene.background = three::Background::Color(0xFFFFFF);

    println!("== STL ==");
    debug_geo(&geometry);
    let material = three::material::Phong {
        color: 0x00A0ff,
        glossiness: 80.0,
    };
    let mesh = window.factory.mesh(geometry, material);
    window.scene.add(&mesh);

    //let camera = window.factory.orthographic_camera(cam_center, yextent, zrange);
    let camera = window.factory.perspective_camera(CAM_FOV_DEG, 1.0..500.0);
    //let cam_pos = [150.0, -150.0, 150.0];
    let cam_pos = locate_camera(&bounds);
    camera.set_position(cam_pos);
    camera.look_at(cam_pos, center, None);

    // Plane
    let plane = {
        let geometry = Geometry::plane(
            (bounds.max.x - bounds.min.x) * 3.0,
            (bounds.max.y - bounds.min.y) * 3.0,
        );
        let material = three::material::Lambert {
            //color: 0xA0ffA0,
            color: 0xffffff,
            flat: false,
        };
        window.factory.mesh(geometry, material)
    };
    plane.set_position([center[0], center[1], bounds.min.z]);
    window.scene.add(&plane);

    // Test sphere
    let sphere = {
        let geometry = Geometry::uv_sphere(40.0, 20, 20);
        println!("== Sphere ==");
        debug_geo(&geometry);
        let material = three::material::Phong {
            color: 0xffA0A0,
            glossiness: 80.0,
        };
        window.factory.mesh(geometry, material)
    };
    sphere.set_position([30.0, 40.0, 2.5]);
    //window.scene.add(&sphere);

    // Lights
    //let hemisphere_light = window.factory.hemisphere_light(0xffffff, 0x8080ff, 0.5);
    //window.scene.add(&hemisphere_light);
    // TODO: Change dir_light position and shadow map size based on size of object
    let mut dir_light = window.factory.directional_light(0xffffff, 0.9);
    dir_light.look_at([-100.0, -400.0, 350.0], [0.0, 0.0, 0.0], None);
    let shadow_map = window.factory.shadow_map(2048, 2048);
    dir_light.set_shadow(shadow_map, 400.0, 1.0..1000.0);
    window.scene.add(&dir_light);
    let ambient_light = window.factory.ambient_light(0xdc8874, 0.5);
    window.scene.add(ambient_light);
    let point_light = window.factory.point_light(0xffffff, 0.5);
    point_light.set_position([150.0, 350.0, 350.0]);
    window.scene.add(point_light);

    while window.update() {
        window.render(&camera);
    }

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
