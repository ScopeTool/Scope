extern crate glium;
extern crate color_set;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
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



#[derive(Debug,  Clone)]
pub struct Range{
    pub min: Vec<f64>, //minimums on x y and z
    pub max: Vec<f64>
}
impl Range{
	fn new() -> Range{
		Range{min: Vec::new(), max: Vec::new()}
	}
	fn expandby(&mut self, other: &Range){
		let mut new_min = Vec::new();//TODO: dont do these tiny heap allocations every frame
		for i in 0..self.min.len().max(other.min.len()){
			let mut a = if i < other.min.len() {other.min[i]} else {NAN};
			let mut b = if i < self.min.len() {self.min[i]} else {NAN};
			new_min.push(a.min(b));
		}
		let mut new_max = Vec::new();
		for i in 0..self.max.len().max(other.max.len()){
			let mut a = if i < other.max.len() {other.max[i]} else {NAN};
			let mut b = if i < self.max.len() {self.max[i]} else {NAN};
			new_max.push(a.max(b));
		}
		self.min = new_min;
		self.max = new_max;
	}
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
		// println!("{:?}", (pt.axes[0].clone().into(), pt.axes[1].clone().into()));
		let mut range_update = false;
		for i in 0..A::size(){
			let t = pt.axes[i].clone().into();
			if self.range.min[i] > t || self.range.min[i].is_nan(){ 
				self.range.min[i] = t;
				range_update = true;
			}
			if self.range.max[i] < t || self.range.max[i].is_nan(){
				self.range.max[i] = t;
				range_update = true;
			}
		}
		// println!("{:?}", self.range);
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
	pub fn len(&self) -> usize{
		self.points.len()
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


#[derive(Debug,  Clone)]
struct ViewData{
	pos: (f32, f32),
    zoom: f64,
    range: Range,
    maintain_aspect: bool,
    mode: u8,
    id: String
}
impl ViewData{
	fn clear_range(&mut self){
	    self.range.min = vec![NAN; self.range.min.len()];
		self.range.max = vec![NAN; self.range.max.len()];
	}
}

#[derive(Debug, Clone)]
pub struct View {
    data: Rc<RefCell<ViewData>>,
}
impl View {
    fn new(name: String) -> View{
    	let data = Rc::new(RefCell::new(ViewData{pos: (0.,0.), 
    		zoom: 1., 
    		range: Range::new(), 
    		maintain_aspect: false, 
    		mode: 0,
    		id: name
    	}));
    	View{data}
    }

    fn share(&self, range: &Range){
    	let mut data = self.data.borrow_mut();
    	data.range.expandby(range);
    }

    fn get_transform(&self, area: Rect, range: &Range) -> Transform{
    	let data = self.data.borrow();

    	let mut working_range = if 0b10 & data.mode != 0{&data.range} else {range};
    	let xmin = working_range.min[0];
    	let xmax = working_range.max[0];
    	working_range = if 0b100 & data.mode != 0{&data.range} else {range};
    	let ymin = working_range.min[1];
    	let ymax = working_range.max[1];

    	// println!("Working Range: x: ({:?}, {:?}), y: ({:?}, {:?})", xmin, xmax, ymin, ymax);

    	let mut xs = ((area.2-area.0)/(xmax-xmin)).max(MIN_SCALE);
		let mut ys = ((area.3-area.1)/(ymax-ymin)).max(MIN_SCALE); 

		if data.maintain_aspect{
			let val = if (1.-xs).abs() > (1.-ys).abs() {xs} else {ys};
			xs = val; ys = val;
		}

		xs *= data.zoom;
		ys *= data.zoom;

		Transform{
			dx: (area.2-xmax*xs) as f32, dy: ((-ys*(ymax+ymin)/2.)+(area.3+area.1)/2.0) as f32,
			sx: xs as f32, sy: ys as f32, sz: 1.0
		}
	}

	fn zoom(&self, by: f64){
		let mut data = self.data.borrow_mut();
		data.zoom += by/10.;
		data.zoom = data.zoom.max(MIN_SCALE)
	}

    fn set_bind_mode(&self, mode: u8){
    	self.data.borrow_mut().mode = mode;
    }
}

struct Signal<'a, A> {
    name: String,
    color: Color,
    unit_scale: Vec<f64>, //If axis values exceed that which can fit in f32, divide by these values and use these values for display
    points: RangedDeque<A>,
    style: Box<DrawStyle<A>>,
    health: SignalHealth,
    view: View,
    pick_thresh: f32,
    display: &'a glium::Display
}

impl <'a, T> Signal<'a, T>
	where T: Axes<T> + Clone + std::ops::Index<usize>,
	<T as std::ops::Index<usize>>::Output: std::marker::Sized+std::convert::Into<f64>+Clone{
	fn new(name: String, style: Box<DrawStyle<T>>,  view: View, display: &'a glium::Display) -> Signal<'a,T>{
		Signal{	
				name: name.clone(), 
				color: Generator::get_color(name.clone(), 0.8, 1.),
				unit_scale: T::ones().as_vec(), 
				points: RangedDeque::new(), 
				style,
				health: SignalHealth::Good,
				view,
				pick_thresh: 0.1,
				display
			}
	}
	fn get_transform(&self, area: Rect) -> Transform{
		
		let range = self.style.get_range(&self.points); // this range in xy view space

		self.view.get_transform(area, &range)
	}
	fn add_ds_point(&mut self, pt: &Point<T>){
		//TODO:  Do unit scaling here before pass to drawstyle
		self.style.push(&pt, &self.color, &self.points, self.display);
	}
}

pub trait GenericSignal {
    fn draw(&self, target: &mut glium::Frame, area: Rect);
    fn add_point(&mut self, point: MsgPoint);
    fn get_color(&self)-> Color;
    fn get_health(&self) -> SignalHealth;
    fn pick(&self, mouse: (f32, f32), area: Rect)->Option<PickData>;
    fn get_point_strings(&self, idx: usize) -> (String, String, String);
    fn set_style(&mut self, style: &Styles);
    fn set_bind_mode(&self, mode: u8);
    fn get_view(&mut self) -> &mut View;
    fn share_view(&self);
    fn zoom_by(&self, by: f64);
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
		self.points.push(pt.clone());
		self.add_ds_point(&pt);
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
	fn get_point_strings(&self, idx: usize) -> (String, String, String){
		self.style.get_point_strs(self.points.get(idx))
	}
	fn set_style(&mut self, style: &Styles){
		//TODO: full vbo construction
		self.style = match style{
			Styles::Scatter => Box::new(Scatter::new(self.display)),
			Styles::Lines => Box::new(Lines::new(self.display))
		};
		for i in 0..self.points.len(){
			let a = self.points.get(i).clone(); //TODO: gotta be a better way
			self.add_ds_point(&a);
		}
	}
	fn set_bind_mode(&self, mode: u8){
		self.view.set_bind_mode(mode);
	}
	fn get_view(&mut self) -> &mut View{
		&mut self.view
	}
	fn share_view(&self){
		self.view.share(&self.style.get_range(&self.points))
	}
	fn zoom_by(&self, by: f64){
		self.view.zoom(by);
	}
}


pub struct SignalManager<'a> {
	signals: HashMap<String, Box<GenericSignal+'a>>,
	display: &'a glium::Display,
	selection: Option<String>,
	views: Vec<Weak<RefCell<ViewData>>>,
	pub point_count: usize
}

impl <'a> SignalManager<'a>{
	pub fn new(display: &glium::Display) -> SignalManager {
	    SignalManager{
	    	// signals_d1: HashMap::new(), signals_d2: HashMap::new(), signals_d3: HashMap::new(),
	    	signals: HashMap::new(),
	    	display,
	    	selection: None,
	    	views: Vec::new(),
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
				let view = View::new(name.clone());
				self.views.push(Rc::downgrade(&view.data));
				let mut ch: Box<GenericSignal+'a> = match point.ty {
					PointType::D1=> {
						let ds: Box<DrawStyle<D1>> = Box::new(Lines::new(self.display));
						Box::new(Signal::new(name.clone(), ds, view, self.display))
					},
					PointType::D2 => {
						let ds: Box<DrawStyle<D2>> = Box::new(Lines::new(self.display));
						Box::new(Signal::new(name.clone(), ds, view, self.display))
					},
					PointType::D3=> {
						let ds: Box<DrawStyle<D3>> = Box::new(Scatter::new(self.display));
						Box::new(Signal::new(name.clone(), ds, view, self.display))
					},
					PointType::BreakPoint => {
						let ds: Box<DrawStyle<D1>> = Box::new(Scatter::new(self.display));
						Box::new(Signal::new(name.clone(), ds, view, self.display))
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
		for i in self.views.iter(){
			if let Some(v) = i.upgrade(){ //TODO: dont leave invalid refrences in vec
				v.borrow_mut().clear_range();
			}
		}
		for sig in self.signals.values(){
			sig.share_view();
		}
		for (_name, sig) in self.signals.iter(){
			sig.draw(target, area);
		}
	}

	pub fn iter(&self) -> std::collections::hash_map::Iter<String, Box<GenericSignal + 'a>>{
		self.signals.iter()
	}

	pub fn get_names(&self) -> impl Iterator<Item = &String>{
		self.signals.keys()
	}

	//TODO: !Change this interface such that if a signal doesnt exist it is created, this also means changing Signal, so that it is not a generic
	pub fn get_signal(&mut self, name: &str) -> Option<&mut Box<GenericSignal+'a>>{
		self.signals.get_mut(name)
	}
 
	pub fn len(&self) -> usize{
		self.signals.len()
	}

	pub fn get_selection(&mut self) -> &Option<String>{
		if let Some(ref n) = self.selection.clone(){
			if !self.signals.contains_key(n){
				self.selection = None;
			}
		}
		&self.selection
	}
	pub fn get_selected(&mut self) -> Option<&mut Box<GenericSignal+'a>>{
		let name = self.get_selection().clone();
		if let Some(n) = name {
			self.get_signal(&n)
		} else {
			None
		}
	}

	pub fn set_selection(&mut self, s: Option<String>){
		self.selection = if let Some(n) = s{
			if self.signals.contains_key(&n){
				Some(n)
			} else { None}
		} else {
			None
		};
	}

	pub fn bind(&mut self, base: &String, other: &String){
		let v;
		if let Some(s) = self.get_signal(base){
			v = s.get_view().clone();
		} else {
			return;
		}
		if let Some(s) = self.get_signal(other){
			*s.get_view() = v;
		}
	}
}



pub struct PickData{
	pub index: usize,
	pub screen_pos: (f32, f32)
}