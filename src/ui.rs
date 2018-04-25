extern crate glium;
extern crate glium_text_rusttype as glium_text;

use glium::{Display, Frame, Rect, Surface};

use self::glium_text::{TextSystem, FontTexture};

use signal::{SignalManager, SignalHealth};
use drawstyles::Transform;

type Color = (f32, f32, f32, f32);

pub struct UI<'a> {
	system: TextSystem,
	font:  FontTexture,
	pub signal_manager: SignalManager<'a>,
	window_size: (u32, u32),
	ledgend_width: f32
}

impl <'a> UI<'a> {
    pub fn new(display: &Display) -> UI{
	    let system = glium_text::TextSystem::new(display);
		//TODO: Figure out licensing
		let font = glium_text::FontTexture::new(display, &include_bytes!("../resources/UbuntuMono-R.ttf")[..], 70, glium_text::FontTexture::ascii_character_list()).unwrap();
    	let signal_manager = SignalManager::new(display);
    	UI{system, font, signal_manager, window_size: (0,0), ledgend_width: 0.2}
    }
    pub fn draw(&mut self, target: &mut Frame, window_size: (u32, u32), mouse_pos: (f64, f64), frametime: f64 ){
    	self.window_size = window_size;
    	{
			let scale = 0.04;
			let mat = [
		        [scale, 0.0, 0.0, 0.],
		        [0.0, scale, 0.0, 0.],
		        [0.0, 0.0, scale, 0.0],
		        [ -0.98 , 0.96, 0.0, 1.0f32],
		    ];
			let text = glium_text::TextDisplay::new(&self.system, &self.font, &frametime.floor().to_string());
		    glium_text::draw(&text, &self.system, target, mat, (1.0, 1.0, 1.0, 1.0)).unwrap();
		}

		let mouse = ((2.*(mouse_pos.0/(window_size.0 as f64))-1.) as f32, (1. - 2.*(mouse_pos.1/(window_size.1 as f64))) as f32);

	    let view_start_x = -1.0 + self.get_axis_width();
	    let view_start_y = -1.0 + self.get_axis_height() + self.get_cmd_height();
	    let view_end_x = 1.0 - self.get_log_width() - self.get_ledgend_width() - self.get_axis_width();
	    let view_end_y = 1.0 - self.get_axis_height();

	    let area = (view_start_x, view_start_y, view_end_x, view_end_y);

	    self.signal_manager.draw_signals(target, area);

	    let cmd_height = self.get_cmd_height()*0.9;
	    self.draw_rect(target, (0.01, 0.01, 0.01, 1.0), (view_start_x,(view_start_y-1.0)/2.0 - cmd_height/2.0),((view_end_x-view_start_x), cmd_height));

	    self.draw_ledgend(target, view_end_x);

	    // self.draw_rect(target, (1.,1.,1.,1.), (mouse.0 as f64, mouse.1 as f64), (0.1,0.1) );

	    for (_, sig) in self.signal_manager.iter(){
	    	if let Some(pick) = sig.pick(mouse, area){
    			let scale = 0.04;
    			let mat = [
    		        [scale, 0.0, 0.0, 0.],
    		        [0.0, scale, 0.0, 0.],
    		        [0.0, 0.0, scale, 0.0],
    		        [ pick.screen_pos.0 , pick.screen_pos.1, 0.0, 1.0f32],
    		    ];
	    		let text = glium_text::TextDisplay::new(&self.system, &self.font, &sig.get_point_string(pick.index));
	    		let c = sig.get_color();
	    		self.draw_rect(target, (0.,0.,0.,1.), (pick.screen_pos.0 as f64, pick.screen_pos.1 as f64 - 0.01f64), ((text.get_width()*scale )as f64, (text.get_height()*scale*1.5) as f64));
			    glium_text::draw(&text, &self.system, target, mat, (c.0, c.1, c.2, 1.0)).unwrap();
	    	}
	    }
    }

    fn draw_ledgend(&mut self, target: &mut Frame, view_end_x: f64){
		let scale = 0.04;
		let mut pos = 1.0 - self.get_axis_width() as f32;
		let mut text = glium_text::TextDisplay::new(&self.system, &self.font, "A");
		let mut max_width = 0.0f32;
		let th = text.get_height()*scale*1.5;
    	for (name, sig) in self.signal_manager.iter(){
    		text.set_text(&name);
    		max_width = max_width.max(text.get_width());
    		
			let c = sig.get_color();
			let ts = view_end_x+self.get_axis_width()*1.1;
		    self.draw_rect(target, 
		    	match sig.get_health(){
		    		SignalHealth::Good => (62.0/256.0, 107.0/256.0, 12.0/256.0, 1.),
		    		SignalHealth::InvalidFormat => (1., 0., 0., 1.),
		    	}
		    	, (ts as f64, pos as f64), (th as f64, th as f64));
			let trans = Transform{dx: ts as f32 + th as f32, dy: pos, sx: scale, sy: scale, sz: scale};
		    glium_text::draw(&text, &self.system, target, &trans.into(), (c.0, c.1, c.2, 1.0)).unwrap();
		    pos -= th;
    	}
    	self.ledgend_width = max_width*scale*1.5+0.01+th;
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

    fn get_log_width(&self) -> f64{
    	0.0
    }

    fn get_ledgend_width(&self) -> f64{
    	self.ledgend_width as f64
    }

    fn get_cmd_height(&self) -> f64{
    	0.2
    }

    fn get_axis_width(&self) -> f64{
    	0.08
    }

    fn get_axis_height(&self) -> f64{
    	0.05
    }
}