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
    position: [f32; 2],
    color: [f32; 4],
}
implement_vertex!(Vertex, position, color);

fn make_vertex<T>(color: &Color, pt: &Point<T>) -> Vertex
	where T: Axes<T>{
	let d = pt.axes.as_vec();
	let (x, y, z) = match d.len(){
		1 => (pt.time, d[0], 1.),
		2 => (d[0], d[1], 1.),
		3 => (d[0], d[1], 1.), //TODO: z should be d[1]/max z
		_ => panic!("Point can only be of type D1, D2, D3 which can only create a vertex of up to length three")
	};
	let v = Vertex{
		position: [(x) as f32, (y) as f32],
		color: [color.0, color.1, color.2, z]
	};
	// println!("{:?}", v);
	return v;
}

fn make_mat(x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32) -> [[f32; 4]; 4]{
	[
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, sz, 0.0],
        [ x , y, z, 1.0f32],
    ]
}

const VBO_SIZE: usize = 64;

pub trait DrawStyle<T> {
	fn push(&mut self, pt: &Point<T>, color: &Color, points:&RangedDeque<T>, display: &glium::Display);
	fn draw(&self, target: &mut glium::Frame);
    fn pick(&self);
}


pub struct DrawHidden{

}
impl <T> DrawStyle<T> for DrawHidden{
	fn push(&mut self, _pt: &Point<T>, _color: &Color, _points:&RangedDeque<T>, _display: &glium::Display){
		unimplemented!()
	}
	fn draw(&self, _target: &mut glium::Frame){
		unimplemented!()
	}
	fn pick(&self){
		unimplemented!()
	}
}


pub struct Scatter{
	vbos: VecDeque<VertexBuffer<Vertex>>,
	current_vbo_size: usize,
	program: glium::Program
}

impl Scatter {
    pub fn new(display: &glium::Display) -> Scatter{
    	Scatter{
    		vbos: VecDeque::new(),
    		current_vbo_size: 0,
    		program: glium::Program::from_source(display, 
    		r##"
			    #version 140

			    in vec2 position;
			    in vec4 color;
			    out vec4 attr_color;

			    uniform mat4 matrix;

			    void main() {
			    	attr_color = color;
			        gl_Position = matrix * vec4(position, 0.0, 1.0);
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

impl <T> DrawStyle<T> for Scatter
	where T: Axes<T> + Clone + std::ops::Index<usize>,
	<T as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{ 
	fn push(&mut self, pt: &Point<T>, color: &Color, points:&RangedDeque<T>, display: &glium::Display){
		if self.vbos.back().is_none() ||  (self.current_vbo_size == VBO_SIZE){
			match VertexBuffer::empty_dynamic(display, VBO_SIZE) {
				Ok(mut vbo) => {
							self.current_vbo_size = 0;
							if let Some(pt) = points.get_last(){
								vbo.as_mut_slice().slice(0..1).unwrap().write(&[make_vertex(color, pt)]);
								self.current_vbo_size += 1;
							}
							self.vbos.push_back(vbo);
							},
				Err(e) => println!("{:?}", e) //TODO: update signal health indicator, if out of vram start using ram?
			}
		}
		// println!("~.L@{:?}", self.vbos.len());
		let v = make_vertex(color, pt);
		if let Some(vbo) = self.vbos.back_mut(){
			vbo.as_mut_slice().slice(self.current_vbo_size..self.current_vbo_size+1).unwrap().write(&[v]);
			self.current_vbo_size += 1;
			// println!("~.V@{:?}", self.current_vbo_size);
		}
	}
	fn draw(&self, target: &mut glium::Frame){
		let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
		let uniforms = uniform! {
		    matrix: make_mat(0.,0.,0., 1.,1.,1.)
		};
		let mut c = 0;
		for i in self.vbos.iter(){
			//Is there a performance hit for slicing everything?
			let vb = i.slice(0..if c < self.vbos.len()-1 {VBO_SIZE}else{self.current_vbo_size}).unwrap();
			target.draw(vb, &indices, &self.program, &uniforms,
           		 &Default::default()).unwrap();
			c += 1;
		}
	}
	fn pick(&self){
		unimplemented!()
	}
}




//Program for lines style
   //  		program: glium::Program::from_source(display, 
   //  		r##"
			//     #version 140

			//     in vec2 position;
			//     in vec4 color;
			//     out vec4 attr_color;

			//     uniform mat4 matrix;

			//     void main() {
			//     	attr_color = color;
			//         gl_Position = matrix * vec4(position, 0.0, 1.0);
			//     }
			// "##,
   //  		r##"
   //  		    #version 140

   //  		    in vec4 attr_color;
   //  		    out vec4 color;

   //  		    void main() {
   //  		        color = attr_color;
   //  		    }
   //  		"##, 
   //  		None).unwrap()