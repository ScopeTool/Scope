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


pub type Rect = (f64, f64, f64, f64);

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
pub trait Axes<T>: 
	std::ops::Index<usize, Output = <Self as Axes<T>>::AxesUnit> {
    type AxesUnit: std::marker::Sized+std::convert::Into<f64>+Clone;
	fn size() -> usize;
	fn default() -> T;
	fn ones() -> T;
	fn as_vec(&self) -> Vec<f64>;
	fn into(point: MsgPoint) -> Point<T>;
	fn timestamp() -> usize;
	fn x() -> usize;
	fn y() -> usize;
	fn z() -> i32;
}
impl Axes<D1> for D1 {
	type AxesUnit = f64;
	fn size() -> usize{2}
	fn default() -> D1 {D1{0:[NAN; 2]}} 
	fn ones() -> D1 {D1{0:[1.; 2]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn into(point: MsgPoint) -> Point<D1>{
		Point::new(D1{0:[point.timestamp, point.x]})
	}
	fn timestamp() -> usize{0}
	fn x() -> usize{Self::timestamp()}
	fn y() -> usize{1}
	fn z() -> i32{-1}
}
impl Axes<D2> for D2 {
	type AxesUnit = f64;
	fn size() -> usize{3}
	fn default() -> D2 {D2{0:[NAN; 3]}} 
	fn ones() -> D2 {D2{0:[1.; 3]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn into(point: MsgPoint) -> Point<D2>{
		Point::new(D2{0:[point.timestamp, point.x, point.y]})
	}
	fn timestamp() -> usize{0}
	fn x() -> usize{1}
	fn y() -> usize{2}
	fn z() -> i32{-1}
}
impl Axes<D3> for D3 {
	type AxesUnit = f64;
	fn size() -> usize{4}
	fn default() -> D3 {D3{0:[NAN; 4]}} 
	fn ones() -> D3 {D3{0:[1.; 4]}}
	fn as_vec(&self) -> Vec<f64>{self.0.to_vec()}
	fn into(point: MsgPoint) -> Point<D3>{
		Point::new(D3{0:[point.timestamp, point.x, point.y, point.z]})
	}
	fn timestamp() -> usize{0}
	fn x() -> usize{1}
	fn y() -> usize{2}
	fn z() -> i32{3}
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
	fn new_cap(cap: usize) -> Range{
		Range{min: vec![NAN; cap], max: vec![NAN; cap]}
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
	A: Axes<A> + Clone{
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
	pos: (f64, f64),
    zoom: f64,
    range: Range,
    maintain_aspect: bool,
    id: String
}
impl ViewData{
	fn clear_range(&mut self){
		for i in 0..self.range.min.len(){
		    self.range.min[i] = NAN;
			self.range.max[i] = NAN;
		}
	}
}

#[repr(u8)]
pub enum AxisBind {
	None = 	0b0000u8,
    X = 	0b0010u8,
    Y = 	0b0100u8,
    Z = 	0b1000u8,
    SX = 	0b10000u8,
    SY = 	0b100000u8,
    SZ = 	0b1000000u8,
}

#[derive(Debug, Clone)]
pub struct View {
    data: Rc<RefCell<ViewData>>,
    mode: u8,
    local_pos: (f64, f64)
}
impl View {
    fn new(name: String) -> View{
    	let data = Rc::new(RefCell::new(
    		ViewData{
    			pos: (0.,0.), 
	    		zoom: 1., 
	    		range: Range::new_cap(3), 
	    		maintain_aspect: false, 
	    		id: name
	    	}));
    	View{data, mode: 0, local_pos: (0.,0.)}
    }

    fn share(&self, range: &Range){
    	let mut data = self.data.borrow_mut();
    	let r = &mut data.range;

    	for i in 0..r.min.len().max(range.min.len()){
    		if self.mode & (1 << (i+1)) != 0 {
	    		let mut a = if i < range.min.len() {range.min[i]} else {NAN};
	    		let mut b = if i < r.min.len() {r.min[i]} else {NAN};
	    		r.min[i] = a.min(b);
	    	}
    	}
    	for i in 0..r.max.len().max(range.max.len()){
    		if self.mode & (1 << (i+1)) != 0 {
	    		let mut a = if i < range.max.len() {range.max[i]} else {NAN};
	    		let mut b = if i < r.max.len() {r.max[i]} else {NAN};
	    		r.max[i] = a.max(b);
	    	}
    	}
    }

    fn get_transform(&self, area: Rect, range: &Range) -> Transform{
    	let data = self.data.borrow();

    	let (xs, ys, _, xmax, ymin, ymax) = self.get_working_scale(&data, area, range);

    	let dx = if AxisBind::X as u8 & self.mode != 0{data.pos.0} else {self.local_pos.0};
    	let dy = if AxisBind::Y as u8 & self.mode != 0{data.pos.1} else {self.local_pos.1};

		Transform{
			dx: (area.2-xmax*xs + dx) as f32, dy: ((-ys*(ymax+ymin)/2.)+(area.3+area.1)/2.0 + dy) as f32,
			// dx: dx as f32, dy: dy as f32,
			sx: xs as f32, sy: ys as f32, sz: 1.0
		}
	}

	fn get_working_scale(&self, data: &ViewData, area: Rect, range: &Range) -> (f64, f64, f64, f64, f64, f64){
    	let mut working_range = if AxisBind::X as u8 & self.mode != 0{&data.range} else {range};
    	let xmin = working_range.min[0];
    	let xmax = working_range.max[0];
    	working_range = if AxisBind::Y as u8 & self.mode != 0{&data.range} else {range};
    	let ymin = working_range.min[1];
    	let ymax = working_range.max[1];

    	// println!("Working Range: x: ({:?}, {:?}), y: ({:?}, {:?})", xmin, xmax, ymin, ymax);

    	let mut xs = ((area.2-area.0)/(xmax-xmin)).max(MIN_SCALE);
		let mut ys = ((area.3-area.1)/(ymax-ymin)).max(MIN_SCALE); 

		if data.maintain_aspect{
			let val = if (1.-xs).abs() > (1.-ys).abs() {xs} else {ys};
			xs = val; ys = val;
		}

		let zoom = data.zoom.max(1.);

		(xs * zoom, ys * zoom, xmin, xmax, ymin, ymax)
	}

	fn zoom(&mut self, by: f64, center: (f64, f64)){
		let mut data = self.data.borrow_mut();
		let last = data.zoom;
		data.zoom = (data.zoom + by/10.).max(1.);
		let zoom = data.zoom;
		let mut final_zoom = data.zoom;
		let dz = data.zoom - last;
		{
			let pos = &mut data.pos;
			let x = if AxisBind::X as u8 & self.mode != 0{&mut (pos.0)} else {&mut self.local_pos.0};
			let y = if AxisBind::Y as u8 & self.mode != 0{&mut (pos.1)} else {&mut self.local_pos.1};
			if zoom > 1.{
				*x += (center.0)*dz;
				*y += (center.1)*dz;
			} else {
				final_zoom = 1.;
				*x = 0.;
				*y = 0.;
			}
		}
		data.zoom = final_zoom;
	}

	// Takes screen position mouse dx and dy
	fn move_by(&mut self, by: (f64, f64), _area: Rect, _range: &Range){
		let data = &mut self.data.borrow_mut().pos;
		let x = if AxisBind::X as u8 & self.mode != 0{&mut (data.0)} else {&mut self.local_pos.0};
		let y = if AxisBind::Y as u8 & self.mode != 0{&mut (data.1)} else {&mut self.local_pos.1};
		*x += by.0;
		*y += by.1;
	}

    fn set_bind_mode(&mut self, mode: u8){
    	self.mode = mode;
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
	where T: Axes<T> + Clone{
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
	fn get_name(&self) -> &String;
    fn draw(&self, target: &mut glium::Frame, area: Rect);
    fn add_point(&mut self, point: MsgPoint);
    fn get_color(&self)-> Color;
    fn get_health(&self) -> SignalHealth;
    fn pick(&self, mouse: (f32, f32), area: Rect)->Option<PickData>;
    fn get_point_strings(&self, idx: usize) -> (String, String, String);
    fn set_style(&mut self, style: &Styles);
    fn set_bind_mode(&mut self, mode: u8);
    fn get_view(&mut self) -> &mut View;
    fn share_view(&self);
    fn zoom_by(&mut self, by: f64, center: (f64, f64));
    fn move_view_by(&mut self, by: (f64, f64), area: Rect);
}

impl <'a, T> GenericSignal for Signal<'a, T>
	where T: Axes<T> + Clone + std::fmt::Debug{
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
	fn set_bind_mode(&mut self, mode: u8){
		self.view.set_bind_mode(mode);
	}
	fn get_view(&mut self) -> &mut View{
		&mut self.view
	}
	fn share_view(&self){
		self.view.share(&self.style.get_range(&self.points))
	}
	fn zoom_by(&mut self, by: f64, center: (f64, f64)){
		self.view.zoom(by, center);
	}
	fn move_view_by(&mut self, by: (f64, f64), area: Rect){
		self.view.move_by(by, area, &self.style.get_range(&self.points));
	}
	fn get_name(&self) -> &String{
		&self.name
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
	pub fn free(&mut self, name: &String){
		let view = View::new(name.clone());
		let mut cpy = false;
		if let Some(s) = self.get_signal(name){
			*s.get_view() = view.clone();
			cpy = true;
		}
		if cpy {
			self.views.push(Rc::downgrade(&view.data));
		}
	}
}



pub struct PickData{
	pub index: usize,
	pub screen_pos: (f32, f32)
}