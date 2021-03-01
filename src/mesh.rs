extern crate cgmath;
extern crate stl_io;
extern crate tobj;

use std::collections::hash_map::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;

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
    stl_had_normals: bool,
}

impl Mesh {
    pub fn from_obj(mut obj_file: File) -> Result<Mesh, Box<dyn Error>> {
        let mut input = BufReader::new(obj_file);
        let (models, _) =
            tobj::load_obj_buf(&mut input, true, |_| Ok((Vec::new(), HashMap::new())))?;
        let first_mesh = &models.iter().next().ok_or("Empty Model")?.mesh;
        let mut first_vertex = first_mesh
            .positions
            .iter();
        let mut mesh = Mesh {
            vertices: Vec::with_capacity(first_mesh.positions.len() / 3),
            normals: Vec::with_capacity(first_mesh.normals.len() / 3),
            indices: Vec::with_capacity(first_mesh.indices.len() / 3),
            bounds: BoundingBox::new(&[
                *first_vertex.next().ok_or("Empty Mesh")?,
                *first_vertex.next().ok_or("Empty Mesh")?,
                *first_vertex.next().ok_or("Empty Mesh")?,
            ]),
            stl_had_normals: true,
        };
        for model in &models {
            let tri_idx = &model.mesh.indices;
            let p = &model.mesh.positions;
            let n = &model.mesh.normals;
            for i in (0..tri_idx.len()).step_by(3) {
                let index0: usize = tri_idx[i] as usize;
                let index1: usize = tri_idx[i + 1] as usize;
                let index2: usize = tri_idx[i + 2] as usize;

                let vertices = [
                    ([p[index0 * 3], p[index0 * 3 + 1], p[index0 * 3 + 2]]),
                    ([p[index1 * 3], p[index1 * 3 + 1], p[index1 * 3 + 2]]),
                    ([p[index2 * 3], p[index2 * 3 + 1], p[index2 * 3 + 2]]),
                ];
                for v in vertices.iter() {
                    mesh.bounds.expand(&v);
                    mesh.vertices.push(Vertex { position: *v });
                    //debug!("{:?}", v);
                }

                let normals = if n.len() > 0 {
                    [
                        ([n[index0 * 3], n[index0 * 3 + 1], n[index0 * 3 + 2]]),
                        ([n[index1 * 3], n[index1 * 3 + 1], n[index1 * 3 + 2]]),
                        ([n[index2 * 3], n[index2 * 3 + 1], n[index2 * 3 + 2]]),
                    ]
                } else {
                    let n = normal(vertices);
                    [n, n, n]
                };
                for normal in normals.iter() {
                    mesh.normals.push(Normal { normal: *normal });
                }
            }
        }
        return Ok(mesh);
    }
    pub fn from_stl(mut stl_file: File) -> Result<Mesh, Box<dyn Error>> {
        //let stl = stl_io::read_stl(&mut stl_file)?;
        //debug!("{:?}", stl);
        let mut stl_iter = stl_io::create_stl_reader(&mut stl_file)?;

        // Get starting point for finding bounding box
        let t1 = stl_iter.next().unwrap().unwrap();
        let v1 = t1.vertices[0];

        let mut mesh = Mesh {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            bounds: BoundingBox::new(&v1),
            stl_had_normals: true,
        };

        let mut face_count = 0;
        mesh.process_tri(&t1);
        face_count += 1;

        for triangle in stl_iter {
            mesh.process_tri(&triangle?);
            face_count += 1;
            //debug!("{:?}",triangle);
        }

        if !mesh.stl_had_normals {
            warn!("STL file missing surface normals");
        }
        info!("Bounds:");
        info!("{}", mesh.bounds);
        info!("Center:\t{:?}", mesh.bounds.center());
        info!("Triangles processed:\t{}\n", face_count);

        Ok(mesh)
    }

    fn process_tri(&mut self, tri: &stl_io::Triangle) {
        for v in tri.vertices.iter() {
            self.bounds.expand(&v);
            self.vertices.push(Vertex { position: *v });
            //debug!("{:?}", v);
        }
        // Use normal from STL file if it is provided, otherwise calculate it ourselves
        let n: stl_io::Normal;
        if tri.normal == [0.0, 0.0, 0.0] {
            self.stl_had_normals = false;
            n = normal(tri.vertices);
        } else {
            n = tri.normal;
        }
        //debug!("{:?}",tri.normal);
        // TODO: Figure out how to get away with 1 normal instead of 3
        for _ in 0..3 {
            self.normals.push(Normal { normal: n });
        }
    }

    // Move the mesh to be centered at the origin
    // and scaled to fit a 2 x 2 x 2 box. This means that
    // all coordinates will be between -1.0 and 1.0
    pub fn scale_and_center(&self) -> cgmath::Matrix4<f32> {
        // Move center to origin
        let center = self.bounds.center();
        let translation_vector = cgmath::Vector3::new(
            -center.x,
            -center.y,
            -center.z,
        );
        let translation_matrix = cgmath::Matrix4::from_translation(translation_vector);
        // Scale
        let longest = self.bounds.length()
            .max(self.bounds.width())
            .max(self.bounds.height());
        let scale = 2.0 / longest;
        info!("Scale:\t{}",scale);
        let scale_matrix = cgmath::Matrix4::from_scale(scale);
        let matrix = scale_matrix * translation_matrix;
        matrix
    }
}

impl fmt::Display for Mesh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Verts: {}", self.vertices.len())?;
        writeln!(f, "Norms: {}", self.normals.len())?;
        //writeln!(f, "Tex Coords: {:?}", geometry.tex_coords)?;
        writeln!(f, "Indices: {:?}", self.indices.len())?;
        writeln!(f)?;
        Ok(())
    }
}

// Calculate surface normal of triangle using cross product
// TODO: The GPU can probably do this a lot faster than we can.
// See if there is an option for offloading this.
// Probably need to use a geometry shader (not supported in Opengl ES).
fn normal(vertices: [[f32; 3]; 3]) -> stl_io::Normal {
    let p1: cgmath::Vector3<f32> = vertices[0].into();
    let p2: cgmath::Vector3<f32> = vertices[1].into();
    let p3: cgmath::Vector3<f32> = vertices[2].into();
    let v = p2 - p1;
    let w = p3 - p1;
    let n = v.cross(w);
    let mag = n.x.abs() + n.y.abs() + n.z.abs();
    [n.x / mag, n.y / mag, n.z / mag]
}
