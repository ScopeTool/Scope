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

pub struct MsgPoint{
	pub name: String,
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
pub struct D1([f64;1]);
#[derive(Debug, Clone, Copy)]
pub struct D2([f64;2]);
#[derive(Debug, Clone, Copy)]
pub struct D3([f64;3]);
pub trait Axes<T>{ 
	fn default() -> T;
	fn ones() -> T;
	fn as_vec(&self) -> Vec<f64>;
	fn size() -> usize;
	fn into(point: MsgPoint) -> Point<T>;
	}
impl Axes<D1> for D1 {
	fn default() -> D1 {D1{0:[NAN; 1]}} 
	fn ones() -> D1 {D1{0:[1.; 1]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn size() -> usize{1}
	fn into(point: MsgPoint) -> Point<D1>{
		Point::new(point.timestamp, D1{0:[point.x]})
	}
}
impl Axes<D2> for D2 {
	fn default() -> D2 {D2{0:[NAN; 2]}} 
	fn ones() -> D2 {D2{0:[1.; 2]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn size() -> usize{2}
	fn into(point: MsgPoint) -> Point<D2>{
		Point::new(point.timestamp, D2{0:[point.x, point.y]})
	}
}
impl Axes<D3> for D3 {
	fn default() -> D3 {D3{0:[NAN; 3]}} 
	fn ones() -> D3 {D3{0:[1.; 3]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn size() -> usize{3}
	fn into(point: MsgPoint) -> Point<D3>{
		Point::new(point.timestamp, D3{0:[point.x, point.y, point.z]})
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


#[derive(Debug)]
pub struct Point<A>{
    pub time: f64,
    pub axes: A
}

impl <A> Point<A>{
	fn new(time: f64, axes: A) -> Point<A>{
		Point{time, axes}
	}
}



#[derive(Debug, Clone)]
struct Range{
    min: Vec<f64>, //minimums on x y and z
    max: Vec<f64>
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
	fn push(&mut self, pt: Point<A>){
		//TODO: recompupte range
		for i in 0..A::size(){
			self.range.min[i] = pt.axes[i].clone().into().min(self.range.min[i]);
			self.range.max[i] = pt.axes[i].clone().into().max(self.range.max[i]);
		}
		self.points.push_back(pt);
	}
	pub fn get_last(&self) -> Option<&Point<A>>{
		self.points.back()
	}
	#[allow(dead_code)]
	fn pop(&mut self){
		//TODO: Recompute range if popped val matches range
		self.points.pop_front();
	}
	#[allow(dead_code)]
	fn get_range(&self) -> Range{
		self.range.clone()
	}
}

struct Signal<'a, A> {
    name: String,
    color: Color,
    unit_scale: Vec<f64>, //If axis values exceed that which can fit in f32, divide by these values and use these values for display
    points: RangedDeque<A>,
    style: Box<DrawStyle<A>>,
    display: &'a glium::Display
}

impl <'a, T> Signal<'a, T>
	where T: Axes<T> + Clone + std::ops::Index<usize>,
	<T as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{
	fn new(name: String, style: Box<DrawStyle<T>>,  display: &'a glium::Display) -> Signal<'a,T>{
		Signal{	
				name: name.clone(), 
				color: Generator::get_color(name, 0.9, 0.9),
				unit_scale: T::ones().as_vec(), 
				points: RangedDeque::new(), 
				style,
				display
			}
	}
	fn push(&mut self, pt: Point<T>){
		// Do unit scaling here before pass to drawstyle
		self.style.push(&pt, &self.color, &self.points, self.display); // This order is critical bc the way chunks are linked in push_vbos
		self.points.push(pt);
	}
	fn _draw(&self, target: &mut glium::Frame){
		self.style.draw(target);
	}
}

trait GenericSignal {
    fn draw(&self, target: &mut glium::Frame);
    fn add_point(&mut self, point: MsgPoint);
}

impl <'a, T> GenericSignal for Signal<'a, T>
	where T: Axes<T> + Clone + std::ops::Index<usize>,
	<T as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{
	fn draw(&self, target: &mut glium::Frame){
		self._draw(target);
	}
	fn add_point(&mut self, point: MsgPoint){
		self.push(T::into(point));
	}
}


pub struct SignalManager<'a> {
	// signals_d1: HashMap<String, Signal<'a, D1>>,
	// signals_d2: HashMap<String, Signal<'a, D2>>,
	// signals_d3: HashMap<String, Signal<'a, D3>>,

	signals: HashMap<String, Box<GenericSignal+'a>>,

	display: &'a glium::Display
}

impl <'a> SignalManager<'a>{
	pub fn new(display: &glium::Display) -> SignalManager {
	    SignalManager{
	    	// signals_d1: HashMap::new(), signals_d2: HashMap::new(), signals_d3: HashMap::new(),
	    	signals: HashMap::new(),
	    	display,
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
						let ds: Box<DrawStyle<D1>> = Box::new(Scatter::new(self.display));
						Box::new(Signal::new(name.clone(), ds, self.display))
					},
					PointType::D2 => {
						let ds: Box<DrawStyle<D2>> = Box::new(Scatter::new(self.display));
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
	}

	// pub fn add_point(&mut self, point: MsgPoint){
	// 	// match point {
	// 	//     MsgPoint::D1(name, time, x) => {SignalManager::_add_point(self.display, &mut self.signals_d1, name, Point::new(time, [x]))},
	// 	//     MsgPoint::D2(name, time, x, y) => {SignalManager::_add_point(self.display, &mut self.signals_d2, name, Point::new(time, [x, y]))},
	// 	//     MsgPoint::D3(name, time, x, y, z) => {SignalManager::_add_point(self.display, &mut self.signals_d3, name, Point::new(time, [x, y, z]))},
	// 	//     MsgPoint::BreakPoint(ref _name, _time) => {}//{SignalManager::_add_point(&mut self.signalsD3, name, Point::new(time, [x, y, z]))},
	// 	// }
	// }

	// pub fn _add_point(display: &'a glium::Display, name: String, point: Point<T>){
	// }

	// // fn _add_point<T: Axes<T> + Clone>(display: &'a glium::Display, hm: &mut HashMap<String, Signal<'a, T>>, name: String, point: Point<T>){
	// // 	// match hm.entry(name.clone()){
	// // 	// 	std::collections::hash_map::Entry::Occupied(mut val) => {
	// // 	// 		let mut ch = val.get_mut();
	// // 	// 		ch.push(point);
	// // 	// 	},
	// // 	// 	std::collections::hash_map::Entry::Vacant(val) => {
	// // 	// 		let mut ch = Signal::new(name.clone(), Box::new(Scatter::new(display)), display);
	// // 	// 		println!("New Signal: {:?}", name);
	// // 	// 		ch.push(point);
	// // 	// 		val.insert(ch);
	// // 	// 	}
	// // 	// }
	// // }

	pub fn draw_signals(&self, target: &mut glium::Frame){
		// for (_name, sig) in self.signals_d1.iter(){sig.draw(target);}
		// for (_name, sig) in self.signals_d2.iter(){sig.draw(target);}
		// for (_name, sig) in self.signals_d3.iter(){sig.draw(target);}
		for (_name, sig) in self.signals.iter(){
			sig.draw(target);
		}
	}
}

		