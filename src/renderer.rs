use anyhow::Result;
use ego_tree::iter::Edge;
use egui::{Image, Margin, RichText, Rounding, Separator};
use scraper::Html;
use std::collections::VecDeque;
use uuid::Uuid;

enum WidgetType {
    Label {
        text: RichText,
    },
    Hyperlink {
        text: RichText,
        destination: String,
    },
    Image {
        src: Option<String>,
        width: Option<String>,
        height: Option<String>,
    },
    Separator,
    Newline,
}

impl std::fmt::Debug for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Label { text } => f.debug_struct("Label").field("text", &text.text()).finish(),
            Self::Hyperlink { text, destination } => f
                .debug_struct("Hyperlink")
                .field("text", &text.text())
                .field("destination", destination)
                .finish(),
            Self::Image { src, width, height } => f
                .debug_struct("Image")
                .field("src", src)
                .field("width", width)
                .field("height", height)
                .finish(),
            Self::Separator => write!(f, "Separator"),
            Self::Newline => write!(f, "Newline"),
        }
    }
}

#[derive(PartialEq, Debug)]
enum ElementType<'a> {
    P,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Img,
    A { destination: Option<&'a str> },
    Em,
    Strong,
    Hr,
    Code,
    Br,
    Pre,
}

pub struct ArticleComponent<'a> {
    channel: &'a str,
    author: Option<&'a str>,
    title: &'a str,
    link: &'a str,
    time: &'a str,
    widgets: VecDeque<WidgetType>,
    // For preview
    max_rows: usize,
    break_anywhere: bool,
    overflow_character: Option<char>,
    fulltext: String,
}

fn richtext_generator(text: &str, dom_stack: &[ElementType<'_>]) -> egui::RichText {
    let richtext = dom_stack.iter().fold(
        egui::RichText::new(text).size(16.0).line_height(Some(28.0)),
        |richtext, element| match element {
            ElementType::H1 => richtext.strong().size(32.0),
            ElementType::H2 => richtext.strong().size(24.0),
            ElementType::H3 => richtext.strong().size(18.72),
            ElementType::H4 => richtext.strong().size(16.0),
            ElementType::H5 => richtext.strong().size(13.28),
            ElementType::H6 => richtext.strong().size(10.72),
            ElementType::Em => richtext.italics(),
            ElementType::Strong => richtext.strong(),
            ElementType::Code => richtext.code(),
            _ => richtext,
        },
    );
    richtext
}

impl<'a> ArticleComponent<'_> {
    pub fn new(
        channel: &'a str,
        author: Option<&'a str>,
        title: &'a str,
        link: &'a str,
        time: &'a str,
        content: &'a str,
    ) -> ArticleComponent<'a> {
        let fragment = Html::parse_fragment(content);
        let mut dom_stack: Vec<_> = Vec::new();
        let mut widgets: VecDeque<_> = VecDeque::new();
        let mut fulltext = String::new();

        for edge in fragment.root_element().traverse() {
            match edge {
                Edge::Open(node) => match node.value() {
                    scraper::Node::Text(text) => {
                        let text = if !dom_stack.contains(&ElementType::Pre) {
                            text.replace('\n', "")
                        } else {
                            text.to_string()
                        };
                        if text.is_empty() {
                            continue;
                        }
                        fulltext += &text;
                        let richtext = richtext_generator(&text, &dom_stack);
                        let hyperlink_destination = dom_stack.iter().fold(None, |dest, element| {
                            if let &ElementType::A { destination } = element {
                                destination
                            } else {
                                dest
                            }
                        });
                        if let Some(dest) = hyperlink_destination {
                            widgets.push_back(WidgetType::Hyperlink {
                                text: richtext,
                                destination: dest.to_owned(),
                            });
                        } else {
                            widgets.push_back(WidgetType::Label { text: richtext });
                        }
                    }
                    scraper::Node::Element(tag) => match tag.name() {
                        "p" => dom_stack.push(ElementType::P),
                        "h1" => dom_stack.push(ElementType::H1),
                        "h2" => dom_stack.push(ElementType::H2),
                        "h3" => dom_stack.push(ElementType::H3),
                        "h4" => dom_stack.push(ElementType::H4),
                        "h5" => dom_stack.push(ElementType::H5),
                        "h6" => dom_stack.push(ElementType::H6),
                        "a" => dom_stack.push(ElementType::A {
                            destination: tag.attr("href"),
                        }),
                        "img" => {
                            dom_stack.push(ElementType::Img);
                            widgets.push_back(WidgetType::Image {
                                src: tag.attr("src").map(|s| s.to_owned()),
                                width: tag.attr("width").map(|s| s.to_owned()),
                                height: tag.attr("height").map(|s| s.to_owned()),
                            });
                        }
                        "em" => dom_stack.push(ElementType::Em),
                        "strong" => dom_stack.push(ElementType::Strong),
                        "hr" => {
                            dom_stack.push(ElementType::Hr);
                            widgets.push_back(WidgetType::Separator);
                        }
                        "code" => dom_stack.push(ElementType::Code),
                        "br" => {
                            dom_stack.push(ElementType::Br);
                            widgets.push_back(WidgetType::Newline);
                        }
                        "pre" => dom_stack.push(ElementType::Pre),
                        _ => {}
                    },
                    _ => {}
                },

                Edge::Close(node) => {
                    if let scraper::Node::Element(tag) = node.value() {
                        match tag.name() {
                            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "a" | "img" | "em"
                            | "strong" | "hr" | "code" | "br" | "pre" => {
                                dom_stack.pop();
                                if dom_stack.is_empty() || tag.name() == "li" {
                                    widgets.push_back(WidgetType::Newline);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        ArticleComponent {
            channel,
            author,
            title,
            link,
            time,
            widgets,
            max_rows: 3,
            break_anywhere: true,
            overflow_character: Some('…'),
            fulltext,
        }
    }

    fn try_get_widget_by_index(&self, idx: usize) -> Option<&WidgetType> {
        self.widgets.get(idx)
    }

    pub fn render_detail_component(&self, ctx: &egui::Context, ui: &mut egui::Ui) -> Result<()> {
        egui::Frame::none()
            .inner_margin(Margin::same(16.0))
            .outer_margin(Margin::symmetric(
                if ui.max_rect().width() > 1024.0 {
                    (ui.max_rect().width() - 1024.0) / 2.0
                } else {
                    0.0
                },
                8.0,
            ))
            .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
            .rounding(Rounding::ZERO.at_least(10.0))
            .show(ui, |ui| {
                // Set the spacing between header and content.
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                ui.style_mut().override_text_style = Some(egui::TextStyle::Body);
                // Render header:
                egui::Frame::none()
                    .outer_margin(Margin::same(16.0))
                    // .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                    .show(ui, |ui| {
                        const HEADER_LARGE_TEXT_SIZE: f32 = 32.0;
                        const HEADER_SMALL_TEXT_SIZE: f32 = 12.0;

                        ui.hyperlink_to(
                            RichText::new(self.title)
                                .size(HEADER_LARGE_TEXT_SIZE)
                                .strong(),
                            self.link,
                        );
                        ui.horizontal_wrapped(|ui| {
                            ui.add_space(4.0);
                            if let Some(author) = self.author {
                                ui.label(RichText::new("👤 ").size(HEADER_SMALL_TEXT_SIZE));
                                ui.label(RichText::new(author).size(HEADER_SMALL_TEXT_SIZE));
                                ui.label(RichText::new(" @ ").size(HEADER_SMALL_TEXT_SIZE));
                                ui.label(RichText::new(self.channel).size(HEADER_SMALL_TEXT_SIZE));
                            } else {
                                ui.label(RichText::new(self.channel).size(HEADER_SMALL_TEXT_SIZE));
                            }
                            ui.label(RichText::new("\t").size(HEADER_SMALL_TEXT_SIZE));
                            ui.label(RichText::new("🕐 ").size(HEADER_SMALL_TEXT_SIZE));
                            ui.label(RichText::new(self.time).size(HEADER_SMALL_TEXT_SIZE));
                        });
                    });
                ui.separator();
                // Render content:
                ui.scope(|ui| {
                    egui::Frame::none()
                        .outer_margin(Margin::symmetric(16.0, 4.0))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);
                            let widgets_num = self.widgets.len();
                            let mut idx: usize = 0;
                            while idx < widgets_num {
                                ui.horizontal_wrapped(|ui| loop {
                                    match self.try_get_widget_by_index(idx) {
                                        Some(WidgetType::Label { text: _ }) => {
                                            if let Some(WidgetType::Label { text: label }) =
                                                self.try_get_widget_by_index(idx)
                                            {
                                                ui.label(label.clone());
                                                idx += 1;
                                            }
                                        }
                                        Some(WidgetType::Newline) => {
                                            ui.end_row();
                                            idx += 1;
                                        }
                                        Some(WidgetType::Hyperlink {
                                            text: _,
                                            destination: _,
                                        }) => {
                                            if let Some(WidgetType::Hyperlink {
                                                text: label,
                                                destination: dest,
                                            }) = self.try_get_widget_by_index(idx)
                                            {
                                                ui.hyperlink_to(label.clone(), dest);
                                                idx += 1;
                                            }
                                        }
                                        Some(WidgetType::Separator) => {
                                            ui.add(Separator::horizontal(Separator::default()));
                                            idx += 1;
                                        }
                                        _ => break,
                                    }
                                });
                                ui.horizontal_wrapped(|ui| {
                                    while let Some(WidgetType::Image {
                                        src: _,
                                        width: _,
                                        height: _,
                                    }) = self.try_get_widget_by_index(idx)
                                    {
                                        if let Some(WidgetType::Image { src, width, height }) =
                                            self.try_get_widget_by_index(idx)
                                        {
                                            if let Some(src) = src {
                                                egui_extras::install_image_loaders(ctx);
                                                ui.add(
                                                    Image::from(src)
                                                        .fit_to_original_size(1.0)
                                                        .max_width(match width {
                                                            Some(width) => {
                                                                match width.parse::<f32>() {
                                                                    Ok(width) => f32::min(
                                                                        width,
                                                                        ui.max_rect().width(),
                                                                    ),
                                                                    _ => ui.max_rect().width(),
                                                                }
                                                            }
                                                            None => ui.max_rect().width(),
                                                        })
                                                        .max_height(match height {
                                                            Some(height) => {
                                                                match height.parse::<f32>() {
                                                                    Ok(height) => height,
                                                                    _ => f32::INFINITY,
                                                                }
                                                            }
                                                            None => f32::INFINITY,
                                                        })
                                                        .rounding(Rounding::ZERO.at_least(10.0))
                                                        .show_loading_spinner(true),
                                                );
                                            }
                                            idx += 1;
                                        }
                                    }
                                    ui.end_row();
                                });
                            }
                        });
                });
            });
        Ok(())
    }

    pub fn render_preview_component(&self, ctx: &egui::Context, ui: &mut egui::Ui) -> Result<()> {
        egui::Frame::none()
            .inner_margin(Margin::same(32.0))
            .outer_margin(Margin::symmetric(
                if ui.max_rect().width() > 1024.0 {
                    (ui.max_rect().width() - 1024.0) / 2.0
                } else {
                    0.0
                },
                8.0,
            ))
            .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
            .rounding(Rounding::ZERO.at_least(10.0))
            .show(ui, |ui| {
                // Set the spacing between header and content.
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                ui.style_mut().override_text_style = Some(egui::TextStyle::Body);
                // Render title:
                ui.label(RichText::new(self.title).size(20.0).strong());

                // Render content:
                // First, render text.
                let mut job = egui::text::LayoutJob::single_section(
                    self.fulltext.clone(),
                    egui::TextFormat::default(),
                );
                job.wrap = egui::text::TextWrapping {
                    max_rows: self.max_rows,
                    break_anywhere: self.break_anywhere,
                    overflow_character: self.overflow_character,
                    ..Default::default()
                };
                ui.label(job);
                // Then render images.
                // egui::ScrollArea::horizontal()
                //     .id_source(Uuid::new_v4())
                //     .auto_shrink([false; 2])
                //     .drag_to_scroll(true)
                //     .show(ui, |ui| {
                ui.horizontal(|ui| {
                    self.widgets
                        .iter()
                        .filter_map(|widget| match widget {
                            WidgetType::Image {
                                src,
                                width: _,
                                height: _,
                            } => src.as_ref(),
                            _ => None,
                        })
                        .take(3)
                        .for_each(|src| {
                            egui_extras::install_image_loaders(ctx);
                            ui.add(
                                Image::from(src)
                                    .fit_to_exact_size(egui::Vec2::new(256.0, 128.0))
                                    .rounding(Rounding::ZERO.at_least(10.0))
                                    .show_loading_spinner(true),
                            );
                        });
                });
                // });
            });
        Ok(())
    }
}
