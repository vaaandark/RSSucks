/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RSSucks {
    // Example stuff:
    label: String,
    list_unread_only: bool,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,
}

impl Default for RSSucks {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            list_unread_only: false,
            value: 2.7,
        }
    }
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
}

impl eframe::App for RSSucks {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            list_unread_only,
            value,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Rust Sucks");
            ui.label("用 Rust 写的 RSS 阅读器");
            ui.label("虽然还不能用但是给我个 Star 好不好就当投资了嘛");
            ui.hyperlink_to("RSSucks on Github", "https://github.com/jyi2ya/RSSucks");

            ui.separator();

            ui.button("今日订阅");
            ui.button("等下再看");
            ui.button("我的收藏");

            ui.separator();

            ui.label("订阅列表");

            let categories = vec![
                "好看的",
                "也不是不能看的",
                "不好看的",
                "难看的",
                "浪费手机资源的",
            ];

            for category in categories {
                ui.collapsing(category, |ui| {
                    for i in 0..3 {
                        ui.button(format!("{}-{}", category, i));
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("订阅分类或者订阅本身的标题");
            ui.label("一些关于订阅或者分类的介绍 blablablabla");

            ui.spacing();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.list_unread_only, true, "未读");
                ui.selectable_value(&mut self.list_unread_only, false, "所有");
            });
            ui.separator();

            if self.list_unread_only {
                ui.label("仅列出未读");
                ui.label("这下面可能还需要列一堆订阅的文章、题图和摘要出来。可能要写个新的控件，先摆了总之");
            } else {
                ui.label("列出所有文章");
                ui.label("这下面可能还需要列一堆订阅的文章、题图和摘要出来。可能要写个新的控件，先摆了总之");
            }
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
