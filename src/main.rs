#[macro_use]
extern crate glium;
extern crate glium_text_rusttype as glium_text;
extern crate regex;
#[macro_use]
extern crate clap;
extern crate crossbeam_channel as channel;


use glium::Surface;
use channel::{Receiver, Sender, TryRecvError};

use std::time;
use std::f64::NAN;
use time::{Duration, Instant};
// use std::rc::Rc;

use std::io::{self};
use regex::{Regex, RegexSet, Captures};
use clap::{App};

pub mod signal;
pub mod drawstyles;
// pub use self::signal;
use signal::{MsgPoint as Point, PointType};
use signal::SignalManager;



fn duration2us(dur: &Duration) -> f64 {
   (dur.as_secs() * 1000_000 + dur.subsec_nanos() as u64 / 1000) as f64
}



fn main(){
	//Mark the start of the program
	let epoch = Instant::now();
	//Start a thread to begin polling standard input for new data, any new lines are timestamped and passed along the parsing thread
	let (send_stdin, rx_stdin): (Sender<(Duration, String)>, Receiver<(Duration, String)>) = channel::unbounded();
	std::thread::spawn(move||{
		let mut buffer = String::new();
		loop {
			match io::stdin().read_line(&mut buffer){
				Ok(0) => break, // EOF Reached
				Ok(_n) => { // Received n bytes
							match send_stdin.send((epoch.elapsed(), buffer.clone())){
								Err(_) => println!("{:?}", "Buffer Error"), // Error on channel
								_ => {}
							}
						}
				Err(error) => println!("{:?}", error)
			}
			buffer.clear();
		}
		println!("EOF Reached"); //TODO: Visual signal on key that pipe is closed
	});
	//Parse comand line arguments
	let cmdargs = load_yaml!("../resources/cmdargs.yml");
	let _matches = App::from_yaml(cmdargs).get_matches();


	//Setup GUI
    use glium::glutin;
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new().with_multisampling(8).with_vsync(false);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
	let system = glium_text::TextSystem::new(&display);
	//TODO: Figure out licensing
	let font = glium_text::FontTexture::new(&display, &include_bytes!("../resources/UbuntuMono-R.ttf")[..], 70, glium_text::FontTexture::ascii_character_list()).unwrap();

	//Spawn point processing thread
	let settings = ReaderSettings {};
	let (send_points, rx_points): (Sender<Point>, Receiver<Point>) = channel::unbounded();
	std::thread::spawn(move || {read_thread_main(&rx_stdin, &send_points, &settings);});

	let scale = 0.04;
	let mat = [
        [scale, 0.0, 0.0, 0.],
        [0.0, scale, 0.0, 0.],
        [0.0, 0.0, scale, 0.0],
        [ -0.98 , 0.96, 0.0, 1.0f32],
    ];

	let mut signal_manager = SignalManager::new(&display);
	let refresh_rate = Duration::from_millis(30);
	let mut ft_av = 16000f64;
	//Main render loop
	let mut closed = false;
	while !closed {
		let frametime = Instant::now();
	    events_loop.poll_events(|ev| {
	        match ev {
	            glutin::Event::WindowEvent { event, .. } => match event {
	                glutin::WindowEvent::Closed => closed = true,
	                // glutin::WindowEvent::CursorMoved{position, ..} => {println!(":#P:{:?},{:?}", position.0, position.1);},
	                glutin::WindowEvent::KeyboardInput{input, ..} => {println!("{:?}", input.scancode)},
	                _ => (),
	            },
	            _ => (),
	        }
	    });
	    //TODO: Only redraw when need user input or new data to draw
	    let mut target = display.draw();
	    target.clear_color(0.06, 0.06, 0.06, 1.0);

	    signal_manager.draw_signals(&mut target);

	    let ft = duration2us(&frametime.elapsed());
	    ft_av = 0.95*ft_av + 0.05*ft;
	    let text = glium_text::TextDisplay::new(&system, &font, &ft_av.floor().to_string());
	    glium_text::draw(&text, &system, &mut target, mat, (1.0, 1.0, 1.0, 1.0)).unwrap();
	    target.finish().unwrap();
	    get_points(&rx_points, &mut signal_manager, &frametime, &refresh_rate);
	}
}

fn get_points(rx: &Receiver<Point>, man: &mut SignalManager, frametime: &Instant, refresh_rate: &Duration){
	loop {
		match rx.try_recv() {
			Ok(d) => {
				man.add_point(d);
			},
			Err(e) => {
				match e {
					TryRecvError::Disconnected => {}, //TODO: Continue to draw but at const framerate
					TryRecvError::Empty => {}
				}
				break;
			}
		}
		if frametime.elapsed() > *refresh_rate{
			break;
		}
	}
}

#[derive(Debug)]
struct ReaderSettings {
}


fn read_thread_main(rx_stdin: &Receiver<(Duration, String)>, send_points: &Sender<Point>, settings: &ReaderSettings) {
	let deci: &'static str =  r"[-+]?[0-9]*\.?[0-9]+(?:[eE][-+]?[0-9]+)?";
	let one = &format!("~\\.(.+)@\\s*({})",deci);
	let two = &format!("~\\.(.+)@\\s*({})\\s*,\\s*({})", deci, deci);
	let three = &format!("~\\.(.+)@\\s*({})\\s*,\\s*({})\\s*,\\s*({})", deci, deci, deci);
	let list = &format!(r"~#(.+)#(?:(\d+),(\d+))?@((?:(?:{})|,|\(|\)|\s)+)", deci); //Untested
	let node = r"~%(.+)@(\d+)\[((?:\d+|,|\s)*)\]"; //Untested
	let set = RegexSet::new(&[
			one,
			two,
			three,
			list,
			node
		]).unwrap(); //Guaranteed to unwrap since static input
	let onegrab = Regex::new(one).unwrap(); //Guaranteed to unwrap since static input
	let twograb = Regex::new(two).unwrap(); //Guaranteed to unwrap since static input
	let threegrab = Regex::new(three).unwrap(); //Guaranteed to unwrap since static input
	let listgrab = Regex::new(list).unwrap();
	let nodegrab = Regex::new(node).unwrap();
	let grabbers = [onegrab, twograb, threegrab, listgrab, nodegrab]; // must match order of regex set constructor

	loop {
		match rx_stdin.try_recv() {
			Ok(d) => {
				parse_line(&d.0, &d.1, send_points, &set, &grabbers, settings)
			},
			Err(e) => {
				match e {
					TryRecvError::Disconnected => {}, //TODO: Continue to draw but at const framerate
					TryRecvError::Empty => {}
				}
				break;
			}
		}
	}
}

fn parse_line(timestamp: &Duration, data: &String, tx: &Sender<Point>, set: &RegexSet, grabbers: &[Regex; 5], settings: &ReaderSettings){
	let ts = duration2us(timestamp);
	let which: Vec<usize> = set.matches(data).into_iter().collect();
	if which.len() == 0 { //No candidate matches
		passthrough(data);
	} else {
		let idx = which.into_iter().fold(0, std::cmp::max); // Get the most desired candidate that will match
		match handle_caps(grabbers[idx].captures(data), ts, settings) { // Match the result of handling the capture groups. If a valid point was found send it. Otherwise pass line through and log it. 
			Some(tosend) => { //Vaild point send to main thread
				match tx.send(tosend){
					Ok(_) => {},
					Err(_) => println!("{:?}", "Buffer Error") // Error on channel
				}
			},
			None => passthrough(data)// Data was not valid syntax || marked to be passed through, must be passed on to stdout
		}

	}
}

fn handle_caps(caps: Option<Captures>, timestamp: f64, settings: &ReaderSettings) -> Option<Point>{
	let vals = caps.unwrap(); //Guaranteed unwrap since captured by RegexSet

	let mut c = vals.iter();
	c.next();// Dont want entire match
	let name = String::from(c.next().unwrap().unwrap().as_str()); //TODO: Guarantee unwraps

	//TODO: Filter out signals

	//convert remaining values to floats
	let v: Vec<f64> = c.map(|m| {
			m.map_or(0.0, |m| m.as_str().parse::<f64>().unwrap()) //TODO: should fail gracefully here, set channel health to bad
		}).collect();

	return Some(match v.len(){
		1 => Point{name, timestamp, ty:PointType::D1, x:v[0], y:NAN, z:NAN},
		2 => Point{name, timestamp, ty:PointType::D2, x:v[0], y:v[1], z:NAN},
		3 => Point{name, timestamp, ty:PointType::D3, x:v[0], y:v[1], z:v[2]},
		_ => Point{name, timestamp, ty:PointType::BreakPoint, x:NAN, y:NAN, z:NAN},
	});
}

fn passthrough(line: &String) {
	// Log data for later examination if breakpoint is reached etc
    print!("{}", line) //TODO: Ensure that stdout is still open and writable
}