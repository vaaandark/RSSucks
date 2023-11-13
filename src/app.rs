/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct RSSucks {
    #[serde(skip)]
    list_unread_only: bool,
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
        self.draw_side_panel(ctx);
        self.draw_central_panel(ctx);
    }
}

impl RSSucks {
    fn draw_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_buttons(ui);
            ui.heading("Rust SuckS");
            ui.label("用 Rust 写的 RSS 阅读器");
            ui.hyperlink_to("RSSucks on Github", "https://github.com/jyi2ya/RSSucks");

            ui.separator();

            ui.label("订阅列表");

            ui.separator();
        });
    }

    fn draw_central_panel(&mut self, ctx: &egui::Context) {
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
    }
}
