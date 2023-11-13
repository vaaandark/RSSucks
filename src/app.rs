use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    utils::rss_client::RssClient,
    view::{self, View},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct RSSucks {
    pub rss_client: RssClient,

    #[serde(skip)]
    pub view: Option<Box<dyn View>>,
    #[serde(skip)]
    pub next_view: RefCell<Option<Box<dyn View>>>,

    #[serde(skip)]
    windows: Arc<Mutex<Vec<Box<dyn view::Window>>>>,
    #[serde(skip)]
    adding_windows: Arc<Mutex<Vec<Box<dyn view::Window>>>>,
}

impl RSSucks {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "NeoXiHei".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/LXGWNeoXiHei.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("NeoXiHei".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("NeoXiHei".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn add_window(&self, window: impl view::Window + 'static) {
        self.adding_windows
            .lock()
            .expect("rare error detected")
            .push(Box::new(window));
    }
}

impl eframe::App for RSSucks {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        view::LeftSidePanel::new(&self).show(ctx);
        view::CentralPanel::new(&self).show(ctx);

        for window in self.windows.lock().unwrap().iter_mut() {
            window.show(ctx);
        }

        self.windows
            .lock()
            .unwrap()
            .extend(self.adding_windows.lock().unwrap().drain(..));

        self.windows
            .lock()
            .expect("rare error detected")
            .retain(|window| window.is_open());

        match self.next_view.replace(None) {
            Some(view) => {
                self.view.replace(view);
            }
            None => {}
        };
    }
}

impl RSSucks {
    pub fn set_view(&self, view: impl View + 'static) -> &Self {
        self.next_view.replace(Some(Box::new(view)));
        self
    }
}
