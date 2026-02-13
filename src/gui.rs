use crate::*;
use std::sync::{Arc, Mutex};

use sdl2::{
    event::Event,
    keyboard::Scancode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    surface::Surface,
    render::SurfaceCanvas,
    ttf::Font,
    gfx::primitives::DrawRenderer,
};

use std::time::Duration;

struct ModuleWindow {
    // TODO change to i16?
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    title: &'static str,
    inputs: Vec<(DataType, &'static str)>,
}

impl ModuleWindow {
    fn new(title: &'static str) -> Self {
        Self {
            x: 50,
            y: 50,
            width: 200,
            height: 200,
            title,
            inputs: Vec::new(),
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }

    fn render(&self, font: &Font) -> SurfaceCanvas<'_> {
        let mut canvas =
            Surface::new(self.width, self.height,
                PixelFormatEnum::RGBA32).unwrap()
            .into_canvas().unwrap();

        let rect = canvas.surface().rect();

        canvas.set_draw_color(Color::RGB(200, 200, 200));
        canvas.clear();

        let rendered_title = font.render(self.title).solid(Color::RGB(0, 0, 0)).unwrap();
        rendered_title.blit(rendered_title.rect(), canvas.surface_mut(), Rect::new(
            ((rect.width() / 2).saturating_sub(rendered_title.width() / 2)) as i32,
            0,
            rect.width(),
            rendered_title.height()
        )).unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.draw_rect(rect).unwrap();
        // canvas.draw_line((0, rendered_title.height() as i32), (rect.width() as i32, rendered_title.height() as i32));

        canvas
    }

    fn output_conn(&self) -> (i32, i32) {
        (self.x + self.width as i32, self.y + 20)
    }

    fn input_conns(&self) -> Vec<(i32, i32)> {
        (0..self.inputs.len()).map(|i| (self.x, self.y + 20 + 20 * i as i32)).collect()
    }
}

pub struct Gui {
    modules: HashMap<ModuleId, ModuleWindow>,
}

enum Selection {
    Window(u16, Rect, i32, i32),
    Output(ModuleId),
    Input(ModuleId, usize),
}

impl Gui {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    fn get_selected_conn(&self, x: i32, y: i32) -> Option<Selection> {
        for (i, module) in self.modules.iter() {
            let (cx, cy) = module.output_conn();
            if cx - 10 < x && x < cx + 10 && cy - 10 < y && y < cy + 10 {
                return Some(Selection::Output(*i));
            }

            for (conn_id, (cx, cy)) in module.input_conns().iter().enumerate() {
                if cx - 10 < x && x < cx + 10 && cy - 10 < y && y < cy + 10 {
                    return Some(Selection::Input(*i, conn_id));
                }
            }

            let rect = module.rect();
            if rect.contains_point((x, y)) {
                return Some(Selection::Window(i.clone(), rect, x, y));
            }
        }

        None
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

        let font = ttf_context.load_font("/usr/share/fonts/gnu-free/FreeMono.otf", 16).unwrap();

        let mut output_win = ModuleWindow::new("Output");
        output_win.inputs = vec![(DataType::Audio, "")];
        self.modules.insert(0, output_win);

        {
            let app = app.lock().unwrap();
            for (id, module) in app.modules.iter() {
                self.modules.insert(*id, ModuleWindow::new(module.title()));
            }
        }

        let mut selection = None;

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::MouseButtonDown { x, y, .. } => {
                        selection = self.get_selected_conn(x, y);
                    }
                    Event::MouseButtonUp { x, y, .. } => {
                        let mut app = app.lock().unwrap();
                        let new_selection = self.get_selected_conn(x, y);
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
                    _ => {}
                }
            }

            let keyboard = event_pump.keyboard_state();
            let mouse = event_pump.mouse_state();

            canvas.set_draw_color(Color::RGB(150, 180, 190));
            canvas.clear();

            match selection {
                Some(Selection::Window(module_id, start_rect, mx, my)) => {
                    if !keyboard.is_scancode_pressed(Scancode::LCtrl) {
                        selection = None;

                    } else {
                        let module = self.modules.get_mut(&module_id).unwrap();

                        let dx = mouse.x().saturating_sub(mx);
                        let dy = mouse.y().saturating_sub(my);

                        if mouse.left() {
                            module.x = start_rect.x() + dx;
                            module.y = start_rect.y() + dy;

                        } else if mouse.right() {
                            module.width  = i32::max(50, start_rect.width()  as i32 + dx) as u32;
                            module.height = i32::max(50, start_rect.height() as i32 + dy) as u32;

                        } else {
                            selection = None;
                        }
                    }
                }
                Some(Selection::Output(mod_id)) => {
                    if mouse.left() {
                        let output_conn = self.modules[&mod_id].output_conn();
                        canvas.set_draw_color(Color::RGB(255, 0, 0));
                        canvas.draw_line(output_conn, (mouse.x(), mouse.y())).unwrap();
                    } else {
                        selection = None;
                    }
                }
                Some(Selection::Input(mod_id, conn_id)) => {
                    if mouse.left() {
                        let input_conn = self.modules[&mod_id].input_conns()[conn_id];
                        canvas.set_draw_color(Color::RGB(255, 0, 0));
                        canvas.draw_line(input_conn, (mouse.x(), mouse.y())).unwrap();
                    } else {
                        selection = None;
                    }
                }
                _ => {}
            }

            {
                let app = app.lock().unwrap();

                // TODO update this less often
                for (i, module) in app.modules.iter() {
                    self.modules.get_mut(i).unwrap().inputs =
                        module.get_inputs().iter().map(|i| (i.0.clone(), i.1.into())).collect();
                }

                canvas.set_draw_color(Color::RGB(255, 0, 0));
                for ((input_id, conn_id), output_id) in &app.conns {
                    let inputs = self.modules[&input_id].input_conns();
                    canvas.draw_line(inputs[*conn_id], self.modules[&output_id].output_conn()).unwrap();
                }
            }

            for (i, module_win) in self.modules.iter() {
                let surface = module_win.render(&font).into_surface();
                let texture = surface.as_texture(&texture_creator).unwrap();
                canvas.copy(&texture, surface.rect(), module_win.rect()).unwrap();

                for input in module_win.input_conns() {
                    canvas.filled_circle(input.0 as i16, input.1 as i16, 5, Color::RGB(255, 0, 0)).unwrap();
                }

                if *i != 0 {
                    let output = module_win.output_conn();
                    canvas.filled_circle(output.0 as i16, output.1 as i16, 5, Color::RGB(255, 0, 0)).unwrap();
                }
            }

            canvas.present();
            std::thread::sleep(Duration::from_millis(1000 / 60));
        }
    }
}
