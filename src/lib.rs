extern crate clap;
extern crate mint;
extern crate stl_io;
extern crate three;

use std::error::Error;
use std::fs::File;
use clap::{Arg, App};
use mint::Point3;
use stl_io::{Triangle, Vertex};
use three::Object;

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

fn process_tri(tri: &Triangle, verts: &mut Vec<Point3<f32>>, bounds: &mut BoundingBox) {
    for v in tri.vertices.iter() {
        bounds.expand(&v);
        // TODO: Should figure out how to do this with into() instead
        verts.push(
            Point3 { x: v[0], y: v[1], z: v[2] }
        );
        //println!("{:?}", v);
    }
}

pub fn run(config: &Config) -> Result<(), Box<Error>> {
    let mut stl_file = File::open(&config.stl_filename)?;
    //let stl = stl_io::read_stl(&mut stl_file)?;
    //println!("{:?}", stl);

    let mut stl_iter = stl_io::create_stl_reader(&mut stl_file).unwrap();

    // Get starting point for finding bounding box
    let t1 = stl_iter.next().unwrap().unwrap();
    let v1 = t1.vertices[0];
    let mut bounds = BoundingBox::new(&v1);

    let mut face_count = 0;
    let mut geometry = three::Geometry { .. three::Geometry::default() };

    process_tri(&t1, &mut geometry.base.vertices, &mut bounds);
    face_count += 1;

    for triangle in stl_iter {
        process_tri(&triangle.unwrap(), &mut geometry.base.vertices, &mut bounds);
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

    // Graphics Stuff
    // ==============

    let mut window = three::Window::new(env!("CARGO_PKG_NAME"));
    //let geometry = three::Geometry::with_vertices(vec![
    //    [-0.5, -0.5, -0.5].into(),
    //    [ 0.5, -0.5, -0.5].into(),
    //    [ 0.0,  0.5, -0.5].into(),
    //]);
    let material = three::material::Wireframe {
        color: 0x888888,
    };
    let mesh = window.factory.mesh(geometry, material);
    window.scene.add(&mesh);
    window.scene.background = three::Background::Color(0xFFFFFF);

    let center = [0.0, 0.0];
    let yextent = 20.0;
    let zrange = -40.0 .. 40.0;
    let camera = window.factory.orthographic_camera(center, yextent, zrange);
    let cam_pos = [10.0, -10.0, 20.0];
    camera.set_position(cam_pos);
    camera.look_at(cam_pos, [0.0, 0.0, 0.0], None);

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
