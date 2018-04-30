extern crate glium;
extern crate glium_text_rusttype as glium_text;

use std::{
	rc::Rc,
	cell::RefCell,
};

use glium::{Display, Frame, Rect, Surface};

use glimput::Editor;

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
    	UI{signal_manager, editor, window_size: (0,0), ledgend_width: 0.2, text_height, text_system: system, text_format: RefCell::new(text)}
    }
    pub fn draw(&mut self, target: &mut Frame, window_size: (u32, u32), mouse_pos: (f64, f64), frametime: f64 ){
    	self.window_size = window_size;
		self.draw_text(target, -0.98, 0.97, 0.04, (1.0, 1.0, 1.0, 1.0), &frametime.floor().to_string());

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
    	self.editor.send_key(c);
    }

    fn draw_cmdline(&self, target: &mut Frame, area: (f64, f64, f64, f64)){
    	let rhs = area.0;
    	let cmd_com_y = (area.1-1.0)/2.0;
    	let cmd_height = self.get_cmd_height()*0.95;
    	self.draw_rect(target, DARK_GREY, (rhs,cmd_com_y - cmd_height/2.0),((area.2-area.0), cmd_height));
    	let scale = (cmd_height as f32)*0.65/(self.text_height);
    	self.draw_text(target, rhs, cmd_com_y, scale, (1.,1.,1., 1.0), &self.editor.get_buffer());
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