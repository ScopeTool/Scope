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
	pub fn send_key(&mut self, c: char){
		match c{
			'\u{8}' => if self.cursor > 1 && self.cursor <= self.buffer.len() {self.buffer.remove(self.cursor-1); self.cursor -=1;},
			_ => {self.buffer.insert(self.cursor, c); self.cursor+=1},
		};
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

	pub fn clear(&mut self){
		self.buffer.clear();
		self.buffer.push('~');
		self.cursor = 1;
	}

	pub fn autofill(&mut self, sub: &str){
		let mut b = self.buffer.chars().collect::<Vec<_>>();
		let mut pt = self.cursor-1;
		while pt > 0 && !is_delim(b[pt]) {
			b.remove(pt);
			pt -= 1;
		} 
		pt += 1;
		while pt < b.len() && !is_delim(b[pt]) {
			b.remove(pt);
		}

		self.buffer = b.into_iter().collect();
		self.buffer.insert_str(pt, sub); //TODO: how will this work out with unicode?
		self.cursor = pt+sub.len();
		println!("{:?}", self.buffer);
	}

	pub fn get_working_term(&self) -> String{
		let b = self.buffer.chars().collect::<Vec<_>>();
		let mut pt = self.cursor-1;
		while pt > 0 && !is_delim(b[pt]) {
			pt -= 1;
		} 
		pt += 1;
		let low = pt;
		while pt < b.len() && !is_delim(b[pt]) {
			pt+=1;
		}
		return b[low..pt].into_iter().collect()
	}


	pub fn get_buffer(&self) -> &str{
		&self.buffer[1..]
	}

	pub fn get_buffer_parts(&self) -> (&str, &str, &str){
		(&self.buffer[0..self.cursor-1], &self.buffer[self.cursor-1..self.cursor], &self.buffer[self.cursor..self.buffer.len()])
	}
}
	fn is_delim(c: char) -> bool{
		return ' ' == c;
	}