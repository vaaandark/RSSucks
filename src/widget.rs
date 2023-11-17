use std::rc::Rc;

use egui::{Response, Ui, Widget};

use crate::{
    utils::rss_client_ng::{EntryId, FolderId},
    view, RSSucks,
};

pub struct FeedMinimal<'a> {
    app: &'a RSSucks,
    id: EntryId,
}

impl<'a> FeedMinimal<'a> {
    pub fn new(app: &'a RSSucks, id: EntryId) -> Self {
        Self { app, id }
    }
}

impl<'a> Widget for FeedMinimal<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.allocate_ui(ui.available_size(), |ui| {
            let feed = self.app.rss_client.get_entry(&self.id).unwrap();
            ui.horizontal(|ui| {
                let feed_button = ui.button(feed.get_name());

                if feed_button.clicked() {
                    self.app
                        .set_view(Rc::new(Box::new(view::FeedFlowView::new(self.id))));
                }

                if self
                    .app
                    .rss_client
                    .entry_is_syncing(self.id)
                    .unwrap_or(false)
                {
                    ui.spinner();
                }

                if ui.button("🔁").on_hover_text("拉取文章").clicked() {
                    self.app.rss_client.try_start_sync_entry(self.id).unwrap();
                }

                if ui.button("🗙").on_hover_text("删除订阅").clicked() {
                    self.app.rss_client.delete_entry(self.id);
                }
            });
        })
        .response
    }
}

pub struct CollapsingFolder<'app> {
    app: &'app RSSucks,
    folder_id: FolderId,
}

impl<'app> CollapsingFolder<'app> {
    pub fn new(app: &'app RSSucks, folder_id: FolderId) -> Self {
        Self { app, folder_id }
    }
}

impl<'app> Widget for CollapsingFolder<'app> {
    fn ui(self, ui: &mut Ui) -> Response {
        let folder = self.app.rss_client.get_folder(&self.folder_id).unwrap();
        let response = egui::CollapsingHeader::new(folder.name()).show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("🔁").on_hover_text("拉取文章").clicked() {
                    self.app
                        .rss_client
                        .try_start_sync_folder(self.folder_id)
                        .unwrap();
                }

                if ui.button("📋").on_hover_text("新增订阅").clicked() {
                    self.app.add_window(view::NewFeedWindow::new(
                        self.app.rss_client.clone(),
                        Some(self.folder_id),
                    ));
                }
                if ui.button("🗙").on_hover_text("删除文件夹").clicked() {
                    self.app.rss_client.delete_folder(self.folder_id).unwrap();
                }
            });
            if let Ok(feed_ids) = self.app.rss_client.try_list_entry_by_folder(self.folder_id) {
                for feed_id in feed_ids {
                    ui.add(FeedMinimal::new(self.app, feed_id));
                }
            }
        });
        response.body_response.unwrap_or(response.header_response)
    }
}
