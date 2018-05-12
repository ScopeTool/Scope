// get suggestions given current buffer state, and parse buffer and set state
use signal::SignalManager;


pub struct LineState{
	pub valid: bool,
	pub possible_completions: Vec<String>
}



pub fn parse(line: &str, run: bool, manager: &mut SignalManager) -> LineState {
    let mut valid = false;
    let mut possible_completions = Vec::<String>::new();
	if line.len() > 1 {
	    let cmd: &str = line.split_whitespace().take(1).collect::<Vec<&str>>()[0];
	    valid = true;

	    match cmd{
	    	"ds"|"drawstyle" => drawstyle(line, run, &mut valid, &mut possible_completions, manager),
	    	&_ => if run {println!("Invalid Command: {:?}", cmd)} else {possible_completions.push(String::from("drawstyle"))}
	    }
	}
    LineState{valid, possible_completions}
}


fn drawstyle(cmd: &str, run: bool, valid: &mut bool, pc: &mut Vec<String>, _manager: &mut SignalManager) {
	let bits = cmd.split_whitespace().collect::<Vec<&str>>();
	if bits.len() > 1{
		match bits[1]{
			"scatter" => if run {},
			"lines" => if run {},
			&_ => {
				*valid = false;
				pc.push(String::from("scatter")); pc.push(String::from("lines"));
			} 
		}
	}
}