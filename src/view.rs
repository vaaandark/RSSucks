use std::cell::RefCell;

use egui::Widget;
use uuid::Uuid;

use crate::utils::rss_client_ng::ArticleId;
use crate::widgets::article;
use crate::{
    utils::rss_client_ng::{EntryId, FolderId, RssClient},
    widget::{self, CollapsingFolder},
    RSSucks,
};

pub trait Window {
    fn show(&mut self, ctx: &egui::Context);
    fn is_open(&self) -> bool;
}

pub struct ReaderView {
    article_id: ArticleId,
    cached_detail: RefCell<Option<article::Detail>>,
}

impl ReaderView {
    pub fn new(article_id: ArticleId) -> Self {
        Self {
            article_id,
            cached_detail: RefCell::new(None),
        }
    }
}

impl View for ReaderView {
    fn show(&self, app: &RSSucks, ui: &mut egui::Ui) {
        if self.cached_detail.borrow().is_none() {
            let feed = app.rss_client.get();
            let article = app
                .rss_client
                .get_article_by_id(&self.article_id)
                .unwrap()
                .get();
            let detail = article::Detail::from(article::Builder::from_article(
                article.lock().as_ref().unwrap(),
                self.article_id.get(),
                feed,
            ));
            self.cached_detail.replace(Some(detail));
        }
        self.cached_detail.borrow().as_ref().unwrap().ui(ui);
    }
}

pub struct FeedFlowView {
    id: EntryId,
    page: usize,
    per_page: usize,
    cached_previews: RefCell<Option<Vec<article::Preview>>>,
}

impl FeedFlowView {
    pub fn new(id: EntryId) -> Self {
        Self {
            id,
            page: 1,
            per_page: 20,
            cached_previews: RefCell::new(None),
        }
    }
}

impl View for FeedFlowView {
    fn show(&self, app: &RSSucks, ui: &mut egui::Ui) {
        if app.rss_client.entry_is_syncing(self.id) {
            ui.spinner();
        }

        let articles = app
            .rss_client
            .get()
            .borrow()
            .try_get_all_article_ids_by_entry_id(&self.id.get());

        match articles {
            Ok(articles) => {
                if self.cached_previews.borrow().is_none() {
                    let previews = articles
                        .into_iter()
                        .skip((self.page - 1) * self.per_page)
                        .take(self.per_page)
                        .map(ArticleId::from)
                        .map(|article_id| {
                            let feed = app.rss_client.get();
                            let article =
                                app.rss_client.get_article_by_id(&article_id).unwrap().get();
                            let article = article.lock();
                            let builder = article::Builder::from_article(
                                article.as_ref().unwrap(),
                                article_id.get(),
                                feed,
                            );
                            article::Preview::from(builder)
                        })
                        .collect();
                    self.cached_previews.replace(Some(previews));
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for preview in self.cached_previews.borrow().as_ref().unwrap() {
                        ui.add(preview);
                        if ui.button("阅读全文").clicked() {
                            app.set_view(ReaderView::new(ArticleId::from(
                                preview.article_id.clone(),
                            )));
                        }
                    }
                });

                ui.label("第一页（暂时还没写翻页的操作");
            }
            Err(_) => {
                ui.label("该订阅尚未同步，现在同步吗？");
                if ui.button("同步").clicked() {
                    app.rss_client.try_start_sync_entry(self.id).unwrap();
                }
            }
        };
    }
}

pub struct InfoWindow {
    id: egui::Id,
    is_open: bool,
    title: String,
    message: String,
}

#[allow(unused)]
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
                                        self.client.create_entry_with_folder(url, folder_id);
                                    }
                                    None => {
                                        self.client.create_entry(url);
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
                ui.add(CollapsingFolder::new(self.app, folder_id));
            }

            for feed_id in self.app.rss_client.list_orphan_entry() {
                ui.add(widget::FeedMinimal::new(self.app, feed_id));
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
