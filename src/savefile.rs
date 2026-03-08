use crate::*;

use serde::{Serialize, Deserialize};

use std::{
    sync::{Arc, Mutex},
};

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub app: App,
    pub modules: SerializeableModules,
    pub gui: Gui,
}

pub fn load_file(filename: &str) -> (Arc<Mutex<App>>, Gui) {
    fn load_file(filename: &str) -> anyhow::Result<(Arc<Mutex<App>>, Gui)> {
        let Project { mut app, gui, modules } = deserialize(std::fs::read(filename)?)?;
        for (id, (type_id, data)) in modules {
            let mut module = module_from_id(&type_id).unwrap();
            module.load_data(data);
            app.modules.insert(id, module);
        }
        Ok((Arc::new(Mutex::new(app)), gui))
    }

    match load_file(filename) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("error while loading file: {error}");
            let mut app = App::new();
            let mut gui = Gui::new();
            app.init();
            gui.init();
            for (id, module) in app.modules.iter() {
                gui.insert_module(*id, module);
            }
            (Arc::new(Mutex::new(app)), gui)
        }
    }
}

pub fn save_file(filename: &str, app: App, modules: SerializeableModules, gui: Gui) {
    let project = Project {
        app,
        modules,
        gui: gui.clone(),
    };
    std::fs::write(filename, serialize(&project).unwrap()).unwrap()
}
