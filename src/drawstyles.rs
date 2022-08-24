extern crate color_set;
extern crate glium;

use self::color_set::Color;
use super::signal::{Axes, PickData, Point, Range, RangedDeque};
use glium::Surface;
use glium::VertexBuffer;
use std::collections::VecDeque;

#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

fn make_vertex<T>(color: &Color, pt: &Point<T>) -> Vertex
where
    T: Axes<T>,
{
    let x = pt.axes[T::x()].clone().into();
    let y = pt.axes[T::y()].clone().into();
    let z = if T::z() >= 0 {
        pt.axes[T::z() as usize].clone().into()
    } else {
        1.
    }; //TODO: z should be d[1]/max z
    let v = Vertex {
        position: [(x) as f32, (y) as f32, z as f32],
        color: [color.0, color.1, color.2],
    };
    return v;
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub dx: f32,
    pub dy: f32,
    pub sx: f32,
    pub sy: f32,
    pub sz: f32,
}

impl<'a> From<&'a Transform> for [[f32; 4]; 4] {
    fn from(tf: &Transform) -> Self {
        [
            [tf.sx, 0.0, 0.0, 0.0],
            [0.0, tf.sy, 0.0, 0.0],
            [0.0, 0.0, tf.sz, 0.0],
            [tf.dx, tf.dy, 0.0, 1.0f32],
        ]
    }
}

struct VBOChunks {
    vbos: VecDeque<VertexBuffer<Vertex>>,
    current_vbo_size: usize,
    connected: bool,
}
const VBO_SIZE: usize = 256;
impl VBOChunks {
    fn new(connected: bool) -> VBOChunks {
        VBOChunks {
            vbos: VecDeque::new(),
            current_vbo_size: 0,
            connected,
        }
    }
    fn push<T>(&mut self, v: Vertex, display: &glium::Display)
    where
        T: Axes<T>,
    {
        if (self.vbos.back().is_none()) || (self.current_vbo_size == VBO_SIZE) {
            match VertexBuffer::empty_dynamic(display, VBO_SIZE) {
                Ok(mut vbo) => {
                    self.current_vbo_size = 0;
                    if self.connected {
                        //map back vertex from last buffer
                        if let Some(t) = self.vbos.back() {
                            let last_pt = t.slice(VBO_SIZE - 1..VBO_SIZE).unwrap().read().unwrap();
                            vbo.as_mut_slice().slice(0..1).unwrap().write(&last_pt);
                            self.current_vbo_size += 1;
                        }
                    }
                    self.vbos.push_back(vbo);
                }
                Err(e) => println!("{:?}", e), //TODO: update signal health indicator, if out of vram start using ram?
            }
        }
        // println!("~.L@{:?}", self.vbos.len());
        // dbg!(self.current_vbo_size);
        if let Some(vbo) = self.vbos.back_mut() {
            vbo.as_mut_slice()
                .slice(self.current_vbo_size..self.current_vbo_size + 1)
                .unwrap()
                .write(&[v]);
            self.current_vbo_size += 1;
            // println!("~.V@{:?}", self.current_vbo_size);
        }
    }
    fn draw<F>(&self, mut drawer: F)
    where
        F: FnMut(glium::vertex::VertexBufferSlice<Vertex>) -> (),
    {
        let mut c = 0;
        for i in self.vbos.iter() {
            //Is there a performance hit for slicing everything?
            let vb = i
                .slice(
                    0..if c < self.vbos.len() - 1 {
                        VBO_SIZE
                    } else {
                        self.current_vbo_size
                    },
                )
                .unwrap();
            drawer(vb);
            c += 1;
        }
    }
}

//transform applied in shader, point x and y, unit scale x and y. allows draw style to select what point values are used for x and y (x might be time)
fn point_pos(trans: Transform, x: f64, y: f64, us_x: f64, us_y: f64) -> (f32, f32) {
    let x2 = (x * us_x) as f32; // TODO: these need to work according to unit scale implementation, find a nice way for signal to handle this
    let y2 = (y * us_y) as f32;
    (x2 * trans.sx + trans.dx, y2 * trans.sy + trans.dy)
}

fn find_min<T, F>(points: &RangedDeque<T>, cmp: F) -> (Option<usize>, f32)
where
    T: Axes<T> + Clone,
    F: Fn(&Point<T>) -> f32,
{
    let mut min_val = 2.0;
    let mut min_idx = None;
    for (i, pt) in points.iter().enumerate() {
        let d = cmp(pt);
        if d < min_val {
            min_val = d;
            min_idx = Some(i)
        }
    }
    return (min_idx, min_val);
}

// fn find_min_ord(){
// 	unimplemented!();
// }

fn get_std_pt_strs<T>(pt: &Point<T>) -> (String, String, String)
where
    T: Axes<T> + Clone,
{
    //DrawStyle should provide this
    let ts = pt.axes[T::timestamp()].clone().into();
    let x = pt.axes[T::x()].clone().into();
    let y = pt.axes[T::y()].clone().into();
    let mut z = String::new();
    if T::x() != T::timestamp() {
        z.push_str(&format!("{:.*} ns, ", 3, ts))
    }
    if T::z() >= 0 {
        z.push_str(&format!("{:.*}", 3, T::z() as usize));
    } else {
        z.pop();
        z.pop(); // remove trailing comma and space from timestamp
    }
    (format!("{:.*}", 3, x), format!("{:.*}", 3, y), z)
}

// List of Drawstyles defined in this file, used to ensure that all styles are provided as options from command line
#[derive(Debug)]
pub enum Styles {
    Scatter,
    Lines,
}

pub trait DrawStyle<T>
where
    T: Axes<T> + Clone,
{
    fn push(
        &mut self,
        pt: &Point<T>,
        color: &Color,
        points: &RangedDeque<T>,
        display: &glium::Display,
    );
    fn draw(&self, trans: &Transform, target: &mut glium::Frame);
    fn pick(
        &self,
        points: &RangedDeque<T>,
        mouse: (f32, f32),
        trans: Transform,
        unit_scale: Vec<f64>,
        pick_thresh: f32,
    ) -> Option<PickData> {
        let mut t = trans.clone();
        t.dx -= mouse.0;
        t.dy -= mouse.1;
        let ux = unit_scale[T::x()];
        let uy = unit_scale[T::y()];
        let d = find_min(points, move |pt| {
            let (x, y) = point_pos(
                t.clone(),
                pt.axes[T::x()].clone().into(),
                pt.axes[T::y()].clone().into(),
                ux,
                uy,
            );
            x.abs() + y.abs()
        });
        if let Some(idx) = d.0 {
            if pick_thresh >= d.1 {
                let pt = points.get(idx);
                return Some(PickData {
                    index: idx,
                    screen_pos: point_pos(
                        trans,
                        pt.axes[T::x()].clone().into(),
                        pt.axes[T::y()].clone().into(),
                        ux,
                        uy,
                    ),
                });
            }
        }
        return None;
    }
    fn get_range(&self, points: &RangedDeque<T>) -> Range {
        let r = points.get_range();
        return Range {
            min: vec![r.min[T::x()], r.min[T::y()]],
            max: vec![r.max[T::x()], r.max[T::y()]],
        };
    }
    fn get_point_strs(&self, pt: &Point<T>) -> (String, String, String) {
        get_std_pt_strs(pt)
    }
}

pub struct Scatter {
    vbos: VBOChunks,
    program: glium::Program,
}

impl Scatter {
    pub fn new(display: &glium::Display) -> Scatter {
        let source = glium::program::ProgramCreationInput::SourceCode {
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            outputs_srgb: false,
            uses_point_size: true,

            vertex_shader: r##"
			    #version 140

			    in vec3 position;
			    in vec3 color;
			    out vec3 attr_color;

			    uniform mat4 matrix;

			    void main() {
			    	attr_color = color;
			        gl_PointSize = max(position.z*matrix[2][2], 4);
			        gl_Position = matrix * vec4(position.xy, 0.0, 1.0);
			    }
				"##,
            fragment_shader: r##"
    		    #version 140

    		    in vec3 attr_color;
    		    out vec4 color;

    		    void main() {
    		        color = vec4(attr_color, 1.0);
    		    }
	    		"##,

            transform_feedback_varyings: None,
        };
        Scatter {
            vbos: VBOChunks::new(false),
            program: glium::Program::new(display, source).unwrap(),
        }
    }
}

impl<T> DrawStyle<T> for Scatter
where
    T: Axes<T> + Clone,
{
    fn push(
        &mut self,
        pt: &Point<T>,
        color: &Color,
        _points: &RangedDeque<T>,
        display: &glium::Display,
    ) {
        self.vbos.push::<T>(make_vertex::<T>(color, pt), display);
    }
    fn draw(&self, trans: &Transform, target: &mut glium::Frame) {
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
        let t: [[f32; 4]; 4] = trans.into();
        let uniforms = uniform! {
            matrix: t
        };

        self.vbos.draw(move |vb| {
            target
                .draw(vb, &indices, &self.program, &uniforms, &Default::default())
                .unwrap()
        });
    }
}

pub struct Lines {
    vbos: VBOChunks,
    program: glium::Program,
}

impl Lines {
    pub fn new(display: &glium::Display) -> Lines {
        Lines {
            vbos: VBOChunks::new(true),
            program: glium::Program::from_source(
                display,
                r##"
			    #version 140

			    in vec3 position;
			    in vec3 color;
			    out vec4 attr_color;

			    uniform mat4 matrix;

			    void main() {
			    	attr_color = vec4(color, 1.0);
			        gl_Position = matrix * vec4(position.xy, 0.0, 1.0);
			    }
			"##,
                r##"
    		    #version 140

    		    in vec4 attr_color;
    		    out vec4 color;

    		    void main() {
    		        color = attr_color;
    		    }
    		"##,
                None,
            )
            .unwrap(),
        }
    }
}

impl<T> DrawStyle<T> for Lines
where
    T: Axes<T> + Clone,
{
    fn push(
        &mut self,
        pt: &Point<T>,
        color: &Color,
        _points: &RangedDeque<T>,
        display: &glium::Display,
    ) {
        self.vbos.push::<T>(make_vertex::<T>(color, pt), display);
    }
    fn draw(&self, trans: &Transform, target: &mut glium::Frame) {
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
        let t: [[f32; 4]; 4] = trans.into();
        let uniforms = uniform! {
            matrix: t
        };

        self.vbos.draw(move |vb| {
            target
                .draw(vb, &indices, &self.program, &uniforms, &Default::default())
                .unwrap()
        });
    }
}
