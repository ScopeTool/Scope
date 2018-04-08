extern crate glium;
extern crate color_set;
use std;
use std::f64::NAN;
// use std::mem::size_of;
use glium::VertexBuffer;
// use std::marker::Sized;
use std::collections::{HashMap, VecDeque};

use glium::Surface;

use self::color_set::{Color, Generator};

pub enum MsgPoint {
	BreakPoint(String, f64),
	D1(String, f64, f64),
	D2(String, f64, f64, f64),
	D3(String, f64, f64, f64, f64),
}

#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

implement_vertex!(Vertex, position, color);

const VERTEX_SHADER_SRC: &str = r##"
    #version 140

    in vec2 position;
    in vec4 color;
    out vec4 attr_color;

    uniform mat4 matrix;

    void main() {
    	attr_color = color;
        gl_Position = matrix * vec4(position, 0.0, 1.0);
    }
"##;

const FRAGMENT_SHADER_SRC: &str = r##"
    #version 140

    in vec4 attr_color;
    out vec4 color;

    void main() {
        color = attr_color;
    }
"##;

type D1 = [f64; 1];
type D2 = [f64; 2];
type D3 = [f64; 3];
trait Axes<T>{ 
	fn default() -> T;
	fn ones() -> T;
	fn as_vec(&self) -> Vec<f64>;
	}
impl Axes<D1> for D1 {
	fn default() -> D1 {[NAN; 1]} 
	fn ones() -> D1 {[1.; 1]}
	fn as_vec(&self) -> Vec<f64>{self.to_vec()}
}
impl Axes<D2> for D2 {
	fn default() -> D2 {[NAN; 2]} 
	fn ones() -> D2 {[1.; 2]}
	fn as_vec(&self) -> Vec<f64>{self.to_vec()}
}
impl Axes<D3> for D3 {
	fn default() -> D3 {[NAN; 3]} 
	fn ones() -> D3 {[1.; 3]}
	fn as_vec(&self) -> Vec<f64>{self.to_vec()}
}


#[derive(Debug)]
struct Point<A>{
    time: f64,
    axes: A
}

impl <A> Point<A>{
	fn new(time: f64, axes: A) -> Point<A>{
		Point{time, axes}
	}
}



#[derive(Debug, Copy, Clone)]
struct Range<A>{
    min: A, //minimums on x y and z
    max: A
}

//This class supports constant time fifo buffer, and tracks minimums and maximum values
//In the future it may be posible to get minimums and maximums in constant time as well
//Using https://stackoverflow.com/questions/4802038/implement-a-queue-in-which-push-rear-pop-front-and-get-min-are-all-consta
#[derive(Debug)]
struct RangedDeque<A>{
    points: VecDeque<Point<A>>,
    range: Range<A>
}

impl <A> RangedDeque<A> where
	A: Axes<A> + Clone{
	fn new() -> RangedDeque<A>{
		RangedDeque{points: std::collections::VecDeque::new(),
	   				range: Range{min: A::default(), max: A::default()}}
	}
	fn push(&mut self, pt: Point<A>){
		//TODO: recompupte range
		self.points.push_back(pt);
	}
	fn get_last(&self) -> Option<&Point<A>>{
		self.points.back()
	}
	#[allow(dead_code)]
	fn pop(&mut self){
		//TODO: Recompute range if popped val mathces range
		self.points.pop_front();
	}
	#[allow(dead_code)]
	fn get_range(&self) -> Range<A>{
		self.range.clone()
	}
}

struct Signal<'a, A> {
    name: String,
    color: Color,
    unit_scale: Vec<f64>, //If axis values exceed that which can fit in f32, divide by these values and use these values for display
    points: RangedDeque<A>,
    vbos: VecDeque<VertexBuffer<Vertex>>,
    current_vbo_size: usize,
    display: &'a glium::Display
}

impl <'a, T> Signal<'a, T>
	where T: Axes<T> + Clone{
	const VBO_SIZE: usize = 64usize;
	fn new(name: String, display: &'a glium::Display) -> Signal<'a,T>{
		Signal{	
				name: name.clone(), 
				color: Generator::get_color(name, 0.9, 0.9),
				unit_scale: T::ones().as_vec(), 
				points: RangedDeque::new(), 
				vbos: VecDeque::new(),
				current_vbo_size: 0,
				display}
	}
	fn push(&mut self, pt: Point<T>){
		self.push_vbos(&pt); // This order is critical bc the way chunks are linked in push_vbos
		self.points.push(pt);
	}
	fn push_vbos(&mut self, pt: &Point<T>){
		if self.vbos.back().is_none() ||  (self.current_vbo_size == <Signal<'a, T>>::VBO_SIZE){
			match VertexBuffer::empty_dynamic(self.display, <Signal<'a, T>>::VBO_SIZE) {
				Ok(mut vbo) => {
							self.current_vbo_size = 0;
							if let Some(pt) = self.points.get_last(){
								vbo.as_mut_slice().slice(0..1).unwrap().write(&[self.make_vertex(pt)]);
								self.current_vbo_size += 1;
							}
							self.vbos.push_back(vbo);
							},
				Err(e) => println!("{:?}", e) //TODO: update signal health indicator, if out of vram start using ram?
			}
		}
		// println!("~.L@{:?}", self.vbos.len());
		let v = self.make_vertex(pt);
		if let Some(vbo) = self.vbos.back_mut(){
			vbo.as_mut_slice().slice(self.current_vbo_size..self.current_vbo_size+1).unwrap().write(&[v]);
			self.current_vbo_size += 1;
			// println!("~.V@{:?}", self.current_vbo_size);
		}
	}
	fn make_vertex(&self, pt: &Point<T>) -> Vertex{
		let d = pt.axes.as_vec();
		let (x, y, z) = match d.len(){
			1 => (pt.time, d[0], 1.),
			2 => (d[0], d[1], 1.),
			3 => (d[0], d[1], 1.), //TODO: z should be d[1]/max z
			_ => panic!("Point can only be of type D1, D2, D3 which can only create a vertex of up to length three")
		};
		let v = Vertex{
			position: [(x/self.unit_scale[0]) as f32, (y/self.unit_scale[1]) as f32],
			color: [self.color.0, self.color.1, self.color.2, z]
		};
		// println!("{:?}", v);
		return v;
	}
	fn draw(&self, target: &mut glium::Frame, program: &glium::Program){
		let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
		let mat = [
		        [1., 0.0, 0.0, 0.0],
		        [0.0, 1., 0.0, 0.0],
		        [0.0, 0.0, 1., 0.0],
		        [ 0.0 , 0.0, 0.0, 1.0f32],
		    ];
		let uniforms = uniform! {
		    matrix: mat
		};
		let mut c = 0;
		for i in self.vbos.iter(){
			//Is there a performance hit for slicing everything?
			let vb = i.slice(0..if c < self.vbos.len()-1 {<Signal<'a, T>>::VBO_SIZE}else{self.current_vbo_size}).unwrap();
			target.draw(vb, &indices, &program, &uniforms,
           		 &Default::default()).unwrap();
			c += 1;
		}
	}
}


pub struct SignalManager<'a> {
	signals_d1: HashMap<String, Signal<'a, D1>>,
	signals_d2: HashMap<String, Signal<'a, D2>>,
	signals_d3: HashMap<String, Signal<'a, D3>>,

	display: &'a glium::Display,

	simple_lines: glium::Program
}

impl <'a> SignalManager<'a>{
	pub fn new(display: &glium::Display) -> SignalManager {
	    SignalManager{
	    	signals_d1: HashMap::new(), signals_d2: HashMap::new(), signals_d3: HashMap::new(),
	    	display,
	    	simple_lines: glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap()
	    }
	}

	pub fn add_point(&mut self, point: MsgPoint){
		match point {
		    MsgPoint::D1(name, time, x) => {SignalManager::_add_point(self.display, &mut self.signals_d1, name, Point::new(time, [x]))},
		    MsgPoint::D2(name, time, x, y) => {SignalManager::_add_point(self.display, &mut self.signals_d2, name, Point::new(time, [x, y]))},
		    MsgPoint::D3(name, time, x, y, z) => {SignalManager::_add_point(self.display, &mut self.signals_d3, name, Point::new(time, [x, y, z]))},
		    MsgPoint::BreakPoint(ref _name, _time) => {}//{SignalManager::_add_point(&mut self.signalsD3, name, Point::new(time, [x, y, z]))},
		}
	}

	fn _add_point<T: Axes<T> + Clone>(display: &'a glium::Display, hm: &mut HashMap<String, Signal<'a, T>>, name: String, point: Point<T>){
		match hm.entry(name.clone()){
			std::collections::hash_map::Entry::Occupied(mut val) => {
				let mut ch = val.get_mut();
				ch.push(point);
			},
			std::collections::hash_map::Entry::Vacant(val) => {
				let mut ch = Signal::new(name.clone(), display);
				println!("New Signal: {:?}", name);
				ch.push(point);
				val.insert(ch);
			}
		}
	}

	pub fn draw_signals(&self, target: &mut glium::Frame){
		for (_name, sig) in self.signals_d1.iter(){sig.draw(target, &self.simple_lines);}
		for (_name, sig) in self.signals_d2.iter(){sig.draw(target, &self.simple_lines);}
		for (_name, sig) in self.signals_d3.iter(){sig.draw(target, &self.simple_lines);}
	}
}

		