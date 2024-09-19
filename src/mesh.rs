extern crate ahash;
extern crate cgmath;
extern crate stl_io;
extern crate tobj;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::{Cursor, Read, Seek};
use std::{fmt, io};

use self::stl_io::{Triangle, Vector};

use self::ahash::AHashMap;
use self::tobj::LoadOptions;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    //texcoords: [f32; 2],
}

implement_vertex!(Vertex, position);
//implement_vertex!(Vertex, position, texcoords);

#[derive(Debug, Copy, Clone)]
pub struct Normal {
    normal: [f32; 3],
}

implement_vertex!(Normal, normal);

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub normals: Vec<Normal>,
    pub indices: Vec<usize>,
    pub bounds: BoundingBox,
    stl_had_normals: bool,
}

impl Mesh {
    // Load mesh data from file (if provided) or stdin
    pub fn load(stl_filename: &str, recalc_normals: bool) -> Result<Mesh, Box<dyn Error>> {
        // TODO: Add support for URIs instead of plain file names
        // https://developer.gnome.org/integration-guide/stable/thumbnailer.html.en
        match stl_filename {
            "-" => {
                // create_stl_reader requires Seek, so we must read the entire stream into memory before proceeding.
                // So I guess this can just consume all RAM if it gets bad input. Hmmm....
                let mut input_buffer = Vec::new();
                io::stdin().read_to_end(&mut input_buffer)?;
                Mesh::from_stl(Cursor::new(input_buffer), recalc_normals)
            }
            _ => {
                let stl_filename = std::path::Path::new(stl_filename);
                // TODO: Try BufReader and see if it's faster
                let stl_file = File::open(stl_filename)?;
                Ok(
                    match stl_filename
                        .extension()
                        .and_then(std::ffi::OsStr::to_str)
                        .unwrap_or("")
                        .to_lowercase()
                        .as_str()
                    {
                        "obj" => Mesh::from_obj(stl_file, recalc_normals)?,
                        "stl" => Mesh::from_stl(stl_file, recalc_normals)?,
                        "3mf" => Mesh::from_3mf(stl_file, recalc_normals)?,
                        _ => unimplemented!("Format not supported"),
                    },
                )
            }
        }
    }

    pub fn from_3mf<R>(stl_file: R, _recalc_normals: bool) -> Result<Mesh, Box<dyn Error>>
    where
        R: Read + Seek,
    {
        let models = threemf::read(stl_file)?;

        let mut result = None;

        let vertex_translator = |vertex: &threemf::model::Vertex| {
            stl_io::Vertex::new([vertex.x as f32, vertex.y as f32, vertex.z as f32])
        };

        // Combine all the models into a single mesh.
        for model in models {
            for object in model.resources.object {
                if let Some(mesh) = object.mesh {
                    for triangle in &mesh.triangles.triangle {
                        // Re-use `Mesh::process_tri`, which creates new vertices for every
                        // triangle.
                        // Possible optimization: re-use triangles instead.
                        let triangle = stl_io::Triangle {
                            normal: stl_io::Normal::new([1f32, 0f32, 0f32]),
                            vertices: [
                                vertex_translator(&mesh.vertices.vertex[triangle.v1]),
                                vertex_translator(&mesh.vertices.vertex[triangle.v2]),
                                vertex_translator(&mesh.vertices.vertex[triangle.v3]),
                            ],
                        };
                        result
                            .get_or_insert_with(|| Mesh {
                                vertices: Vec::new(),
                                normals: Vec::new(),
                                indices: Vec::new(),
                                bounds: BoundingBox::new(&triangle.vertices[0]),
                                stl_had_normals: false,
                            })
                            .process_tri(&triangle, true);
                    }
                }
            }
        }

        Ok(result.unwrap())
    }

    pub fn from_stl<R>(mut stl_file: R, recalc_normals: bool) -> Result<Mesh, Box<dyn Error>>
    where
        R: Read + Seek,
    {
        //let stl = stl_io::read_stl(&mut stl_file)?;
        //debug!("{:?}", stl);
        let mut stl_iter = stl_io::create_stl_reader(&mut stl_file)?;

        // Get starting point for finding bounding box
        // TODO: Remove unwraps so lib can fail gracefully instead of panicing
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
        mesh.process_tri(&t1, recalc_normals);
        face_count += 1;

        for triangle in stl_iter {
            mesh.process_tri(&triangle?, recalc_normals);
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

    pub fn from_obj(obj_file: File, _recalc_normals: bool) -> Result<Mesh, Box<dyn Error>> {
        let mut input = BufReader::new(obj_file);
        let (models, _) = tobj::load_obj_buf(
            &mut input,
            &LoadOptions {
                single_index: true,
                triangulate: true,
                ..LoadOptions::default()
            },
            |_| Ok((Vec::new(), AHashMap::new())),
        )?;
        let first_mesh = &models.first().ok_or("Empty Model")?.mesh;
        let mut first_vertex = first_mesh.positions.iter();
        let mut mesh = Mesh {
            vertices: Vec::with_capacity(first_mesh.positions.len() / 3),
            normals: Vec::with_capacity(first_mesh.normals.len() / 3),
            indices: Vec::with_capacity(first_mesh.indices.len() / 3),
            bounds: BoundingBox::new(&Vector::new([
                *first_vertex.next().ok_or("Empty Mesh")?,
                *first_vertex.next().ok_or("Empty Mesh")?,
                *first_vertex.next().ok_or("Empty Mesh")?,
            ])),
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
                    Vector::new([p[index0 * 3], p[index0 * 3 + 1], p[index0 * 3 + 2]]),
                    Vector::new([p[index1 * 3], p[index1 * 3 + 1], p[index1 * 3 + 2]]),
                    Vector::new([p[index2 * 3], p[index2 * 3 + 1], p[index2 * 3 + 2]]),
                ];
                for v in vertices.iter() {
                    mesh.bounds.expand(v);
                    mesh.vertices.push(Vertex {
                        position: (*v).into(),
                    });
                    //debug!("{:?}", v);
                }

                let normals = if !n.is_empty() {
                    [
                        Normal {
                            normal: ([n[index0 * 3], n[index0 * 3 + 1], n[index0 * 3 + 2]]),
                        },
                        Normal {
                            normal: ([n[index1 * 3], n[index1 * 3 + 1], n[index1 * 3 + 2]]),
                        },
                        Normal {
                            normal: ([n[index2 * 3], n[index2 * 3 + 1], n[index2 * 3 + 2]]),
                        },
                    ]
                } else {
                    let n = normal(&Triangle {
                        vertices,
                        normal: Vector::new([0.0, 0.0, 0.0]),
                    });
                    [n, n, n]
                };
                for normal in normals.iter() {
                    mesh.normals.push(*normal);
                }
            }
        }
        Ok(mesh)
    }

    fn process_tri(&mut self, tri: &stl_io::Triangle, recalc_normals: bool) {
        for v in tri.vertices {
            self.bounds.expand(&v);
            self.vertices.push(Vertex { position: v.into() });
            //debug!("{:?}", v);
        }
        // Use normal from STL file if it is provided, otherwise calculate it ourselves
        let n: Normal;
        if recalc_normals || (tri.normal == stl_io::Vector::new([0.0, 0.0, 0.0])) {
            self.stl_had_normals = false;
            n = normal(tri);
        } else {
            n = Normal {
                normal: tri.normal.into(),
            };
        }
        //debug!("{:?}",tri.normal);
        // TODO: Figure out how to get away with 1 normal instead of 3
        for _ in 0..3 {
            self.normals.push(n);
        }
    }

    // Move the mesh to be centered at the origin
    // and scaled to fit a 2 x 2 x 2 box. This means that
    // all coordinates will be between -1.0 and 1.0
    pub fn scale_and_center(&self) -> cgmath::Matrix4<f32> {
        // Move center to origin
        let center = self.bounds.center();
        let translation_vector = cgmath::Vector3::new(-center.x, -center.y, -center.z);
        let translation_matrix = cgmath::Matrix4::from_translation(translation_vector);
        // Scale
        let longest = self
            .bounds
            .length()
            .max(self.bounds.width())
            .max(self.bounds.height());
        let scale = 2.0 / longest;
        info!("Scale:\t{}", scale);
        let scale_matrix = cgmath::Matrix4::from_scale(scale);
        scale_matrix * translation_matrix
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
fn normal(tri: &stl_io::Triangle) -> Normal {
    let p1 = cgmath::Vector3::new(tri.vertices[0][0], tri.vertices[0][1], tri.vertices[0][2]);
    let p2 = cgmath::Vector3::new(tri.vertices[1][0], tri.vertices[1][1], tri.vertices[1][2]);
    let p3 = cgmath::Vector3::new(tri.vertices[2][0], tri.vertices[2][1], tri.vertices[2][2]);
    let v = p2 - p1;
    let w = p3 - p1;
    let n = v.cross(w);
    let mag = n.x.abs() + n.y.abs() + n.z.abs();
    Normal {
        normal: [n.x / mag, n.y / mag, n.z / mag],
    }
}
