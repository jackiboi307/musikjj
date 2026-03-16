use crate::*;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Serialize, Deserialize};

use sdl2::{
    event::Event,
    keyboard::Scancode,
    mouse::MouseButton,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    surface::Surface,
    gfx::primitives::DrawRenderer,
};

// TODO move these constants into ui_utils::UiContext

const COLOR_BG: Color = Color::RGB(150, 180, 190);
const COLOR_WIN_BG: Color = Color::RGB(200, 200, 200);
const COLOR_BORDER: Color = Color::RGB(80, 100, 120);
const COLOR_BORDER_SEL: Color = Color::RGB(0, 0, 0);
const COLOR_CONN: Color = Color::RGB(255, 0, 0);
const COLOR_TEXT: Color = Color::RGB(0, 0, 0);

const DEFAULT_WIN_SIZE: u32 = 160;
const WIN_PADDING: u8 = 20;
const WIN_PADDING_TOP: u8 = 10; // extra top padding

#[derive(Clone, Serialize, Deserialize)]
pub struct ModuleWindow {
    // TODO change to i16?
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    title: Box<str>,
    inputs: Vec<(DataType, Box<str>)>,
}

impl ModuleWindow {
    fn new(title: &'static str) -> Self {
        Self {
            x: 50,
            y: 50,
            width: DEFAULT_WIN_SIZE,
            height: DEFAULT_WIN_SIZE,
            title: title.into(),
            inputs: Vec::new(),
        }
    }

    fn padded_size(&self) -> (u32, u32) {
        (self.width + WIN_PADDING as u32 * 2, self.height + WIN_PADDING as u32 * 2 + WIN_PADDING_TOP as u32)
    }

    fn rect(&self) -> Rect {
        Rect::new(
            self.x,
            self.y,
            self.width,
            self.height,
        )
    }

    fn padded_rect(&self) -> Rect {
        let (width, height) = self.padded_size();
        Rect::new(
            self.x,
            self.y,
            width,
            height,
        )
    }

    fn output_conn(&self) -> (i32, i32) {
        let (_, height) = self.padded_size();
        (self.x + self.padded_size().0 as i32, self.y + height as i32 / 2)
    }

    fn input_conns(&self) -> Vec<(i32, i32)> {
        const SPACING: i32 = 20;
        let (_, height) = self.padded_size();
        let len = self.inputs.len() as i32;
        let y = self.y + height as i32 / 2 - SPACING * (len / 2);
        (0..len).map(|i| (self.x, y + SPACING * i as i32)).collect()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Gui {
    pub modules: Vec<(ModuleId, ModuleWindow)>,
    selected: ModuleId,
    x: i32,
    y: i32,
}

#[derive(Clone, Debug)]
enum Selection {
    Window(ModuleId),
    Output(ModuleId),
    Input(ModuleId, usize),
}

impl Gui {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            selected: 0,
            x: 0,
            y: 0,
        }
    }

    pub fn module(&self, id: ModuleId) -> &ModuleWindow {
        for (iter_id, module) in self.modules.iter() {
            if id == *iter_id {
                return module
            }
        }
        panic!()
    }

    pub fn module_mut(&mut self, id: ModuleId) -> &mut ModuleWindow {
        for (iter_id, module) in self.modules.iter_mut() {
            if id == *iter_id {
                return module
            }
        }
        panic!()
    }

    fn check_selected(&mut self, x: i32, y: i32) -> Option<Selection> {
        // selection box size
        const SEL: i32 = 20;

        let x = x - self.x;
        let y = y - self.y;

        for (id, module) in self.modules.iter().rev() {
            let id = *id;
            let (cx, cy) = module.output_conn();

            if cx - SEL < x && x < cx + SEL && cy - SEL < y && y < cy + SEL {
                // self.selected = id;
                return Some(Selection::Output(id));
            }

            for (conn_id, (cx, cy)) in module.input_conns().iter().enumerate() {
                if cx - SEL < x && x < cx + SEL && cy - SEL < y && y < cy + SEL {
                    // self.selected = id;
                    return Some(Selection::Input(id, conn_id));
                }
            }

            if module.padded_rect().contains_point((x, y)) {
                self.selected = id;
                return Some(Selection::Window(id.clone()));
            }
        }

        None
    }

    pub fn insert_module(&mut self, id: ModuleId, module: &Box<dyn Module + Send>) {
        self.modules.push((id, ModuleWindow::new(module.title())));
    }

    pub fn init(&mut self) {
        let mut output_win = ModuleWindow::new("Output");
        output_win.inputs = vec![(DataType::Audio, "".into())];
        self.modules.push((0, output_win));
    }

    pub fn run(&mut self, app: Arc<Mutex<App>>) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("musikjj", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let texture_creator = canvas.texture_creator();
        let ttf_context = sdl2::ttf::init().unwrap();

        let font = ttf_context.load_font("assets/FreeMono.otf", 16).unwrap();
        let ui_context = UiContext {
            font,
        };

        let mut selection = None;

        'running: loop {
            let mut clicked_mouse_btn = None;
            let keyboard = event_pump.keyboard_state();
            let lctrl = keyboard.is_scancode_pressed(Scancode::LCtrl);

            // handle events, window selection and connecting modules
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                        if !(mouse_btn == MouseButton::Middle && lctrl) {
                            clicked_mouse_btn = Some(mouse_btn);
                            selection = self.check_selected(x, y);

                            let mut app = app.lock().unwrap();
                            app.set_selection(selection.clone().map(|_| self.selected));

                            // put the selected window in the front

                            let mut index = None;
                            for (i, id) in self.modules.iter()
                                    .enumerate().map(|(i, (id, _))| (i, id)) {
                                if *id == self.selected {
                                    index = Some(i);
                                }
                            }

                            if let Some(index) = index {
                                let old = self.modules.remove(index);
                                self.modules.push(old);
                            }
                        }
                    }
                    Event::MouseButtonUp { x, y, mouse_btn, .. } => {
                        if !(mouse_btn == MouseButton::Middle && lctrl) {
                            let mut app = app.lock().unwrap();
                            let new_selection = self.check_selected(x, y);

                            match new_selection {
                                Some(Selection::Output(out_id)) => {
                                    match selection {
                                        Some(Selection::Input(in_id, conn_id)) => {
                                            app.connect(out_id, (in_id, conn_id));
                                        }
                                        _ => {}
                                    }
                                }
                                Some(Selection::Input(in_id, conn_id)) => {
                                    match selection {
                                        Some(Selection::Output(out_id)) => {
                                            app.connect(out_id, (in_id, conn_id));
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            let mouse = event_pump.mouse_state();
            let delta_mouse = event_pump.relative_mouse_state();
            let (dx, dy) = (delta_mouse.x(), delta_mouse.y());

            if lctrl && mouse.middle() {
                self.x += dx;
                self.y += dy;
            }

            if selection.is_some() && mouse.left() {
                if let Some(Selection::Window(id)) = selection {
                    if lctrl {
                        let module = self.module_mut(id);
                        module.x += dx;
                        module.y += dy;
                    } else {
                        selection = None;
                    }
                }
            } else {
                selection = None;
            }

            canvas.set_draw_color(COLOR_BG);
            canvas.clear();

            // TODO see if this can be removed
            let mut pending_conn_line: Option<((i32, i32), (i32, i32))> = None;

            match selection {
                Some(Selection::Output(mod_id)) => {
                    let output_conn = self.module(mod_id).output_conn();
                    let output_conn = (output_conn.0 + self.x, output_conn.1 + self.y);
                    pending_conn_line = Some((output_conn, (mouse.x(), mouse.y())));
                }
                Some(Selection::Input(mod_id, conn_id)) => {
                    let input_conn = self.module(mod_id).input_conns()[conn_id];
                    let input_conn = (input_conn.0 + self.x, input_conn.1 + self.y);
                    pending_conn_line = Some((input_conn, (mouse.x(), mouse.y())));
                }
                _ => {}
            }

            // draw windows / modules
            for (id, module_win) in self.modules.iter_mut() {
                // draw the window

                let (width, height) = module_win.padded_size();

                let mut mod_canvas =
                    Surface::new(width, height, PixelFormatEnum::RGBA32).unwrap()
                    .into_canvas().unwrap();

                mod_canvas.set_draw_color(COLOR_WIN_BG);
                mod_canvas.clear();

                let rendered_title = ui_context.font.render(&module_win.title).solid(COLOR_TEXT).unwrap();
                rendered_title.blit(rendered_title.rect(), mod_canvas.surface_mut(), Rect::new(
                    ((width / 2).saturating_sub(rendered_title.width() / 2)) as i32,
                    WIN_PADDING_TOP as i32 / 2,
                    width,
                    rendered_title.height()
                )).unwrap();

                mod_canvas.set_draw_color(if *id == self.selected {
                    COLOR_BORDER_SEL
                } else {
                    COLOR_BORDER
                });
                mod_canvas.draw_rect(Rect::new(0, 0, width, height)).unwrap();

                let surface = mod_canvas.into_surface();
                let texture = surface.as_texture(&texture_creator).unwrap();
                canvas.copy(&texture, surface.rect(),
                    module_win.padded_rect().right_shifted(self.x).bottom_shifted(self.y)
                ).unwrap();

                // draw input connections
                for input in module_win.input_conns() {
                    canvas.filled_circle(
                        (input.0 + self.x) as i16,
                        (input.1 + self.y) as i16,
                        5, COLOR_CONN
                    ).unwrap();
                }

                if *id != 0 {
                    // draw output connections
                    let output = module_win.output_conn();
                    canvas.filled_circle(
                        (output.0 + self.x) as i16,
                        (output.1 + self.y) as i16,
                        5, COLOR_CONN
                    ).unwrap();

                    let mut app = app.lock().unwrap();

                    // do Module::draw

                    let interact = if self.selected == *id && selection.is_none() {
                        let x = mouse.x() - self.x - module_win.x - WIN_PADDING as i32;
                        let y = mouse.y() - self.y - module_win.y - WIN_PADDING as i32 - WIN_PADDING_TOP as i32;

                        if 0 <= x && x < module_win.width as i32
                            && 0 <= y && y < module_win.height as i32 {

                            Some(ModuleInteractInfo {
                                x: x as u16,
                                y: y as u16,
                                click: clicked_mouse_btn,
                                event_pump: &event_pump,
                            })

                        } else { None }
                    } else { None };

                    if let Some(surface) = app.module(*id).draw(&ui_context, interact) {
                        let texture = surface.as_texture(&texture_creator).unwrap();
                        canvas.copy(
                            &texture,
                            surface.rect(),
                            module_win.rect()
                                .right_shifted(WIN_PADDING as i32 + self.x)
                                .bottom_shifted(WIN_PADDING as i32 + WIN_PADDING_TOP as i32 + self.y)
                        ).unwrap();
                        module_win.width = surface.rect().width();
                        module_win.height = surface.rect().height();
                    }
                }
            }

            // draw pending connection
            if let Some((start, end)) = pending_conn_line {
                canvas.set_draw_color(COLOR_CONN);
                canvas.draw_line(start, end).unwrap();
            }

            // draw present connections
            {
                let app = app.lock().unwrap();

                // TODO update this less often
                for (i, module) in app.modules.iter() {
                    self.module_mut(*i).inputs =
                        module.get_inputs().iter().map(|i| (i.0.clone(), i.1.into())).collect();
                }

                canvas.set_draw_color(COLOR_CONN);
                for ((input_id, conn_id), output_id) in &app.conns {
                    let inputs = self.module(*input_id).input_conns();
                    let input = inputs[*conn_id];
                    let output = self.module(*output_id).output_conn();
                    canvas.draw_line(
                        (input.0 + self.x, input.1 + self.y),
                        (output.0 + self.x, output.1 + self.y)
                    ).unwrap();
                }
            }

            canvas.present();
            std::thread::sleep(Duration::from_millis(1000 / 60));
        }
    }
}
