extern crate clap;
extern crate mint;
extern crate stl_io;
extern crate three;

use std::error::Error;
use std::fs::File;
use clap::{Arg, App};
use mint::{Point3, Vector3};
use stl_io::{Triangle, Vertex};
use three::{Geometry, Object};

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
                              .arg(Arg::with_name("STL_FILE")
                                       .help("STL file")
                                       .required(true)
                                       .index(1))
                              .arg(Arg::with_name("IMG_FILE")
                                       .help("Thumbnail image")
                                       .required(true)
                                       .index(2))
                              .get_matches();

        let stl_filename = matches.value_of("STL_FILE").unwrap().to_string();
        let img_filename = matches.value_of("IMG_FILE").unwrap().to_string();

        Config {stl_filename, img_filename}
    }
}

struct BoundingBox {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}

impl BoundingBox {
    fn new(vert: &Vertex) -> BoundingBox {
        BoundingBox {
            min_x: vert[0],
            min_y: vert[1],
            min_z: vert[2],
            max_x: vert[0],
            max_y: vert[1],
            max_z: vert[2],
        }
    }
    fn expand(&mut self, vert: &Vertex) {
        if vert[0] < self.min_x { self.min_x = vert[0]; }
        else if vert[0] > self.max_x { self.max_x = vert[0]; }
        if vert[1] < self.min_y { self.min_y = vert[1]; }
        else if vert[1] > self.max_y { self.max_y = vert[1]; }
        if vert[2] < self.min_z { self.min_z = vert[2]; }
        else if vert[2] > self.max_z { self.max_z = vert[2]; }
    }
    fn center(&self) -> Vertex {
        let x = (self.min_x + self.max_x) / 2.0;
        let y = (self.min_y + self.max_y) / 2.0;
        let z = (self.min_z + self.max_z) / 2.0;
        [ x, y, z ]
    }
}

// Calculate surface normal of triangle using cross product
// TODO: The GPU can probably do this a lot faster than we can.
// See if there is an option for offloading this.
fn normal(tri: &Triangle) -> Vector3<f32> {
    let p1 = tri.vertices[0];
    let p2 = tri.vertices[1];
    let p3 = tri.vertices[2];
    let vx = p2[0] - p1[0];
    let vy = p2[1] - p1[1];
    let vz = p2[2] - p1[2];
    let wx = p3[0] - p1[0];
    let wy = p3[1] - p1[1];
    let wz = p3[2] - p1[2];
    let nx = (vy * wz) - (vz * wy);
    let ny = (vz * wx) - (vx * wz);
    let nz = (vx * wy) - (vy * wx);
    let mag = nx.abs() + ny.abs() + nz.abs();
    let ax = nx / mag;
    let ay = ny / mag;
    let az = nz / mag;
    Vector3 { x: ax, y: ay, z: az }
}

fn process_tri(tri: &Triangle, geo: &mut Geometry, bounds: &mut BoundingBox) {
    for v in tri.vertices.iter() {
        bounds.expand(&v);
        // TODO: Should figure out how to do this with into() instead
        geo.base.vertices.push(
            Point3 { x: v[0], y: v[1], z: v[2] }
        );
        //println!("{:?}", v);
    }
    // Use normal from STL file if it is provided, otherwise calculate it ourselves
    let n: Vector3<f32>;
    if tri.normal == [0.0, 0.0, 0.0] {
        println!("Calculating surface normal");
        n = normal(&tri);
    } else {
        n = Vector3 { x: tri.normal[0], y: tri.normal[1], z: tri.normal[2] };
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

fn load_mesh(mut stl_file: File) -> Result<(Geometry,BoundingBox), Box<Error>> {
    //let stl = stl_io::read_stl(&mut stl_file)?;
    //println!("{:?}", stl);
    let mut stl_iter = stl_io::create_stl_reader(&mut stl_file).unwrap();

    // Get starting point for finding bounding box
    let t1 = stl_iter.next().unwrap().unwrap();
    let v1 = t1.vertices[0];
    let mut bounds = BoundingBox::new(&v1);

    let mut face_count = 0;
    let mut geometry = Geometry { .. Geometry::default() };

    process_tri(&t1, &mut geometry, &mut bounds);
    face_count += 1;

    for triangle in stl_iter {
        process_tri(&triangle.unwrap(), &mut geometry, &mut bounds);
        face_count += 1;
        //println!("{:?}",triangle);
    }

    println!("Bounds:");
    println!("X: {}, {}", bounds.min_x, bounds.max_x);
    println!("Y: {}, {}", bounds.min_y, bounds.max_y);
    println!("Z: {}, {}", bounds.min_z, bounds.max_z);
    println!("Center:");
    println!("{:?}", bounds.center());
    println!("Triangles processed:");
    println!("{}", face_count);
    println!();

    Ok((geometry,bounds))
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
    let camera = window.factory.perspective_camera(45.0, 1.0 .. 500.0);
    let cam_pos = [150.0, -150.0, 150.0];
    camera.set_position(cam_pos);
    camera.look_at(cam_pos, center, None);

    // Plane
    let plane = {
        let geometry = Geometry::plane(
            (bounds.max_x - bounds.min_x) * 3.0,
            (bounds.max_y - bounds.min_y) * 3.0,
        );
        let material = three::material::Lambert {
            //color: 0xA0ffA0,
            color: 0xffffff,
            flat: false,
        };
        window.factory.mesh(geometry, material)
    };
    plane.set_position([center[0], center[1], bounds.min_z]);
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
    let mut dir_light = window.factory.directional_light(0xffffff, 0.9);
    dir_light.look_at([150.0, 350.0, 350.0], [0.0, 0.0, 0.0], None);
    let shadow_map = window.factory.shadow_map(2048,2048);
    dir_light.set_shadow(shadow_map, 400.0, 1.0 .. 1000.0);
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
            img_filename: "cube.png".to_string()
        };

        match fs::remove_file(&config.img_filename) {
            Ok(_) => (),
            Err(ref error) if error.kind() == ErrorKind::NotFound => (),
            Err(_) => {
                panic!("Couldn't clean files before testing");
            }
        }

        run(&config)
            .expect("Error in run function");

        let size = fs::metadata(config.img_filename)
            .expect("No file created")
            .len();

        assert_ne!(0, size);
    }
}
