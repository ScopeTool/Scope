#[macro_use]
extern crate glium;
extern crate regex;
#[macro_use]
extern crate clap;
extern crate crossbeam_channel as channel;

extern crate glimput;

use channel::{Receiver, RecvTimeoutError, Sender};
use glium::Surface;

use std::f64::NAN;
use std::thread::sleep;
use std::time;
use time::{Duration, Instant};
// use std::rc::Rc;

use clap::App;
use regex::{Captures, Regex, RegexSet};
use std::io::{self, Write};

pub mod command_parse;
pub mod drawstyles;
pub mod signal;
pub mod ui;

use signal::SignalManager;
use signal::{MsgPoint as Point, PointType};

use ui::UI;

fn duration2us(dur: &Duration) -> f64 {
    (dur.as_secs() * 1000_000 + dur.subsec_nanos() as u64 / 1000) as f64
}

//TODO: Forward std error
fn main() {
    //Mark the start of the program
    let epoch = Instant::now();
    //Start a thread to begin polling standard input for new data, any new lines are timestamped and passed along the parsing thread
    let (send_stdin, rx_stdin): (
        Sender<(Duration, String, usize)>,
        Receiver<(Duration, String, usize)>,
    ) = channel::unbounded();
    let _read_thread = std::thread::spawn(move || {
        let mut line_number = 0usize;
        let mut buffer = String::new();
        loop {
            match io::stdin().read_line(&mut buffer) {
                Ok(0) => break, // EOF Reached
                Ok(_n) => {
                    // Received n bytes
                    match send_stdin.send((epoch.elapsed(), buffer.clone(), line_number)) {
                        Err(e) => println!("{:?}", e), // Error on channel
                        _ => {}
                    }
                    line_number += 1;
                }
                Err(error) => println!("{:?}", error),
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
    let window = glutin::WindowBuilder::new().with_title("Scope");
    let context = glutin::ContextBuilder::new()
        .with_multisampling(0)
        .with_vsync(false);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    //Spawn point processing thread
    let settings = ReaderSettings {};
    let (send_points, rx_points): (Sender<Point>, Receiver<Point>) = channel::unbounded();
    let _parse_thread = std::thread::spawn(move || {
        read_thread_main(&rx_stdin, &send_points, &settings);
    });

    display
        .gl_window()
        .window()
        .set_cursor_state(glutin::CursorState::Hide);

    let mut ui = UI::new(&display);

    // display.get_free_video_memory()

    let mut window_size = display.gl_window().get_inner_size().unwrap();

    let refresh_rate = Duration::from_millis(30); //TODO: setting refresh rate to 15ms results in serious performance degradation
    let mut ft_av = 16000f64;
    //Main render loop
    let mut closed = false;
    while !closed {
        let frametime = Instant::now();
        //TODO: Only redraw when need user input or new data to draw
        events_loop.poll_events(|ev| match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => {
                    closed = true;
                }
                glutin::WindowEvent::Resized { .. } => {
                    window_size = display.gl_window().get_inner_size().unwrap()
                }
                glutin::WindowEvent::MouseWheel { .. }
                | glutin::WindowEvent::MouseInput { .. }
                | glutin::WindowEvent::CursorMoved { .. } => ui.send_mouse(event),
                glutin::WindowEvent::KeyboardInput { input, .. } => ui.send_event(input),
                glutin::WindowEvent::ReceivedCharacter(c) => ui.send_key(c),
                _ => (),
            },
            _ => (),
        });
        if closed {
            break;
        } // glutin window closed event makes clear_color hang

        let mut target = display.draw();

        target.clear_color(0.012, 0.012, 0.012, 1.0);

        ui.draw(
            &mut target,
            display.gl_window().hidpi_factor() as f64,
            window_size,
            ft_av,
        );

        ft_av = 0.95 * ft_av + 0.05 * duration2us(&frametime.elapsed());

        target.finish().unwrap();

        get_points(
            &rx_points,
            &mut ui.signal_manager,
            &frametime,
            &refresh_rate,
        );
    }
}

fn get_points(
    rx: &Receiver<Point>,
    man: &mut SignalManager,
    frametime: &Instant,
    refresh_rate: &Duration,
) {
    loop {
        match rx.recv_timeout(
            refresh_rate
                .checked_sub(frametime.elapsed())
                .unwrap_or_else(|| Duration::from_millis(0)),
        ) {
            Ok(d) => {
                man.add_point(d);
            }
            Err(e) => {
                match e {
                    RecvTimeoutError::Disconnected => {} //TODO: Continue to draw but at const framerate
                    RecvTimeoutError::Timeout => return,
                }
                break;
            }
        }
        if frametime.elapsed() > *refresh_rate {
            break;
        }
    }
}

#[derive(Debug)]
struct ReaderSettings {}

fn read_thread_main(
    rx_stdin: &Receiver<(Duration, String, usize)>,
    send_points: &Sender<Point>,
    settings: &ReaderSettings,
) {
    let deci: &'static str = r"[-+]?[0-9]*\.?[0-9]+(?:[eE][-+]?[0-9]+)?";

    let csv = &format!("^(?:({})(?:\\s+|,\\s*|$)){{1,3}}$", deci);
    let csvgrab = Regex::new(csv).unwrap();

    let one = &format!("~\\.(.+)@\\s*({})", deci);
    let two = &format!("~\\.(.+)@\\s*({})\\s*,\\s*({})", deci, deci);
    let three = &format!(
        "~\\.(.+)@\\s*({})\\s*,\\s*({})\\s*,\\s*({})",
        deci, deci, deci
    );
    let list = &format!(r"~#(.+)#(?:(\d+),(\d+))?@((?:(?:{})|,|\(|\)|\s)+)", deci); //Untested
    let node = r"~%(.+)@(\d+)\[((?:\d+|,|\s)*)\]"; //Untested
    let set = RegexSet::new(&[one, two, three, list, node]).unwrap(); //Guaranteed to unwrap since static input
    let onegrab = Regex::new(one).unwrap(); //Guaranteed to unwrap since static input
    let twograb = Regex::new(two).unwrap(); //Guaranteed to unwrap since static input
    let threegrab = Regex::new(three).unwrap(); //Guaranteed to unwrap since static input
    let listgrab = Regex::new(list).unwrap();
    let nodegrab = Regex::new(node).unwrap();
    let grabbers = [onegrab, twograb, threegrab, listgrab, nodegrab]; // must match order of regex set constructor

    loop {
        match rx_stdin.recv() {
            Ok(d) => parse_line(
                &d.0,
                &d.1,
                d.2,
                send_points,
                &set,
                &grabbers,
                settings,
                &csvgrab,
            ),
            Err(_) => sleep(Duration::from_secs(1)),
        }
    }
}

fn parse_line(
    timestamp: &Duration,
    in_data: &String,
    ln: usize,
    tx: &Sender<Point>,
    set: &RegexSet,
    grabbers: &[Regex],
    settings: &ReaderSettings,
    iscsv: &Regex,
) {
    let ts = duration2us(timestamp);

    //Quick and dirty hack to accept CSVs
    let mut data = in_data;
    let mut s;
    if iscsv.is_match(data) {
        s = String::from("~.csv@");
        s.push_str(data);
        data = &s
    }

    let which: Vec<usize> = set.matches(data).into_iter().collect();
    if which.len() == 0 {
        //No candidate matches
        passthrough(data);
    } else {
        let idx = which.into_iter().fold(0, std::cmp::max); // Get the most desired candidate that will match
        match handle_caps(grabbers[idx].captures(data), ts, ln, settings) {
            // Match the result of handling the capture groups. If a valid point was found send it. Otherwise pass line through and log it.
            Some(tosend) => {
                //Vaild point send to main thread
                match tx.send(tosend) {
                    Ok(_) => {}
                    Err(e) => println!("{:?}", e), // Error on channel
                }
            }
            None => passthrough(data), // Data was not valid syntax || marked to be passed through, must be passed on to stdout
        }
    }
}

fn handle_caps(
    caps: Option<Captures>,
    timestamp: f64,
    ln: usize,
    settings: &ReaderSettings,
) -> Option<Point> {
    let vals = caps.unwrap(); //Guaranteed unwrap since captured by RegexSet

    let mut c = vals.iter();
    c.next(); // Dont want entire match
    let name = String::from(c.next().unwrap().unwrap().as_str()); //TODO: Guarantee unwraps

    //TODO: Filter out signals

    //convert remaining values to floats
    let v: Vec<f64> = c
        .map(|m| {
            m.map_or(0.0, |m| m.as_str().parse::<f64>().unwrap()) //TODO: should fail gracefully here, set channel health to bad
        })
        .collect();

    return Some(match v.len() {
        1 => Point {
            name,
            line_number: ln,
            timestamp,
            ty: PointType::D1,
            x: v[0],
            y: NAN,
            z: NAN,
        },
        2 => Point {
            name,
            line_number: ln,
            timestamp,
            ty: PointType::D2,
            x: v[0],
            y: v[1],
            z: NAN,
        },
        3 => Point {
            name,
            line_number: ln,
            timestamp,
            ty: PointType::D3,
            x: v[0],
            y: v[1],
            z: v[2],
        },
        _ => Point {
            name,
            line_number: ln,
            timestamp,
            ty: PointType::BreakPoint,
            x: NAN,
            y: NAN,
            z: NAN,
        },
    });
}

fn passthrough(line: &String) {
    // Log data for later examination if breakpoint is reached etc
    if let Ok(_) = write!(io::stdout(), "{}", line) {}
}
