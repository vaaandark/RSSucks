use std::cell::RefCell;
use std::rc::Rc;

use egui::Widget;
use uuid::Uuid;

use crate::render::article;
use crate::{
    subscription::feed::Feed,
    subscription::opml::Opml,
    utils::rss_client_ng::{ArticleId, EntryId, FolderId, RssClient},
    widget::{self, CollapsingFolder},
    RSSucks,
};

pub trait Window {
    fn show(&mut self, ctx: &egui::Context);
    fn is_open(&self) -> bool;
}

#[derive(Clone)]
pub struct ReaderView {
    article_id: ArticleId,
    parent_view: Option<Rc<Box<dyn View>>>,
    cached_detail: Rc<RefCell<Option<article::Detail>>>,
}

impl ReaderView {
    pub fn new(article_id: ArticleId, parent_view: Option<Rc<Box<dyn View>>>) -> Self {
        Self {
            article_id,
            parent_view,
            cached_detail: Rc::new(RefCell::new(None)),
        }
    }
}

impl View for ReaderView {
    fn show(&self, app: Rc<RSSucks>, ui: &mut egui::Ui) {
        if self.cached_detail.borrow().is_none() {
            let article = app
                .rss_client
                .get_article_by_id(&self.article_id)
                .unwrap()
                .get();
            let detail = article::Detail::from(article::Builder::from_article(
                article.lock().as_ref().unwrap(),
                self.article_id.clone(),
                self.parent_view.as_ref().map(Rc::clone),
                Rc::clone(&app),
            ));
            self.cached_detail.replace(Some(detail));
        }
        self.cached_detail.borrow().as_ref().unwrap().ui(ui);
    }
}

#[derive(Clone)]
pub struct FeedFlowView {
    id: EntryId,
    page: usize,
    per_page: usize,
    cached_previews: Rc<RefCell<Option<Vec<article::Preview>>>>,
}

impl FeedFlowView {
    pub fn new(id: EntryId) -> Self {
        Self {
            id,
            page: 1,
            per_page: 20,
            cached_previews: Rc::new(RefCell::new(None)),
        }
    }
}

impl View for FeedFlowView {
    fn show(&self, app: Rc<RSSucks>, ui: &mut egui::Ui) {
        if let Some(is_syncing) = app.rss_client.entry_is_syncing(self.id) {
            if is_syncing {
                ui.spinner();
            }
        } else {
            return;
        }

        let articles = app
            .rss_client
            .get()
            .borrow()
            .try_get_all_article_ids_by_entry_id(&self.id.get());

        match articles {
            Ok(articles) => {
                let current_view: Rc<Box<dyn View>> = Rc::new(Box::new((*self).clone()));

                if self.cached_previews.borrow().is_none() {
                    let previews = articles
                        .into_iter()
                        .skip((self.page - 1) * self.per_page)
                        .take(self.per_page)
                        .map(ArticleId::from)
                        .map(|article_id| {
                            let article =
                                app.rss_client.get_article_by_id(&article_id).unwrap().get();
                            let article = article.lock();
                            let builder = article::Builder::from_article(
                                article.as_ref().unwrap(),
                                article_id,
                                Some(Rc::clone(&current_view)),
                                Rc::clone(&app),
                            );
                            article::Preview::from(builder)
                        })
                        .collect();
                    self.cached_previews.replace(Some(previews));
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for preview in self.cached_previews.borrow().as_ref().unwrap() {
                        if ui.add(preview).clicked() {
                            app.set_view(Rc::new(Box::new(ReaderView::new(
                                preview.article_id.clone(),
                                Some(Rc::clone(&current_view)),
                            ))));
                        }
                    }
                });

                // ui.label("第一页（暂时还没写翻页的操作");
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
    alias: String,
    folder_id: Option<FolderId>,
    feed_url: String,
}

impl NewFeedWindow {
    pub fn new(client: RssClient, folder_id: Option<FolderId>) -> Self {
        Self {
            client,
            id: egui::Id::new(Uuid::new_v4()),
            is_open: true,
            alias: "稍后自动获取".to_owned(),
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
                let selected = if let Some(select_folder_id) = self.folder_id {
                    self.client.get_folder(&select_folder_id).unwrap().name()
                } else {
                    "None".to_owned()
                };
                ui.horizontal(|ui| {
                    ui.label("目标文件夹");
                    egui::ComboBox::from_label("请选择")
                        .selected_text(selected)
                        .show_ui(ui, |ui| {
                            self.client.list_folder().iter().for_each(|folder_id| {
                                if let Some(folder) = self.client.get_folder(folder_id) {
                                    ui.selectable_value(
                                        &mut self.folder_id,
                                        Some(*folder_id),
                                        folder.name(),
                                    );
                                }
                            });
                            ui.selectable_value(&mut self.folder_id, None, "不选择");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("订阅标题：");
                    ui.text_edit_singleline(&mut self.alias);
                });

                ui.horizontal(|ui| {
                    ui.label("订阅链接：");
                    ui.text_edit_singleline(&mut self.feed_url);
                });

                ui.horizontal(|ui| {
                    match url::Url::parse(&self.feed_url) {
                        Ok(url) => {
                            if ui.button("✔").on_hover_text("确定").clicked() {
                                let alias = if self.alias.is_empty() || self.alias == "稍后自动获取"
                                {
                                    None
                                } else {
                                    Some(&self.alias)
                                };
                                match self.folder_id {
                                    Some(folder_id) => {
                                        self.client.create_entry_with_folder(url, folder_id, alias);
                                    }
                                    None => {
                                        self.client.create_entry(url, alias);
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
        egui::SidePanel::left("left_panel").show(ctx, move |ui| {
            let former_visuals = self.app.visuals.borrow().clone();
            ui.ctx().set_visuals(former_visuals.clone());
            if let Some(visuals) = former_visuals.light_dark_small_toggle_button(ui) {
                ui.ctx().set_visuals(visuals.clone());
                if visuals != former_visuals {
                    self.app.visuals.replace(visuals);
                }
            }
            ui.heading("Rust SuckS");
            ui.label("用 Rust 写的 RSS 阅读器");
            ui.hyperlink_to("RSSucks on Github", "https://github.com/jyi2ya/RSSucks");

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("订阅列表");
                let app = self.app as *const RSSucks as *mut RSSucks;
                if ui.button("📥").on_hover_text("导入配置").clicked() {
                    async_std::task::block_on(async move {
                        if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
                            let data = file.read().await;
                            if let Ok(opml) = Opml::try_from_str(&String::from_utf8_lossy(&data)) {
                                if let Ok(feed) = Feed::try_from(opml) {
                                    unsafe {
                                        (*app).import_feed(feed);
                                    }
                                }
                            }
                        }
                    });
                }
                if ui.button("📤").on_hover_text("导出配置").clicked() {
                    async_std::task::block_on(async move {
                        if let Some(file) = rfd::AsyncFileDialog::new().save_file().await {
                            let opml =
                                Opml::from(unsafe { (*app).rss_client.get().borrow().to_owned() });
                            if let Ok(data) = opml.try_dump() {
                                let _ = file.write(data.as_bytes()).await;
                            }
                        }
                    });
                }
                if ui.button("🔁").on_hover_text("拉取全部").clicked() {
                    let _ = self.app.rss_client.try_start_sync_all();
                }
                if ui.button("新建文件夹").clicked() {
                    self.app
                        .add_window(NewFolderWindow::new(self.app.rss_client.clone()));
                }
                if ui.button("新建订阅").clicked() {
                    self.app
                        .add_window(NewFeedWindow::new(self.app.rss_client.clone(), None));
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.separator();

                for folder_id in self.app.rss_client.list_folder() {
                    ui.add(CollapsingFolder::new(self.app, folder_id));
                }

                for feed_id in self.app.rss_client.list_orphan_entry() {
                    ui.add(widget::FeedMinimal::new(self.app, feed_id));
                }
            });
        });
    }
}

pub trait View {
    fn show(&self, app: Rc<RSSucks>, ui: &mut egui::Ui);
}

#[derive(Default, Clone)]
pub struct DummyView {}

impl View for DummyView {
    fn show(&self, _app: Rc<RSSucks>, ui: &mut egui::Ui) {
        ui.heading("订阅分类或者订阅本身的标题");
        ui.label("一些关于订阅或者分类的介绍 blablablabla");

        ui.spacing();

        ui.label("列出所有文章");
        ui.label(
            "这下面可能还需要列一堆订阅的文章、题图和摘要出来。可能要写个新的控件，先摆了总之",
        );
    }
}
