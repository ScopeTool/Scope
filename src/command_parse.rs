// get suggestions given current buffer state, and parse buffer and set state
use drawstyles::*;
use signal::{AxisBind, SignalManager};

pub struct LineState {
    pub valid: bool,
    pub possible_completions: Vec<String>,
}

pub fn parse(line: &str, run: bool, manager: &mut SignalManager) -> LineState {
    let mut valid = false;
    let mut possible_completions = Vec::<String>::new();
    if line.len() > 1 {
        let cmd: &str = line.split_whitespace().take(1).collect::<Vec<&str>>()[0];
        valid = true;

        match cmd {
            "ds" | "drawstyle" => {
                drawstyle(line, run, &mut valid, &mut possible_completions, manager)
            }
            "s" | "select" => select(line, run, &mut valid, &mut possible_completions, manager),
            "b" | "bind" => bind(line, run, &mut valid, &mut possible_completions, manager),
            "f" | "free" => free(line, run, &mut valid, &mut possible_completions, manager),
            &_ => {
                if run {
                    println!("Invalid Command: {:?}", cmd)
                } else {
                    possible_completions.push(String::from("drawstyle"));
                    possible_completions.push(String::from("select"));
                    possible_completions.push(String::from("bind"));
                    possible_completions.push(String::from("free"));
                }
            }
        }
    }
    LineState {
        valid,
        possible_completions,
    }
}

fn select(
    cmd: &str,
    run: bool,
    valid: &mut bool,
    pc: &mut Vec<String>,
    manager: &mut SignalManager,
) {
    let bits = cmd.split_whitespace().collect::<Vec<&str>>();
    let sigs = select_signals(
        if bits.len() > 1 { &bits[1..] } else { &[] },
        valid,
        pc,
        manager,
    );
    if !*valid {
        return;
    }
    if run {
        for s in sigs {
            println!("Requested Selection: {:?}", s);
            manager.set_selection(Some(s));
        }
    }
}

fn drawstyle(
    cmd: &str,
    run: bool,
    valid: &mut bool,
    pc: &mut Vec<String>,
    manager: &mut SignalManager,
) {
    let bits = cmd.split_whitespace().collect::<Vec<&str>>();
    let mut t = Styles::Scatter;
    if bits.len() > 1 {
        match bits[1] {
            "scatter" => t = Styles::Scatter,
            "lines" => t = Styles::Lines,
            &_ => {
                *valid = false;
                pc.push(String::from("scatter"));
                pc.push(String::from("lines"));
                return;
            }
        }
    }
    // println!("{:?}", bits);
    let sigs = select_signals(
        if bits.len() > 2 { &bits[2..] } else { &[] },
        valid,
        pc,
        manager,
    );
    // println!("1 {:?}", *valid);
    if !*valid {
        return;
    }
    // println!("Selected: {:?}", sigs);
    if run {
        for s in sigs {
            manager
                .get_signal(&s)
                .expect(
                    "This vec can only consist of clones of the key strings from the signals map",
                )
                .set_style(&t);
            println!("Set {:?} to {:?}", s, t);
        }
    }
}

fn bind(cmd: &str, run: bool, valid: &mut bool, pc: &mut Vec<String>, manager: &mut SignalManager) {
    let bits = cmd.split_whitespace().collect::<Vec<&str>>();
    let mut mode = 0;
    if bits.len() > 1 {
        for c in bits[1].chars() {
            mode |= match c {
                '_' => AxisBind::None,
                'x' => AxisBind::X,
                'y' => AxisBind::Y,
                'z' => AxisBind::Z,
                _ => {
                    *valid = false;
                    pc.push(String::from("x"));
                    pc.push(String::from("y"));
                    pc.push(String::from("xy"));
                    pc.push(String::from("xyz"));
                    return;
                }
            } as u8
        }
    }

    let signals = select_signals(
        if bits.len() > 2 { &bits[2..] } else { &[] },
        valid,
        pc,
        manager,
    );
    if !*valid {
        return;
    }

    if run {
        println!("Bind mode: {:?}", mode);
        let mut sigs = signals.iter();
        if signals.len() > 1 {
            let base = sigs.next().expect("signals > 1 .'. next yields some");
            manager
                .get_signal(base)
                .expect(
                    "This vec can only consist of clones of the key strings from the signals map",
                )
                .set_bind_mode(mode);
            for i in sigs {
                manager.bind(base, i);
            }
        }
    }
}

fn free(cmd: &str, run: bool, valid: &mut bool, pc: &mut Vec<String>, manager: &mut SignalManager) {
    let bits = cmd.split_whitespace().collect::<Vec<&str>>();

    let signals = select_signals(
        if bits.len() > 1 { &bits[1..] } else { &[] },
        valid,
        pc,
        manager,
    );
    if !*valid {
        return;
    }

    if run {
        for i in signals.iter() {
            manager.free(i);
        }
    }
}

//TODO: take pc and provide
fn select_signals(
    bits: &[&str],
    _valid: &mut bool,
    pc: &mut Vec<String>,
    man: &SignalManager,
) -> Vec<String> {
    // println!("sig select: {:?}", bits);
    let mut rslt = Vec::new();
    // let bits = s.split_whitespace().collect::<Vec<&str>>();
    let mut inverse = false;
    if bits.len() == 0 {
        inverse = true;
    } else if bits.len() > 0 {
        if bits[0] == "!" {
            inverse = true;
        }
    }
    //TODO: catch misspellings and change validitity
    // println!("inverse {:?}", inverse);
    for name in man.get_names() {
        let mut good = inverse;
        for arg in bits {
            // println!("{:?} == {:?}", name, arg);
            if arg == name {
                good = !good;
                break;
            }
        }
        if bits.len() > 0 {
            pc.push(name.clone());
        }
        if good {
            rslt.push(name.clone());
        }
    }
    return rslt;
}
