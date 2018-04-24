//TODO: implement triad mixing and publish separately http://devmag.org.za/2012/07/29/how-to-choose-colours-procedurally-algorithms/ 

use std::fmt::Debug;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub type Color = (f32, f32, f32);

const GOLDEN_RATIO_CONJUGATE: f64 = 0.618033988749895;
#[derive(Debug)]
pub struct Generator {
    theta: f64,
    pub sat: f32,
    pub val: f32
}

fn get_angle<T: Debug>(arg: T) -> f32 {
    let s = format!("{:?}", arg);
    let mut hasher = DefaultHasher::new();
    for c in s.chars(){
		hasher.write_u8(c as u8);
	}
	let mut ang = hasher.finish() as f64;
    // println!("~.A@{:?}", ang as f32); 
	ang = ang / 10f64.powf(ang.log10().floor());
    ang -= ang.floor();//TODO without this ang tends heavily to less than 2
	// println!("~.C@{:?}", ang as f32);
	ang as f32
}

impl Generator{
	pub fn new() -> Generator{
		Generator{theta:0., sat: 0.9, val:0.9}
	}
	pub fn new_seed<T: Debug>(seed: T) -> Generator {
	    let mut g = Generator::new();
	    g.theta = get_angle(seed) as f64;
	    g
	}
	pub fn get_color<T: Debug>(seed: T, saturation: f32, value: f32) -> Color{
		Generator::angle_to_color(get_angle(seed), saturation, value)
	}
	fn angle_to_color(seed: f32, saturation: f32, value: f32) -> Color{
		hsv_to_rgb(360.*(seed as f32) , saturation, value)
	}
}

impl Iterator for Generator {
	type Item = Color;
    fn next(&mut self) -> Option<Color> {
        let c = Generator::angle_to_color(self.theta as f32, self.sat, self.val);
        self.theta = (self.theta + GOLDEN_RATIO_CONJUGATE) % 1.;
        return Some(c);
    }
}


fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color{
    let hi = ((h / 60.0) % 6.).floor();
    let f =  (h / 60.0) - (h / 60.0).floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - (f*s));
    let t = v * (1.0 - ((1.0 - f) * s));
    match hi as usize{
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => panic!("n mod 6 > 6")
    }
}

#[cfg(test)]
mod tests {
	use super::*;
    #[test]
    fn sanity() {
        for i in Generator::new_seed(0).take(100){
        	println!("{:?}", i);
        }
        println!("What color is this sentence? Its: {:?}", Generator::get_color("What color is this sentence?", 0.9, 0.9));
        assert!(true);
    }
}