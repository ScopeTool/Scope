extern crate glium;
extern crate color_set;
use std;
use std::f64::NAN;
// use std::mem::size_of;
// use std::marker::Sized;
use std::collections::{HashMap, VecDeque};

use self::color_set::{Color, Generator};

// mod drawstyles;
use drawstyles::*;


type Rect = (f64, f64, f64, f64);

static MIN_SCALE: f64 = 1e-12;


#[derive(Debug, Clone, Copy)]
pub enum SignalHealth {
    Good,
    InvalidFormat,
}


pub struct MsgPoint{
	pub name: String,
	pub line_number: usize,
	pub timestamp: f64,
	pub ty: PointType,
	pub x: f64, pub y: f64, pub z: f64
}


pub enum PointType {
	BreakPoint,
	D1,
	D2,
	D3,
}

#[derive(Debug, Clone, Copy)]
pub struct D1([f64;2]);
#[derive(Debug, Clone, Copy)]
pub struct D2([f64;3]);
#[derive(Debug, Clone, Copy)]
pub struct D3([f64;4]);
pub trait Axes<T>{ 
	fn size() -> usize;
	fn default() -> T;
	fn ones() -> T;
	fn as_vec(&self) -> Vec<f64>;
	fn into(point: MsgPoint) -> Point<T>;
	}
impl Axes<D1> for D1 {
	fn size() -> usize{2}
	fn default() -> D1 {D1{0:[NAN; 2]}} 
	fn ones() -> D1 {D1{0:[1.; 2]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn into(point: MsgPoint) -> Point<D1>{
		Point::new(D1{0:[point.timestamp, point.x]})
	}
}
impl Axes<D2> for D2 {
	fn size() -> usize{3}
	fn default() -> D2 {D2{0:[NAN; 3]}} 
	fn ones() -> D2 {D2{0:[1.; 3]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn into(point: MsgPoint) -> Point<D2>{
		Point::new(D2{0:[point.timestamp, point.x, point.y]})
	}
}
impl Axes<D3> for D3 {
	fn size() -> usize{4}
	fn default() -> D3 {D3{0:[NAN; 4]}} 
	fn ones() -> D3 {D3{0:[1.; 4]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn into(point: MsgPoint) -> Point<D3>{
		Point::new(D3{0:[point. timestamp, point.x, point.y, point.z]})
	}
}

impl std::ops::Index<usize> for D1 {
    type Output = f64;
    fn index<'a>(&'a self, idx: usize) -> &'a Self::Output{
        &(self.0)[idx]
    }
}
impl std::ops::Index<usize> for D2 {
    type Output = f64;
    fn index<'a>(&'a self, idx: usize) -> &'a Self::Output{
        &(self.0)[idx]
    }
}
impl std::ops::Index<usize> for D3 {
    type Output = f64;
    fn index<'a>(&'a self, idx: usize) -> &'a Self::Output{
        &(self.0)[idx]
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Point<A>{
    pub axes: A
}

impl <A> Point<A>{
	fn new(axes: A) -> Point<A>{
		Point{axes}
	}
}



#[derive(Debug, Clone)]
pub struct Range{
    pub min: Vec<f64>, //minimums on x y and z
    pub max: Vec<f64>
}

//This class supports constant time fifo buffer, and tracks minimums and maximum values
//In the future it may be posible to get minimums and maximums in constant time as well
//Using https://stackoverflow.com/questions/4802038/implement-a-queue-in-which-push-rear-pop-front-and-get-min-are-all-consta
#[derive(Debug)]
pub struct RangedDeque<A>{
    points: VecDeque<Point<A>>,
    range: Range
}

impl <A> RangedDeque<A> where
	A: Axes<A> + Clone + std::ops::Index<usize>,
	<A as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{
	fn new() -> RangedDeque<A>{
		RangedDeque{points: std::collections::VecDeque::new(),
	   				range: Range{min: A::default().as_vec(), max: A::default().as_vec()}}
	}
	fn push(&mut self, pt: Point<A>) -> bool{
		//TODO: recompupte range
		let mut range_update = false;
		for i in 0..A::size(){
			let t = pt.axes[i].clone().into();
			if self.range.min[i] > t || self.range.min[i].is_nan(){ 
				self.range.min[i] = t;
				range_update = true;
			} else if self.range.max[i] < t || self.range.max[i].is_nan(){
				self.range.max[i] = t;
				range_update = true;
			}
		}
		// println!("{:#?}", self.range);
		self.points.push_back(pt);
		return range_update;
	}
	pub fn get_last(&self) -> Option<&Point<A>>{
		self.points.back()
	}
	pub fn get(&self, idx: usize) -> &Point<A>{
		&self.points[idx]
	}
	#[allow(dead_code)]
	fn pop(&mut self){
		//TODO: Recompute range if popped val matches range
		self.points.pop_front();
	}
	pub fn get_range(&self) -> Range{
		self.range.clone()
	}
	pub fn iter(&self) -> std::collections::vec_deque::Iter<Point<A>>{
		self.points.iter()
	}
}

struct Signal<'a, A> {
    name: String,
    color: Color,
    unit_scale: Vec<f64>, //If axis values exceed that which can fit in f32, divide by these values and use these values for display
    points: RangedDeque<A>,
    style: Box<DrawStyle<A>>,
    health: SignalHealth,
    pick_thresh: f32,
    display: &'a glium::Display
}

impl <'a, T> Signal<'a, T>
	where T: Axes<T> + Clone + std::ops::Index<usize>,
	<T as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{
	fn new(name: String, style: Box<DrawStyle<T>>,  display: &'a glium::Display) -> Signal<'a,T>{
		Signal{	
				name: name.clone(), 
				color: Generator::get_color(name, 0.8, 1.),
				unit_scale: T::ones().as_vec(), 
				points: RangedDeque::new(), 
				style,
				health: SignalHealth::Good,
				pick_thresh: 0.1,
				display
			}
	}
	fn get_transform(&self, area: Rect) -> Transform{
		
		let range = self.style.get_range(&self.points); // this range in xy view space

		let xs = ((area.2-area.0)/(range.max[0]-range.min[0])).max(MIN_SCALE);

		let ys = ((area.3-area.1)/(range.max[1]-range.min[1])).max(MIN_SCALE);

		Transform{
			dx: (area.2-range.max[0]*xs) as f32, dy: ((-ys*(range.max[1]+range.min[1])/2.)+(area.3+area.1)/2.0) as f32,
			sx: xs as f32, sy: ys as f32, sz: 1.0
		}
	}
}

pub trait GenericSignal {
    fn draw(&self, target: &mut glium::Frame, area: Rect);
    fn add_point(&mut self, point: MsgPoint);
    fn get_color(&self)-> Color;
    fn get_health(&self) -> SignalHealth;
    fn pick(&self, mouse: (f32, f32), area: Rect)->Option<PickData>;
    fn get_point_string(&self, idx: usize) -> String;
}

impl <'a, T> GenericSignal for Signal<'a, T>
	where T: Axes<T> + Clone + std::ops::Index<usize> + std::fmt::Debug,
	<T as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{
	fn draw(&self, target: &mut glium::Frame, area: Rect){
		
		let trans = self.get_transform(area);

		self.style.draw(&trans, target);
	}
	fn add_point(&mut self, point: MsgPoint){
		let pt = T::into(point);
		// Do unit scaling here before pass to drawstyle
		self.points.push(pt.clone());
		self.style.push(&pt, &self.color, &self.points, self.display);
	}
	fn pick(&self, mouse: (f32, f32), area: Rect) -> Option<PickData>{
		self.style.pick( &self.points, mouse, self.get_transform(area), self.unit_scale.clone(), self.pick_thresh)
	}
	fn get_color(&self) -> Color{
		self.color
	}
	fn get_health(&self) -> SignalHealth{
		self.health.clone()
	}
	fn get_point_string(&self, idx: usize) -> String{
		//DrawStyle should provide this
		let mut rslt = String::from("");
		for i in self.points.get(idx).axes.as_vec(){
			rslt.push_str(&format!("{:.*}, ", 3, i));
		}
		rslt.pop();rslt.pop();
		rslt
	}
}


pub struct SignalManager<'a> {
	signals: HashMap<String, Box<GenericSignal+'a>>,
	display: &'a glium::Display,
	pub point_count: usize
}

impl <'a> SignalManager<'a>{
	pub fn new(display: &glium::Display) -> SignalManager {
	    SignalManager{
	    	// signals_d1: HashMap::new(), signals_d2: HashMap::new(), signals_d3: HashMap::new(),
	    	signals: HashMap::new(),
	    	display,
	    	point_count: 0
	    }
	}

	pub fn add_point(&mut self, point: MsgPoint){
		let name = point.name.clone();
		match self.signals.entry(name.clone()){
			std::collections::hash_map::Entry::Occupied(mut val) => {
				let mut ch = val.get_mut();
				ch.add_point(point);
			},
			std::collections::hash_map::Entry::Vacant(val) => {
				let mut ch: Box<GenericSignal+'a> = match point.ty {
					PointType::D1=> {
						let ds: Box<DrawStyle<D1>> = Box::new(Lines::new(self.display));
						Box::new(Signal::new(name.clone(), ds, self.display))
					},
					PointType::D2 => {
						let ds: Box<DrawStyle<D2>> = Box::new(Lines::new(self.display));
						Box::new(Signal::new(name.clone(), ds, self.display))
					},
					PointType::D3=> {
						let ds: Box<DrawStyle<D3>> = Box::new(Scatter::new(self.display));
						Box::new(Signal::new(name.clone(), ds, self.display))
					},
					PointType::BreakPoint => {
						let ds: Box<DrawStyle<D1>> = Box::new(Scatter::new(self.display));
						Box::new(Signal::new(name.clone(), ds, self.display))
					}
				};
				ch.add_point(point);
				val.insert(ch);
				println!("New Signal: {:?}", name)
			}
		}
		self.point_count+=1;
	}

	pub fn draw_signals(&self, target: &mut glium::Frame, area: Rect){
		for (_name, sig) in self.signals.iter(){
			sig.draw(target, area);
		}
	}

	pub fn iter(&self) -> std::collections::hash_map::Iter<String, Box<GenericSignal + 'a>>{
		self.signals.iter()
	}
}



pub struct PickData{
	pub index: usize,
	pub screen_pos: (f32, f32)
}