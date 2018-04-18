extern crate glium;
extern crate color_set;
use std;
use glium::VertexBuffer;
use glium::Surface;
use std::collections::VecDeque;
use super::signal::{Axes, Point, RangedDeque};
use self::color_set::Color;

#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

fn make_vertex<T>(color: &Color, pt: &Point<T>) -> Vertex
	where T: Axes<T>{
	let d = pt.axes.as_vec();
	let (x, y, z) = match d.len(){
		1 => (pt.time, d[0], 1.),
		2 => (d[0], d[1], 1.),
		3 => (d[0], d[1], d[2]), //TODO: z should be d[1]/max z
		_ => panic!("Point can only be of type D1, D2, D3 which can only create a vertex of up to length three")
	};
	let v = Vertex{
		position: [(x) as f32, (y) as f32, z as f32],
		color: [color.0, color.1, color.2]
	};
	return v;
}

#[derive(Debug)]
pub struct Transform {
    pub dx: f32, pub dy: f32,
    pub sx: f32, pub sy: f32, pub sz: f32
}

impl From<Transform> for [[f32; 4]; 4] {
    fn from(tf: Transform) -> Self {
        [
        [tf.sx, 0.0, 0.0, 0.0],
        [0.0, tf.sy, 0.0, 0.0],
        [0.0, 0.0, tf.sz, 0.0],
        [tf.dx , tf.dy, 0.0, 1.0f32],
	    ]
    }
}

struct VBOChunks{
	vbos: VecDeque<VertexBuffer<Vertex>>,
	current_vbo_size: usize,
	connected: bool
}
const VBO_SIZE: usize = 64;
impl VBOChunks{
	fn new(connected: bool) -> VBOChunks{
		VBOChunks{vbos: VecDeque::new(), current_vbo_size: 0, connected}
	}
	fn push<T>(&mut self, v: Vertex, display: &glium::Display)
			where T: Axes<T>{
		if self.vbos.back().is_none() ||  (self.current_vbo_size == VBO_SIZE){
			match VertexBuffer::empty_dynamic(display, VBO_SIZE) {
				Ok(mut vbo) => {
							self.current_vbo_size = 0;
							if self.connected{
								//map back vertex from last buffer
								if let Some(t) = self.vbos.back(){
									let last_pt = t.slice(VBO_SIZE-1..VBO_SIZE).unwrap()
												.read().unwrap();
									vbo.as_mut_slice().slice(0..1).unwrap().write(&last_pt);
									self.current_vbo_size += 1;
								}
							}
							self.vbos.push_back(vbo);
							},
				Err(e) => println!("{:?}", e) //TODO: update signal health indicator, if out of vram start using ram?
			}
		}
		// println!("~.L@{:?}", self.vbos.len());
		if let Some(vbo) = self.vbos.back_mut(){
			vbo.as_mut_slice().slice(self.current_vbo_size..self.current_vbo_size+1).unwrap().write(&[v]);
			self.current_vbo_size += 1;
			// println!("~.V@{:?}", self.current_vbo_size);
		}
	}
	fn draw<F>(&self, mut drawer: F)
		where F: FnMut(glium::vertex::VertexBufferSlice<Vertex>)->(){
		let mut c = 0;
		for i in self.vbos.iter(){
			//Is there a performance hit for slicing everything?
			let vb = i.slice(0..if c < self.vbos.len()-1 {VBO_SIZE}else{self.current_vbo_size}).unwrap();
			drawer(vb);
			c += 1;
		}
	}
}


pub trait DrawStyle<T> {
	fn push(&mut self, pt: &Point<T>, color: &Color, points:&RangedDeque<T>, display: &glium::Display);
	fn draw(&self, trans: &Transform, target: &mut glium::Frame);
    fn pick(&self);
}


pub struct DrawHidden{

}
impl <T> DrawStyle<T> for DrawHidden{
	fn push(&mut self, _pt: &Point<T>, _color: &Color, _points:&RangedDeque<T>, _display: &glium::Display){
		unimplemented!()
	}
	fn draw(&self, _trans: &Transform, _target: &mut glium::Frame){
		unimplemented!()
	}
	fn pick(&self){
		unimplemented!()
	}
}


pub struct Scatter{
	vbos: VBOChunks,
	program: glium::Program
}

impl Scatter {
    pub fn new(display: &glium::Display) -> Scatter{
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

    	        transform_feedback_varyings: None
    	    };
    	Scatter{
    		vbos: VBOChunks::new(false),
    		program: glium::Program::new(display, source).unwrap()
    	}
    }
}

impl <T> DrawStyle<T> for Scatter
	where T: Axes<T> + Clone + std::ops::Index<usize>,
	<T as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{ 
	fn push(&mut self, pt: &Point<T>, color: &Color, _points:&RangedDeque<T>, display: &glium::Display){
		self.vbos.push::<T>(make_vertex::<T>(color, pt), display);
	}
	fn draw(&self, trans: &Transform, target: &mut glium::Frame){
		let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
		let t: [[f32; 4]; 4] = (*trans).into();
		let uniforms = uniform! {
		    matrix: t
		};

		self.vbos.draw(move |vb|{
			target.draw(vb, &indices, &self.program, &uniforms, &Default::default()).unwrap()
		});
	}
	fn pick(&self){
		unimplemented!()
	}
}

pub struct Lines{
	vbos: VBOChunks,
	program: glium::Program
}

impl Lines {
    pub fn new(display: &glium::Display) -> Lines{
    	Lines{
    		vbos: VBOChunks::new(true),
    		program: glium::Program::from_source(display, 
    		r##"
			    #version 140

			    in vec3 position;
			    in vec3 color;
			    out vec4 attr_color;

			    uniform mat4 matrix;

			    void main() {
			    	attr_color = color;
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
    		None).unwrap()
    	}
    }
}

impl <T> DrawStyle<T> for Lines
	where T: Axes<T> + Clone + std::ops::Index<usize>,
	<T as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{ 
	fn push(&mut self, pt: &Point<T>, color: &Color, _points:&RangedDeque<T>, display: &glium::Display){
		self.vbos.push::<T>(make_vertex::<T>(color, pt), display);
	}
	fn draw(&self, trans: &Transform, target: &mut glium::Frame){
		let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
		let t: [[f32; 4]; 4] = (*trans).into();
		let uniforms = uniform! {
		    matrix: t
		};

		self.vbos.draw(move |vb|{
			target.draw(vb, &indices, &self.program, &uniforms, &Default::default()).unwrap()
		});
	}
	fn pick(&self){
		unimplemented!()
	}
}
