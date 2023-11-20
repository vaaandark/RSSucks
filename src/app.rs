use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    utils::rss_client_ng::RssClient,
    view::{self, View},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
#[serde(default)]
pub struct RSSucks {
    pub rss_client: RssClient,
    pub visuals: Rc<RefCell<egui::Visuals>>,

    #[serde(skip)]
    pub view: RefCell<Option<Rc<Box<dyn View>>>>,
    #[serde(skip)]
    pub next_view: RefCell<Option<Rc<Box<dyn View>>>>,

    switch_cnt: i32,

    #[serde(skip)]
    windows: Arc<Mutex<Vec<Box<dyn view::Window>>>>,
    #[serde(skip)]
    adding_windows: Arc<Mutex<Vec<Box<dyn view::Window>>>>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
#[serde(default)]
pub struct App {
    app: Rc<RSSucks>,
}

impl App {
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
            let res: App = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Sync all feed
            let _ = res.app.rss_client.try_start_sync_all();
            return res;
        }

        Default::default()
    }
}

impl RSSucks {
    pub fn add_window(&self, window: impl view::Window + 'static) {
        self.adding_windows
            .lock()
            .expect("rare error detected")
            .push(Box::new(window));
    }

    pub fn import_feed(&mut self, feed: crate::subscription::feed::Feed) {
        self.rss_client = RssClient::new(feed);
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        view::LeftSidePanel::new(&self.app).show(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(view) = self.app.view.borrow().as_ref() {
                view.show(Rc::clone(&self.app), ui);
            }
        });

        if let Some(next_view) = self.app.next_view.replace(None) {
            self.app.view.borrow_mut().replace(next_view);
        };

        for window in self.app.windows.lock().unwrap().iter_mut() {
            window.show(ctx);
        }

        self.app
            .windows
            .lock()
            .unwrap()
            .extend(self.app.adding_windows.lock().unwrap().drain(..));

        self.app
            .windows
            .lock()
            .expect("rare error detected")
            .retain(|window| window.is_open());
    }
}

impl RSSucks {
    pub fn set_view(&self, view: Rc<Box<dyn View>>) -> &Self {
        self.next_view.replace(Some(view));
        self
    }
}
