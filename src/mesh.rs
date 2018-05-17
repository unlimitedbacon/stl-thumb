extern crate cgmath;
extern crate stl_io;

use std::error::Error;
use std::fs::File;
use std::fmt;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: stl_io::Vertex,
    //texcoords: [f32; 2],
}

implement_vertex!(Vertex, position);
//implement_vertex!(Vertex, position, texcoords);

#[derive(Copy, Clone)]
pub struct Normal {
    normal: stl_io::Normal,
}

implement_vertex!(Normal, normal);

pub struct BoundingBox {
    pub min: cgmath::Point3<f32>,
    pub max: cgmath::Point3<f32>,
}

impl BoundingBox {
    fn new(vert: &stl_io::Vertex) -> BoundingBox {
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
    fn expand(&mut self, vert: &stl_io::Vertex) {
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
    pub fn center(&self) -> cgmath::Point3<f32> {
        cgmath::Point3 {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
            z: (self.min.z + self.max.z) / 2.0,
        }
    }
    fn length(&self) -> f32 {
        self.max.x - self.min.x
    }
    fn width(&self) -> f32 {
        self.max.y - self.min.y
    }
    fn height(&self) -> f32 {
        self.max.z - self.min.z
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "X: {}, {}", self.min.x, self.max.x)?;
        writeln!(f, "Y: {}, {}", self.min.y, self.max.y)?;
        writeln!(f, "Z: {}, {}", self.min.z, self.max.z)?;
        Ok(())
    }
}


pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub normals: Vec<Normal>,
    pub indices: Vec<usize>,
    pub bounds: BoundingBox,
}

impl Mesh {
    pub fn from_stl(mut stl_file: File) -> Result<Mesh, Box<Error>> {
        //let stl = stl_io::read_stl(&mut stl_file)?;
        //println!("{:?}", stl);
        let mut stl_iter = stl_io::create_stl_reader(&mut stl_file).unwrap();

        // Get starting point for finding bounding box
        let t1 = stl_iter.next().unwrap().unwrap();
        let v1 = t1.vertices[0];

        let mut mesh = Mesh {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            bounds: BoundingBox::new(&v1),
        };

        let mut face_count = 0;
        mesh.process_tri(&t1);
        face_count += 1;

        for triangle in stl_iter {
            mesh.process_tri(&triangle.unwrap());
            face_count += 1;
            //println!("{:?}",triangle);
        }

        println!("Bounds:");
        println!("{}",mesh.bounds);
        println!("Center:");
        println!("{:?}", mesh.bounds.center());
        println!("Triangles processed:");
        println!("{}", face_count);
        println!();

        Ok(mesh)
    }

    fn process_tri(&mut self, tri: &stl_io::Triangle) {
        for v in tri.vertices.iter() {
            self.bounds.expand(&v);
            self.vertices.push( Vertex {
                position: *v,
            });
            //println!("{:?}", v);
        }
        // Use normal from STL file if it is provided, otherwise calculate it ourselves
        let n: stl_io::Normal;
        if tri.normal == [0.0, 0.0, 0.0] {
            println!("Calculating surface normal");
            n = normal(&tri);
        } else {
            n = tri.normal;
        }
        //println!("{:?}",tri.normal);
        for _ in 0..3 {
            self.normals.push( Normal{
                normal: n,
            });
        }
    }
}

impl fmt::Display for Mesh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Verts: {}", self.vertices.len())?;
        writeln!(f, "Norms: {}", self.normals.len())?;
        //writeln!(f, "Tex Coords: {:?}", geometry.tex_coords)?;
        writeln!(f, "Indices: {:?}", self.indices.len())?;
        writeln!(f, )?;
        Ok(())
    }
}

// Calculate surface normal of triangle using cross product
// TODO: The GPU can probably do this a lot faster than we can.
// See if there is an option for offloading this.
fn normal(tri: &stl_io::Triangle) -> stl_io::Normal {
    let p1: cgmath::Vector3<f32> = tri.vertices[0].into();
    let p2: cgmath::Vector3<f32> = tri.vertices[1].into();
    let p3: cgmath::Vector3<f32> = tri.vertices[2].into();
    let v = p2 - p1;
    let w = p3 - p1;
    let n = v.cross(w);
    let mag = n.x.abs() + n.y.abs() + n.z.abs();
    [
        n.x / mag,
        n.y / mag,
        n.z / mag,
    ]
}
