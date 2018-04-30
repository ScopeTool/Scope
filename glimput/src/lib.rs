#[derive(Debug)]
pub struct Editor {
    buffer: String
}

impl Editor{
	pub fn new() -> Editor{
		Editor{buffer: String::from("$")}
	}
	pub fn send_key(&mut self, c: char){
		println!("{:?}", c);
		match c{
			'\u{8}' => {self.buffer.pop();},
			_ => self.buffer.push(c),
		};
	}
	pub fn get_buffer(&self) -> &str{
		&self.buffer
	}
}