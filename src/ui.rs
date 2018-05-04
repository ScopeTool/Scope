extern crate glium;
extern crate glium_text_rusttype as glium_text;
extern crate distance;
use self::distance::damerau_levenshtein as fuzzy_dist; 

extern crate command_parse;

use std::{
	rc::Rc,
	cell::RefCell,
};

use glium::{Display, Frame, Rect, Surface};

use glimput::Editor;

use glium::glutin::{KeyboardInput, VirtualKeyCode as VKC};

use signal::{SignalManager, SignalHealth};
use drawstyles::Transform;

type Color = (f32, f32, f32, f32);

const DARK_GREY: Color = (0.01, 0.01, 0.01, 1.0);

pub struct UI<'a> {
	pub signal_manager: SignalManager<'a>,
	editor: Editor,
	window_size: (u32, u32),
	ledgend_width: f32,
	text_height: f32,
	text_system: glium_text::TextSystem,
	text_format: RefCell<glium_text::TextDisplay<Rc<glium_text::FontTexture>>>,
	cmdline_completions: Vec<String>,
	completion_idx: usize
}

impl <'a> UI<'a> {
    pub fn new(display: &Display) -> UI{
	    let system = glium_text::TextSystem::new(display);
		//TODO: figure out the proper way to do this, dont need rc just need to tell rust to alocate this on heap, destroy it with UI and pass refrence to text
		let font = Rc::new(glium_text::FontTexture::new(display, &include_bytes!("../resources/UbuntuMono-R.ttf")[..], 100, glium_text::FontTexture::ascii_character_list()).unwrap());
    	let signal_manager = SignalManager::new(display);
		let editor = Editor::new();
		let text = glium_text::TextDisplay::new(&system, 
			font.clone(),
			 "Aj");
		let text_height = text.get_height()*1.3;
    	UI{signal_manager, editor, window_size: (0,0), ledgend_width: 0.2, text_height, text_system: system, text_format: RefCell::new(text),
    		cmdline_completions: Vec::new(), completion_idx: 0
    	}
    }
    pub fn draw(&mut self, target: &mut Frame, window_size: (u32, u32), mouse_pos: (f64, f64), frametime: f64 ){
    	self.window_size = window_size;
		self.draw_text(target, -0.98, 0.97, 0.04, (1.0, 1.0, 1.0, 1.0), &frametime.floor().to_string());

		println!("~.{}@{:?}", "frametime", frametime);
		println!("~.{}@{:?}", "points", self.signal_manager.point_count);
		println!("~.{}@{:?}", "performance/pt", frametime/self.signal_manager.point_count as f64);

		let mouse = ((2.*(mouse_pos.0/(window_size.0 as f64))-1.) as f32, (1. - 2.*(mouse_pos.1/(window_size.1 as f64))) as f32);

	    let view_start_x = -1.0 + self.get_axis_width();
	    let view_start_y = -1.0 + self.get_axis_height() + self.get_cmd_height();
	    let view_end_x = 1.0 - self.get_log_width() - self.get_ledgend_width() - self.get_axis_width();
	    let view_end_y = 1.0 - self.get_axis_height();

	    let area = (view_start_x, view_start_y, view_end_x, view_end_y);

	    self.signal_manager.draw_signals(target, area);

	    self.draw_cmdline(target,  area);

	    self.draw_ledgend(target, view_end_x);

	    // self.draw_rect(target, (1.,1.,1.,1.), (mouse.0 as f64, mouse.1 as f64), (0.1,0.1) );

	    for (_, sig) in self.signal_manager.iter(){
	    	if let Some(pick) = sig.pick(mouse, area){
	    		let c = sig.get_color();
	    		let scale = 0.06;
	    		let pad = 0.01;
	    		let text = &sig.get_point_string(pick.index);
	    		let dims = self.draw_text(target, pick.screen_pos.0 as f64 , pick.screen_pos.1 as f64 + scale as f64 * self.text_height as f64/2., scale, (c.0, c.1, c.2, 1.0), text); //TODO: Dont redraw just for size
	    		self.draw_rect(target, DARK_GREY, (pick.screen_pos.0 as f64 -pad/2., pick.screen_pos.1 as f64 - pad/2.), (dims.0+pad, dims.1+pad));
			    self.draw_text(target, pick.screen_pos.0 as f64 , pick.screen_pos.1 as f64 + scale as f64 * self.text_height as f64/2., scale, (c.0, c.1, c.2, 1.0), text);
	    	}
	    }
    }

    pub fn send_key(&mut self, c: char){
    	if c != '\t'{
	    	let run = self.editor.send_key(c);
			self.update_editor(run);
		}
    }

    pub fn send_event(&mut self, input: KeyboardInput){
	    if let Some(k) = input.virtual_keycode{
			if input.state == glium::glutin::ElementState::Released{
				match k {
				    VKC::Tab => {
				    	let mut cmpl = None;
				    	if let Some(c) = self.get_completion(){
				    		cmpl = Some(String::from(c));
				    	}
				    	if let Some(cmpl) = cmpl{
					    	self.editor.autofill(&cmpl)
				    	}
				    	//TODO: there has to be a better way to do this
				    }, 
				    VKC::Return => if input.modifiers.shift && self.cmdline_completions.len() > 0 { 
				    	self.completion_idx = (self.completion_idx + 1) % self.cmdline_completions.len() 
				    }
				    _ => ()
				}
			}
		}
		self.editor.send_event(input);
    }

    fn update_editor(&mut self, run: bool){
    	let rslt = command_parse::parse(self.editor.get_buffer(), run);
    	if rslt.valid && run{
    		self.editor.clear();
    	}
    	self.cmdline_completions = rslt.possible_completions;
    	let current_term = &self.editor.get_working_term();
    	self.cmdline_completions.sort_by(|s1, s2|{
    		fuzzy_dist(current_term, s1).cmp(&fuzzy_dist(current_term, s2))
    	});
    	self.completion_idx = 0;
    }

    fn draw_cmdline(&self, target: &mut Frame, area: (f64, f64, f64, f64)){
    	let mut rhs = area.0;
    	let cmd_com_y = (area.1-1.0)/2.0;
    	let cmd_height = self.get_cmd_height()*0.95;
    	let ypos = cmd_com_y - cmd_height/2.0;
    	self.draw_rect(target, DARK_GREY, (rhs, ypos),((area.2-area.0), cmd_height));
    	let scale = (cmd_height as f32)*0.65/(self.text_height);

    	let (first, c, rest) = self.editor.get_buffer_parts();
    	let (last, _) = self.draw_text(target, rhs, cmd_com_y, scale, (1.,1.,1., 1.0), first);
    	rhs += last;
    	let (last, _) = self.draw_text(target, rhs, cmd_com_y, scale, (0.8,0.2,0.1, 1.0), c);
    	rhs += last;
    	let (last, _) = self.draw_text(target, rhs, cmd_com_y, scale, (1.,1.,1., 1.0), rest);
    	if let Some(cmpl) = self.get_completion(){
	    	rhs += last;
	    	let mut text_dims = self.get_text_dims(scale, cmpl);
	    	text_dims = (text_dims.0*1.1, text_dims.1*1.1);
	    	self.draw_rect(target, DARK_GREY, (rhs, ypos+cmd_height), text_dims);
	    	self.draw_text(target, rhs, ypos+cmd_height + text_dims.1/2., scale, (0.5,0.5,0.5, 1.0), cmpl);
    	}
    }

    fn get_completion(&self) -> Option<&str>{
    	if self.cmdline_completions.len() > 0{
	    	return Some(&self.cmdline_completions[self.completion_idx]);
	    }
	    return None;
    }

    fn draw_ledgend(&mut self, target: &mut Frame, view_end_x: f64){
		let scale = 0.08;
		let mut pos = 1.0 - self.get_axis_width();
		let mut max_width = 0.0f32;
		let th = self.text_height*scale;
		let stat_width = (th*self.resquare()) as f64;
		// self.draw_rect(target, DARK_GREY, (view_end_x+self.get_axis_width(), len), (self.ledgend_width as f64-0.08, 0.5));
    	for (name, sig) in self.signal_manager.iter(){
			let c = sig.get_color();
			let ts = view_end_x+self.get_axis_width()*1.1;
		    self.draw_rect(target, 
		    	match sig.get_health(){
		    		SignalHealth::Good => (62.0/256.0, 107.0/256.0, 12.0/256.0, 1.),
		    		SignalHealth::InvalidFormat => (1., 0., 0., 1.),
		    	}
		    	, (ts as f64, pos as f64), (stat_width, th as f64));
		    let (tw, _) = self.draw_text(target, ts + stat_width, pos+th as f64/2., scale, (c.0, c.1, c.2, 1.0), &name);
    		max_width = max_width.max(tw as f32);
		    pos -= th as f64;
    	}
    	self.ledgend_width = max_width+0.08+stat_width as f32;
    }

    // Input in 3D ogl space
    fn draw_rect(&self, target: &mut glium::Frame, color: Color, corner: (f64, f64), dims: (f64, f64)){
    	let pxx = self.window_size.0 as f64 / 2.0;
    	let pxy = self.window_size.1 as f64 / 2.0;
    	let cornerx = (self.window_size.0/2) as i32 + (pxx*corner.0) as i32;
    	let cornery = (self.window_size.1/2) as i32 + (pxy*corner.1) as i32;
    	let width = (dims.0*pxx) as u32;
    	let height = (dims.1*pxy) as u32;
    	target.clear(Some(&Rect{
    		left: cornerx as u32,
    		bottom: cornery as u32,
    		width: width,
    		height: height,
    	}), Some(color), false, None, None);
    }

    fn draw_text(&self, target: &mut glium::Frame, x: f64, y: f64, scale: f32, color: Color, text: &str) -> (f64, f64){
    	let trans = Transform{dx: x as f32, dy: y as f32 - self.text_height*scale/2.9, sx: scale*self.resquare(), sy: scale, sz: 1.};
    	let mut tf = self.text_format.borrow_mut();
    	tf.set_text(text);
    	glium_text::draw(&tf, &self.text_system, target, &trans.into(), color).unwrap();
    	((tf.get_width()*trans.sx )as f64, (tf.get_height()*trans.sy*1.3) as f64)
    }

    fn get_text_dims(&self, scale: f32,  text: &str) -> (f64,  f64){
    	let mut tf = self.text_format.borrow_mut();
    	tf.set_text(text);
    	((tf.get_width()*scale*self.resquare() )as f64, (tf.get_height()*scale*1.3) as f64)
    }

    fn get_log_width(&self) -> f64{
    	0.0
    }

    fn get_ledgend_width(&self) -> f64{
    	self.ledgend_width as f64
    }

    fn get_cmd_height(&self) -> f64{
    	0.1
    }

    fn get_axis_width(&self) -> f64{
    	0.08
    }

    fn get_axis_height(&self) -> f64{
    	0.05
    }

    fn resquare(&self) -> f32 {
    	(self.window_size.1 as f32 / self.window_size.0 as f32)
    }

}