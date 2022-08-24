extern crate distance;
extern crate glium;
extern crate glium_text_rusttype as glium_text;
use self::distance::damerau_levenshtein as fuzzy_dist;

use command_parse;

use std::f64::consts::PI;
use std::{cell::RefCell, rc::Rc};

use glium::{Display, Frame, Rect, Surface};

use glimput::Editor;

use glium::glutin::event::{self, KeyboardInput, VirtualKeyCode as VKC};

use drawstyles::Transform;
use signal::{SignalHealth, SignalManager};

type Color = (f32, f32, f32, f32);
const DARK_GREY: Color = (0.01, 0.01, 0.01, 1.0);

#[derive(Debug)]
struct DataCursor {
    pos: (f64, f64),
    signal: Option<String>,
}

impl DataCursor {
    fn new() -> DataCursor {
        DataCursor {
            pos: (0., 0.),
            signal: None,
        }
    }
}

pub struct UI<'a> {
    pub signal_manager: SignalManager<'a>,
    editor: Editor,
    window_size: (u32, u32),
    ledgend_width: f32,
    axis_width: f64,
    text_height: f32,
    text_system: glium_text::TextSystem,
    text_format: RefCell<glium_text::TextDisplay<Rc<glium_text::FontTexture>>>,
    cmdline_completions: Vec<String>,
    completion_idx: usize,
    cursor: DataCursor,
    hover_rad: f64,
    last_mouse_pos: (f64, f64),
    cursor2: Option<DataCursor>,
    working_area: (f64, f64, f64, f64),
    lmb_pressed: bool,
    hidpi_factor: f64,
}

impl<'a> UI<'a> {
    pub fn new(display: &Display) -> UI {
        let system = glium_text::TextSystem::new(display);
        //TODO: figure out the proper way to do this, dont need rc just need to tell rust to alocate this on heap, destroy it with UI and pass refrence to text
        let font = Rc::new(
            glium_text::FontTexture::new(
                display,
                &include_bytes!("../resources/UbuntuMono-R.ttf")[..],
                100,
                glium_text::FontTexture::ascii_character_list(),
            )
            .unwrap(),
        );
        let signal_manager = SignalManager::new(display);
        let editor = Editor::new();
        let text = glium_text::TextDisplay::new(&system, font.clone(), "Aj");
        let text_height = text.get_height() * 1.3;
        UI {
            signal_manager,
            editor,
            window_size: (0, 0),
            ledgend_width: 0.2,
            text_height,
            text_system: system,
            text_format: RefCell::new(text),
            axis_width: 0.,
            cmdline_completions: Vec::new(),
            completion_idx: 0,
            cursor: DataCursor::new(),
            cursor2: None,
            hover_rad: 0.03,
            last_mouse_pos: (0., 0.),
            working_area: (0., 0., 0., 0.),
            lmb_pressed: false,
            hidpi_factor: 1.0,
        }
    }
    pub fn draw(
        &mut self,
        target: &mut Frame,
        hidpi_factor: f64,
        window_size: (u32, u32),
        frametime: f64,
    ) {
        self.hidpi_factor = hidpi_factor;
        self.window_size = window_size; //.to_physical(hidpi_factor).into();
        self.draw_text(
            target,
            -0.98,
            0.97,
            0.04,
            (1.0, 1.0, 1.0, 1.0),
            &frametime.floor().to_string(),
        );

        // self.debug_perf(frametime);

        let view_start_x = -1.0 + self.get_axis_width();
        let view_start_y = -1.0 + self.get_axis_height() + self.get_cmd_height();
        let view_end_x = 1.0 - self.get_log_width() - self.get_ledgend_width();
        let view_end_y = 1.0 - self.get_axis_height();

        let area = (view_start_x, view_start_y, view_end_x, view_end_y);
        self.working_area = area;

        let mut sel = self.signal_manager.get_selection();
        if sel.is_none() {
            let sel = self.signal_manager.iter().next().map(|(s, _)| s.to_owned());
            self.signal_manager.set_selection(sel);
        }

        self.signal_manager.draw_signals(target, area);

        self.draw_cursors(target, area);

        self.draw_cmdline(target, area);

        self.draw_ledgend(target, view_end_x);
    }

    fn draw_cursors(&mut self, target: &mut Frame, area: (f64, f64, f64, f64)) {
        let scale = 0.06;
        let mut xlables = 0.;
        let mut ylables = (scale * self.text_height / 2.0) as f64;

        // let corners = vec![(area.0, area.1, (0.,1.)), (area.2, area.1, (-1.,0.)), (area.2, area.3, (0.,-1.)), (area.0, area.3, (1., 0.))];
        // let av = corners.iter().fold((0., 1.), |acc, &pt|{
        // 	let a = (pt.0 - self.cursor.pos.0).powf(2.);
        // 	let b = (pt.1 - self.cursor.pos.1).powf(2.);
        // 	let d = (a + b).sqrt() / ((area.2 - area.0).powf(2.) + (area.3-area.1).powf(2.)).sqrt();
        // 	let w = 3.*d.powf(2.) - 2.*d.powf(3.);
        // 	(acc.0 + (pt.2).0*w, acc.1 + (pt.2).1*w)
        // });
        // let itheta = PI+ 0.75*PI;//av.1.atan2(av.0);
        let itheta = PI / 2.0;
        let mut theta: f64 = itheta;
        if self.cursor.signal.is_some() {
            self.draw_cursor(target, &self.cursor);
            let pad = 0.01;
            let mut axis_width = 0.125f64;
            for (_name, sig) in self.signal_manager.iter() {
                if let Some(pick) =
                    sig.pick((self.cursor.pos.0 as f32, self.cursor.pos.1 as f32), area)
                {
                    let c = sig.get_color();
                    let color = (c.0, c.1, c.2, 1.0);
                    let (xtext, ytext, ztext) = sig.get_point_strings(pick.index);
                    let dims = self.get_text_dims(scale, &ztext);
                    let sign = if theta.cos() > 0. { -1. } else { 1. };
                    // let x = pick.screen_pos.0 as f64 + theta.cos()*self.hover_rad;
                    // let y = pick.screen_pos.1 as f64 + theta.sin()*self.hover_rad;
                    let mut x = self.cursor.pos.0 as f64 + theta.cos() * self.hover_rad;
                    if sign > 0.0 {
                        x -= self.get_text_dims(scale, &ztext).0;
                    }
                    let y = self.cursor.pos.1 as f64 + theta.sin() * self.hover_rad;
                    self.draw_rect(
                        target,
                        DARK_GREY,
                        (
                            x - pad / 2.,
                            y - pad / 2. - scale as f64 * self.text_height as f64 / 2.,
                        ),
                        (dims.0 + pad, dims.1 + pad),
                    );
                    self.draw_text(target, x, y, scale, color, &ztext);

                    let dy = (self.text_height as f64 * scale as f64 + pad) * sign; // height of textbox
                    let ny = theta.sin() * self.hover_rad + dy; // next height around circle
                    let c2 = self.hover_rad.powf(2.);
                    let a2 = ny.powf(2.);
                    if c2 > a2 {
                        // if point can be on circle
                        let nx = (c2 - a2).sqrt() * sign * -1.0; // find next x position that corresponds to desired y on circle
                        theta = ny.atan2(nx); // get corresponding angle
                    } else {
                        theta = -PI / 2. - (PI / 2. - theta.abs()); // jump to other side of circle
                    }

                    let mut wd = self
                        .draw_text(
                            target,
                            self.cursor.pos.0 + pad + xlables,
                            1.0 - self.text_height as f64 / 2.0 * scale as f64,
                            scale,
                            color,
                            &xtext,
                        )
                        .0;
                    xlables += wd + pad;
                    wd = 1.2
                        * self
                            .draw_text(
                                target,
                                -1.0 + pad,
                                self.cursor.pos.1 + pad + ylables,
                                scale,
                                color,
                                &ytext,
                            )
                            .0;
                    axis_width = axis_width.max(wd);
                    ylables += (self.text_height * scale) as f64 + pad;
                }
            }

            self.axis_width = self.axis_width * 0.9 + 0.1 * axis_width;

            theta -= itheta; // theta relative to initial pos
            if theta > 0. {
                // clamp to negative range
                theta -= 2. * PI;
            }
            let target = -PI / 2.; // set our target angle to be 90 deg
            theta = target - theta; // find distance to target

            self.hover_rad += (theta) * 0.08; // proportional control
            self.hover_rad = self.hover_rad.min(0.6);
            self.hover_rad = self.hover_rad.max(0.0001);
        }
        // Draw second cursor if exists and draw rulers
    }

    fn draw_cursor(&self, target: &mut Frame, cursor: &DataCursor) {
        self.draw_rect_px(
            target,
            (1., 1., 1., 1.),
            (cursor.pos.0, -1.),
            (1, self.window_size.1),
        );
        self.draw_rect_px(
            target,
            (1., 1., 1., 1.),
            (-1., cursor.pos.1),
            (self.window_size.0, 1),
        );
    }

    pub fn send_key(&mut self, c: char) {
        if c != '\t' && c != '\r' {
            self.editor.send_key(c);
            self.update_editor();
        }
    }

    pub fn send_event(&mut self, input: &KeyboardInput) {
        if let Some(k) = input.virtual_keycode {
            if input.state == glium::glutin::event::ElementState::Pressed {
                match k {
                    VKC::Tab if input.modifiers.shift() && self.cmdline_completions.len() > 0 => {
                        self.completion_idx =
                            (self.completion_idx + 1) % self.cmdline_completions.len();
                    }
                    VKC::Tab => {
                        let mut cmpl = None;
                        if let Some(c) = self.get_completion() {
                            cmpl = Some(String::from(c));
                        }
                        if let Some(cmpl) = cmpl {
                            self.editor.autofill(&cmpl)
                        }
                        //TODO: there has to be a better way to do this
                    }
                    VKC::Return => {
                        let rslt = command_parse::parse(
                            self.editor.get_buffer(),
                            true,
                            &mut self.signal_manager,
                        );
                        if rslt.valid {
                            self.editor.clear();
                            self.cmdline_completions.clear();
                            self.completion_idx = 0;
                        }
                    }
                    _ => (),
                }
            }
        }
        self.editor.send_event(input);
    }

    pub fn send_mouse(&mut self, event: &glium::glutin::event::WindowEvent) {
        match event {
            glium::glutin::event::WindowEvent::MouseWheel {
                delta,
                phase: _,
                modifiers: _,
                ..
            } => {
                if let Some(sig) = self.signal_manager.get_selected() {
                    if let glium::glutin::event::MouseScrollDelta::LineDelta(_, y) = delta {
                        sig.zoom_by(
                            *y as f64,
                            (
                                self.working_area.2 - self.last_mouse_pos.0,
                                (self.working_area.3
                                    - self.last_mouse_pos.1
                                    - (self.working_area.3 - self.working_area.1) / 2.),
                            ),
                        );
                    }
                }
            }
            event::WindowEvent::MouseInput {
                state,
                button,
                modifiers: _,
                ..
            } => {
                if button == &event::MouseButton::Left {
                    self.lmb_pressed = state == &event::ElementState::Pressed;
                }
            }
            event::WindowEvent::CursorMoved { position, .. } => {
                if let Some(sig) = self.signal_manager.get_selected() {
                    self.last_mouse_pos = self.cursor.pos;
                    self.cursor.pos = (
                        (2. * (position.x * self.hidpi_factor / (self.window_size.0 as f64)) - 1.),
                        (1. - 2. * (position.y * self.hidpi_factor / (self.window_size.1 as f64))),
                    );
                    self.cursor.signal = Some(sig.get_name().clone());
                    if self.lmb_pressed {
                        let delta = (
                            self.cursor.pos.0 - self.last_mouse_pos.0,
                            self.cursor.pos.1 - self.last_mouse_pos.1,
                        );
                        sig.move_view_by(delta, self.working_area)
                    }
                }
            }
            _ => {}
        }
    }

    fn update_editor(&mut self) {
        let rslt = command_parse::parse(self.editor.get_buffer(), false, &mut self.signal_manager);
        self.cmdline_completions = rslt.possible_completions;
        let current_term = &self.editor.get_working_term();
        self.cmdline_completions
            .sort_by(|s1, s2| fuzzy_dist(current_term, s1).cmp(&fuzzy_dist(current_term, s2)));
        self.completion_idx = 0;
    }

    fn draw_cmdline(&mut self, target: &mut Frame, area: (f64, f64, f64, f64)) {
        let mut rhs = area.0;
        let cmd_com_y = (area.1 - 1.0) / 2.0;
        let cmd_height = self.get_cmd_height() * 0.95;
        let ypos = cmd_com_y - cmd_height / 2.0;
        self.draw_rect(
            target,
            DARK_GREY,
            (rhs, ypos),
            ((area.2 - area.0), cmd_height),
        );
        let scale = (cmd_height as f32) * 0.65 / (self.text_height);
        let color = if let Some(s) = self.signal_manager.get_selected() {
            let c = s.get_color();
            (c.0, c.1, c.2, 1.0)
        } else {
            (0.8, 0.2, 0.1, 1.0)
        };

        let (first, c, rest) = self.editor.get_buffer_parts();
        let (last, _) = self.draw_text(target, rhs, cmd_com_y, scale, (1., 1., 1., 1.0), first);
        rhs += last;
        let (last, _) = self.draw_text(target, rhs, cmd_com_y, scale, color, c);
        rhs += last;
        let (last, _) = self.draw_text(target, rhs, cmd_com_y, scale, (1., 1., 1., 1.0), rest);
        if let Some(cmpl) = self.get_completion() {
            rhs += last;
            let mut text_dims = self.get_text_dims(scale, cmpl);
            text_dims = (text_dims.0 * 1.1, text_dims.1 * 1.1);
            self.draw_rect(target, DARK_GREY, (rhs, ypos + cmd_height), text_dims);
            self.draw_text(
                target,
                rhs,
                ypos + cmd_height + text_dims.1 / 2.,
                scale,
                (0.5, 0.5, 0.5, 1.0),
                cmpl,
            );
        }
    }

    fn get_completion(&self) -> Option<&str> {
        if self.cmdline_completions.len() > 0 {
            return Some(&self.cmdline_completions[self.completion_idx]);
        }
        return None;
    }

    fn draw_ledgend(&mut self, target: &mut Frame, view_end_x: f64) {
        let scale = 0.08;
        let th = self.text_height * scale;
        let mut pos = 1.0 - self.get_axis_height() - th as f64;
        let mut max_width = 0.0f32;
        let stat_width = (th * self.resquare()) as f64;
        self.draw_rect(
            target,
            DARK_GREY,
            (
                view_end_x + 0.04,
                1.0 - self.get_axis_height() - th as f64 * self.signal_manager.len() as f64 - 0.01,
            ),
            (
                self.ledgend_width as f64 - 0.06,
                th as f64 * self.signal_manager.len() as f64 + 0.02,
            ),
        );
        let sel = self.signal_manager.get_selection().clone();
        for (name, sig) in self.signal_manager.iter() {
            let c = sig.get_color();
            let ts = view_end_x + 0.05;
            if let Some(ref n) = sel {
                if n == name {
                    //TODO: is there a cleaner way?
                    self.draw_rect(
                        target,
                        (1., 1., 1., 1.),
                        (ts as f64 - stat_width * 0.18, pos as f64),
                        (stat_width * 0.18, th as f64),
                    );
                }
            }
            self.draw_rect(
                target,
                match sig.get_health() {
                    SignalHealth::Good => (62.0 / 256.0, 107.0 / 256.0, 12.0 / 256.0, 1.),
                    SignalHealth::InvalidFormat => (1., 0., 0., 1.),
                },
                (ts as f64, pos as f64),
                (stat_width, th as f64),
            );
            let (tw, _) = self.draw_text(
                target,
                ts + stat_width,
                pos + th as f64 / 2.,
                scale,
                (c.0, c.1, c.2, 1.0),
                &name,
            );
            max_width = max_width.max(tw as f32);
            pos -= th as f64;
        }
        self.ledgend_width = max_width + 0.08 + stat_width as f32;
    }

    // Input in 3D ogl space
    fn draw_rect(
        &self,
        target: &mut glium::Frame,
        color: Color,
        corner: (f64, f64),
        dims: (f64, f64),
    ) {
        let pxx = self.window_size.0 as f64 / 2.0;
        let pxy = self.window_size.1 as f64 / 2.0;
        let width = (dims.0 * pxx) as u32;
        let height = (dims.1 * pxy) as u32;
        self.draw_rect_px(target, color, corner, (width, height))
    }

    fn draw_rect_px(
        &self,
        target: &mut glium::Frame,
        color: Color,
        corner: (f64, f64),
        dims: (u32, u32),
    ) {
        let pxx = self.window_size.0 as f64 / 2.0;
        let pxy = self.window_size.1 as f64 / 2.0;
        let cornerx = (self.window_size.0 / 2) as i32 + (pxx * corner.0) as i32;
        let cornery = (self.window_size.1 / 2) as i32 + (pxy * corner.1) as i32;
        target.clear(
            Some(&Rect {
                left: cornerx as u32,
                bottom: cornery as u32,
                width: dims.0,
                height: dims.1,
            }),
            Some(color),
            false,
            None,
            None,
        );
    }

    fn draw_text(
        &self,
        target: &mut glium::Frame,
        x: f64,
        y: f64,
        scale: f32,
        color: Color,
        text: &str,
    ) -> (f64, f64) {
        let trans = Transform {
            dx: x as f32,
            dy: y as f32 - self.text_height * scale / 2.9,
            sx: scale * self.resquare(),
            sy: scale,
            sz: 1.,
        };
        let mut tf = self.text_format.borrow_mut();
        tf.set_text(text);
        glium_text::draw(&tf, &self.text_system, target, &trans.into(), color).unwrap();
        (
            (tf.get_width() * trans.sx) as f64,
            (tf.get_height() * trans.sy * 1.3) as f64,
        )
    }

    fn get_text_dims(&self, scale: f32, text: &str) -> (f64, f64) {
        let mut tf = self.text_format.borrow_mut();
        tf.set_text(text);
        (
            (tf.get_width() * scale * self.resquare()) as f64,
            (tf.get_height() * scale * 1.3) as f64,
        )
    }

    fn get_log_width(&self) -> f64 {
        0.0
    }

    fn get_ledgend_width(&self) -> f64 {
        self.ledgend_width as f64
    }

    fn get_cmd_height(&self) -> f64 {
        0.1
    }

    fn get_axis_width(&self) -> f64 {
        self.axis_width
    }

    fn get_axis_height(&self) -> f64 {
        0.05
    }

    fn resquare(&self) -> f32 {
        self.window_size.1 as f32 / self.window_size.0 as f32
    }

    fn debug_perf(&self, frametime: f64) {
        println!("~.{}@{:?}", "frametime", frametime);
        println!("~.{}@{:?}", "points", self.signal_manager.point_count);
        println!(
            "~.{}@{:?}",
            "performance/pt",
            frametime / self.signal_manager.point_count as f64
        );
    }
}
