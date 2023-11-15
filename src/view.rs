use std::cell::{Ref, RefCell};

use egui::Widget;
use uuid::Uuid;

use crate::{
    renderer,
    utils::rss_client_ng::{FeedId, FolderId, RssClient},
    widget::{self, CollapsingFolder},
    RSSucks,
};

pub trait Window {
    fn show(&mut self, ctx: &egui::Context);
    fn is_open(&self) -> bool;
}

pub struct ReaderView {
    entry_id_in_feed: String,
    feed_id: FeedId,
    cached_detail: RefCell<Option<renderer::Detail>>,
}

impl ReaderView {
    pub fn new(entry_id_in_feed: String, feed_id: FeedId) -> Self {
        Self {
            entry_id_in_feed,
            feed_id,
            cached_detail: RefCell::new(None),
        }
    }
}

impl View for ReaderView {
    fn show(&self, app: &RSSucks, ui: &mut egui::Ui) {
        if self.cached_detail.borrow().is_none() {
            let content = "stub";
            let time = "stub";
            let link = "stub";
            let title = "stub";
            let author = "stub";
            let channel = "stub";
            let component =
                renderer::ArticleComponent::new(channel, Some(author), title, link, time, content);
            self.cached_detail.replace(Some(component.to_detail()));
        }
        self.cached_detail.borrow().as_ref().unwrap().ui(ui);
    }
}

pub struct FeedFlowView {
    id: FeedId,
    page: usize,
    per_page: usize,
    cached_previews: RefCell<Option<Vec<renderer::Preview>>>,
}

impl<'a> FeedFlowView {
    pub fn new(id: FeedId) -> Self {
        Self {
            id,
            page: 1,
            per_page: 5,
            cached_previews: RefCell::new(None),
        }
    }
}

impl View for FeedFlowView {
    fn show(&self, app: &RSSucks, ui: &mut egui::Ui) {
        if app.rss_client.feed_is_syncing(self.id) {
            ui.spinner();
        }

        let feed = app.rss_client.get_feed(&self.id).unwrap();

        // match feed.model {
        //     Some(model) => {
        //         if let Some(title) = model.title {
        //             ui.heading(&title.content);
        //         };
        //         if let Some(updated) = model.updated {
        //             ui.label(format!("更新于 {}", updated));
        //         };
        //         if let Some(description) = model.description {
        //             ui.heading(&description.content);
        //         };
        //         ui.separator();

        //         if self.cached_previews.borrow().is_none() {
        //             let previews = model
        //                 .entries
        //                 .iter()
        //                 .map(|entry| {
        //                     let content = entry
        //                         .summary
        //                         .iter()
        //                         .next()
        //                         .map(|content| content.content.clone())
        //                         .unwrap_or("no content".to_owned());
        //                     let time = entry
        //                         .updated
        //                         .iter()
        //                         .next()
        //                         .map(|dt| dt.to_string())
        //                         .unwrap_or("no time".to_owned());
        //                     let link = entry
        //                         .links
        //                         .iter()
        //                         .next()
        //                         .map(|link| link.href.as_str())
        //                         .unwrap_or("no link");
        //                     let title = entry
        //                         .title
        //                         .as_ref()
        //                         .map(|title| title.content.clone())
        //                         .unwrap_or("unnamed".to_owned());
        //                     let author = entry
        //                         .authors
        //                         .iter()
        //                         .next()
        //                         .map(|author| author.name.as_str());
        //                     let channel = feed.url.as_str();
        //                     let component = renderer::ArticleComponent::new(
        //                         channel,
        //                         author,
        //                         title.as_str(),
        //                         link,
        //                         time.as_str(),
        //                         content.as_str(),
        //                     );
        //                     component.to_preview(self.id, entry.id.to_owned())
        //                 })
        //                 .collect();
        //             self.cached_previews.replace(Some(previews));
        //         }

        //         egui::ScrollArea::vertical().show(ui, |ui| {
        //             for preview in self.cached_previews.borrow().as_ref().unwrap() {
        //                 ui.add(preview);
        //                 if ui.button("阅读全文").clicked() {
        //                     app.set_view(ReaderView::new(
        //                         preview.entry_id.to_owned(),
        //                         preview.feed_id,
        //                     ));
        //                 }
        //             }
        //         });

        //         ui.label("第一页（暂时还没写翻页的操作");
        //     }
        //     None => {
        //         ui.label("该订阅尚未同步，现在同步吗？");
        //         if ui.button("同步").clicked() {
        //             app.rss_client.try_start_sync_feed(self.id);
        //         }
        //     }
        // };
    }
}

pub struct InfoWindow {
    id: egui::Id,
    is_open: bool,
    title: String,
    message: String,
}

impl InfoWindow {
    pub fn new(title: String, message: String) -> Self {
        Self {
            id: egui::Id::new(Uuid::new_v4()),
            is_open: true,
            title,
            message,
        }
    }
}

impl Window for InfoWindow {
    fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new(self.title.to_owned())
            .id(self.id)
            .open(&mut self.is_open)
            .movable(true)
            .collapsible(true)
            .title_bar(true)
            .show(ctx, |ui| ui.label(self.message.to_owned()));
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

pub struct NewFeedWindow {
    client: RssClient,
    id: egui::Id,
    is_open: bool,
    folder_id: Option<FolderId>,
    feed_url: String,
}

impl NewFeedWindow {
    pub fn new(client: RssClient, folder_id: Option<FolderId>) -> Self {
        Self {
            client,
            id: egui::Id::new(Uuid::new_v4()),
            is_open: true,
            folder_id,
            feed_url: String::new(),
        }
    }
}

impl Window for NewFeedWindow {
    fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("新建订阅")
            .id(self.id)
            .movable(true)
            .collapsible(true)
            .title_bar(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("订阅链接：");
                    ui.text_edit_singleline(&mut self.feed_url);
                });

                ui.horizontal(|ui| {
                    match url::Url::parse(&self.feed_url) {
                        Ok(url) => {
                            if ui.button("✔").on_hover_text("确定").clicked() {
                                match self.folder_id {
                                    Some(folder_id) => {
                                        self.client.create_feed_with_folder(url, folder_id);
                                    }
                                    None => {
                                        self.client.create_feed(url);
                                    }
                                }
                                self.is_open = false;
                            }
                        }
                        Err(err) => {
                            ui.label(format!("非法的 URL：{err}"));
                        }
                    };
                    if ui.button("🗙").on_hover_text("取消").clicked() {
                        self.is_open = false;
                    }
                });
            });
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

pub struct NewFolderWindow {
    client: RssClient,
    id: egui::Id,
    is_open: bool,
    folder_name: String,
}

impl NewFolderWindow {
    pub fn new(client: RssClient) -> Self {
        Self {
            client,
            id: egui::Id::new(Uuid::new_v4()),
            is_open: true,
            folder_name: String::new(),
        }
    }
}

impl Window for NewFolderWindow {
    fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("新建文件夹")
            .id(self.id)
            .movable(true)
            .collapsible(true)
            .title_bar(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("文件夹名称：");
                    ui.text_edit_singleline(&mut self.folder_name);
                });
                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        self.client.create_folder(&self.folder_name);
                        self.is_open = false;
                    }
                    if ui.button("取消").clicked() {
                        self.is_open = false;
                    }
                });
            });
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

pub struct LeftSidePanel<'app> {
    app: &'app RSSucks,
}

impl<'app> LeftSidePanel<'app> {
    pub fn new(app: &'app RSSucks) -> Self {
        Self { app }
    }
}

impl<'app> LeftSidePanel<'app> {
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_buttons(ui);
            ui.heading("Rust SuckS");
            ui.label("用 Rust 写的 RSS 阅读器");
            ui.hyperlink_to("RSSucks on Github", "https://github.com/jyi2ya/RSSucks");

            ui.separator();

            ui.label("订阅列表");

            ui.separator();

            if ui.button("新建文件夹").clicked() {
                self.app
                    .add_window(NewFolderWindow::new(self.app.rss_client.clone()));
            }

            for folder_id in self.app.rss_client.list_folder() {
                ui.add(CollapsingFolder::new(&self.app, folder_id));
            }

            for feed_id in self.app.rss_client.list_orphan_feed() {
                ui.add(widget::FeedMinimal::new(&self.app, feed_id));
            }
        });
    }
}

pub trait View {
    fn show(&self, app: &RSSucks, ui: &mut egui::Ui);
}

#[derive(Default)]
pub struct DummyView {}

impl View for DummyView {
    fn show(&self, _app: &RSSucks, ui: &mut egui::Ui) {
        ui.heading("订阅分类或者订阅本身的标题");
        ui.label("一些关于订阅或者分类的介绍 blablablabla");

        ui.spacing();

        ui.label("列出所有文章");
        ui.label(
            "这下面可能还需要列一堆订阅的文章、题图和摘要出来。可能要写个新的控件，先摆了总之",
        );
    }
}
