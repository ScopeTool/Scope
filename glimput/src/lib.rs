extern crate glium;
use glium::glutin::{KeyboardInput, VirtualKeyCode as VKC};


#[derive(Debug)]
pub struct Editor {
    buffer: String,
    cursor: usize
}

impl Editor{
	pub fn new() -> Editor{
		Editor{buffer: String::from("~"), cursor: 1}
	}
	pub fn send_key(&mut self, c: char) -> bool{
		let mut ret = false;
		match c{
			'\u{8}' => if self.cursor > 1 && self.cursor <= self.buffer.len() {self.buffer.remove(self.cursor-1); self.cursor -=1;},
			'\r' => {ret = true},
			_ => {self.buffer.insert(self.cursor, c); self.cursor+=1},
		};
		return ret;
	}

	pub fn clear(&mut self){
		self.buffer.clear();
		self.buffer.push('~');
		self.cursor = 1;
	}

	pub fn send_event(&mut self, input: KeyboardInput){
		if let Some(k) = input.virtual_keycode{
			if input.state == glium::glutin::ElementState::Pressed{
				match k {
				    VKC::Left => if self.cursor > 1{self.cursor -= 1},
				    VKC::Right => if self.cursor < self.buffer.len(){self.cursor += 1},
				    _ => ()
				}
			}
		}
	}

	pub fn get_buffer(&self) -> &str{
		&self.buffer[1..]
	}

	pub fn get_buffer_parts(&self) -> (&str, &str, &str){
		(&self.buffer[0..self.cursor-1], &self.buffer[self.cursor-1..self.cursor], &self.buffer[self.cursor..self.buffer.len()])
	}
}