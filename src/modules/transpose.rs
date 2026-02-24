use crate::*;

macro_rules! add_btn {
    ($self:ident, $ui:ident, $canvas:ident, $layout:ident, $interact:ident,
        $amount:literal, $width:expr) => {

        if $ui.add_button(&mut $canvas, &mut $layout, &$interact, stringify!($amount), Some($width)) {
            $self.amount += $amount;
        }
    }
}

pub struct Transpose {
    amount: i16,
    notes: Box<[Note]>,
}

impl Transpose {
    pub fn new() -> Self {
        Self {
            amount: 0,
            notes: Box::new([]),
        }
    }
}

impl Module for Transpose {
    fn tick(&mut self) -> Option<Data> {
        let res = if !self.notes.is_empty() {
            let res = Some(Data::Notes(self.notes.iter().map(|note| note.transpose(self.amount)).collect()));
            self.notes = Box::new([]);
            res
        } else {
            None
        };
        res
    }

    define_module! {
        title: "Transpose",
        output: Notes,
        inputs: [(Notes, "notes")],
    }

    fn send(&mut self, _input: usize, data: Data) {
        self.notes = data.notes();
    }

    fn draw(&mut self, ui: &UiContext, interact: Option<ModuleInteractInfo>)
        -> Option<sdl2::surface::Surface<'_>> {

        use sdl2::{
            surface::Surface,
            pixels::PixelFormatEnum,
        };

        let (width, height) = (200, 100);

        let mut canvas =
            Surface::new(width, height, PixelFormatEnum::RGBA32)
            .unwrap().into_canvas().unwrap();

        let mouse_pos = interact.as_ref().and_then(|info| Some((info.x, info.y)));
        let mut layout = crate::ui_utils::SimpleLayoutBuilder::new((0, 0), mouse_pos);

        ui.add_label(&mut canvas, &mut layout, &*format!("transposition: {}", self.amount), None);
        layout.next_row();

        add_btn!(self, ui, canvas, layout, interact, 1, 2);
        add_btn!(self, ui, canvas, layout, interact, -1, 3);
        layout.next_row();
        add_btn!(self, ui, canvas, layout, interact, 7, 2);
        add_btn!(self, ui, canvas, layout, interact, -7, 3);
        layout.next_row();
        add_btn!(self, ui, canvas, layout, interact, 12, 2);
        add_btn!(self, ui, canvas, layout, interact, -12, 3);

        layout.next_row();
        if ui.add_button(&mut canvas, &mut layout, &interact, "reset", None) {
            self.amount = 0;
        }

        Some(canvas.into_surface())
    }
}
