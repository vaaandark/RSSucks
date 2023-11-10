use anyhow::Result;
use ego_tree::iter::Edge;
use egui::{Image, Margin, RichText, Separator};
use scraper::Html;
use std::collections::VecDeque;

enum WidgetType<'a> {
    Label {
        text: RichText,
    },
    Hyperlink {
        text: RichText,
        destination: &'a str,
    },
    Image {
        src: Option<&'a str>,
        width: Option<&'a str>,
        height: Option<&'a str>,
    },
    Separator,
    Newline,
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
    Others,
}

pub struct ArticleComponent<'a> {
    channel: &'a str,
    author: Option<&'a str>,
    title: &'a str,
    link: &'a str,
    time: &'a str,
    content: &'a str,
}

fn richtext_generator(text: &str, dom_stack: &[ElementType<'_>]) -> egui::RichText {
    let richtext =
        dom_stack.iter().fold(
            egui::RichText::new(text).size(16.0),
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
        ArticleComponent {
            channel,
            author,
            title,
            link,
            time,
            content,
        }
    }

    pub fn render_detail_component(&self, ctx: &egui::Context, ui: &mut egui::Ui) -> Result<()> {
        let fragment = Html::parse_fragment(self.content);
        let mut dom_stack: Vec<_> = Vec::new();
        let mut widget_queue: VecDeque<_> = VecDeque::new();

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
                        let richtext = richtext_generator(&text, &dom_stack);
                        let hyperlink_destination = dom_stack.iter().fold(None, |dest, element| {
                            if let &ElementType::A { destination } = element {
                                destination
                            } else {
                                dest
                            }
                        });
                        if let Some(dest) = hyperlink_destination {
                            widget_queue.push_back(WidgetType::Hyperlink {
                                text: richtext,
                                destination: dest,
                            });
                        } else {
                            widget_queue.push_back(WidgetType::Label { text: richtext });
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
                            let (src, width, height) =
                                (tag.attr("src"), tag.attr("width"), tag.attr("height"));
                            widget_queue.push_back(WidgetType::Image { src, width, height });
                        }
                        "em" => dom_stack.push(ElementType::Em),
                        "strong" => dom_stack.push(ElementType::Strong),
                        "hr" => {
                            dom_stack.push(ElementType::Hr);
                            widget_queue.push_back(WidgetType::Separator);
                        }
                        "code" => dom_stack.push(ElementType::Code),
                        "br" => {
                            dom_stack.push(ElementType::Br);
                            widget_queue.push_back(WidgetType::Newline);
                        }
                        "pre" => dom_stack.push(ElementType::Pre),
                        _ => dom_stack.push(ElementType::Others),
                    },
                    _ => {}
                },

                Edge::Close(node) => {
                    if let scraper::Node::Element(tag) = node.value() {
                        dom_stack.pop();
                        if (dom_stack.len() == 1 || tag.name() == "li")
                            && tag.name() != "ul"
                            && tag.name() != "img"
                        {
                            widget_queue.push_back(WidgetType::Newline);
                        }
                    }
                }
            }
        }

        ui.group(|ui| {
            // Set the spacing between header and content.
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);
            ui.style_mut().override_text_style = Some(egui::TextStyle::Body);
            // Render header:
            egui::Frame::none()
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                .show(ui, |ui| {
                    const HEADER_LARGE_TEXT_SIZE: f32 = 32.0;
                    const HEADER_SMALL_TEXT_SIZE: f32 = 10.0;
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.hyperlink_to(
                        RichText::new(self.title)
                            .size(HEADER_LARGE_TEXT_SIZE)
                            .strong(),
                        self.link,
                    );
                    ui.horizontal_wrapped(|ui| {
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
                    // Fill the rest empty space, to make the width of the frame the same as the outer frame.
                    ui.allocate_space(egui::Vec2 {
                        x: ui.max_rect().width(),
                        y: 0.0,
                    });
                });
            // Render content:
            egui::Frame::none()
                .inner_margin(Margin::same(5.0))
                .show(ui, |ui| {
                    while !widget_queue.is_empty() {
                        ui.horizontal_wrapped(|ui| loop {
                            match widget_queue.front() {
                                Some(WidgetType::Label { text: _ }) => {
                                    if let Some(WidgetType::Label { text: label }) =
                                        widget_queue.pop_front()
                                    {
                                        ui.label(label);
                                    }
                                }
                                Some(WidgetType::Newline) => {
                                    widget_queue.pop_front();
                                    ui.end_row();
                                }
                                Some(WidgetType::Hyperlink {
                                    text: _,
                                    destination: _,
                                }) => {
                                    if let Some(WidgetType::Hyperlink {
                                        text: label,
                                        destination: dest,
                                    }) = widget_queue.pop_front()
                                    {
                                        ui.hyperlink_to(label, dest);
                                    }
                                }
                                Some(WidgetType::Separator) => {
                                    widget_queue.pop_front();
                                    ui.add(Separator::horizontal(Separator::default()));
                                }
                                _ => break,
                            }
                        });
                        ui.horizontal_wrapped(|ui| {
                            while let Some(WidgetType::Image {
                                src: _,
                                width: _,
                                height: _,
                            }) = widget_queue.front()
                            {
                                if let Some(WidgetType::Image { src, width, height }) =
                                    widget_queue.pop_front()
                                {
                                    egui_extras::install_image_loaders(ctx);
                                    ui.add(
                                        Image::from(src.unwrap())
                                            .fit_to_original_size(1.0)
                                            .max_width(match width {
                                                Some(width) => match width.parse::<f32>() {
                                                    Ok(width) => width,
                                                    _ => ui.max_rect().width(),
                                                },
                                                None => ui.max_rect().width(),
                                            })
                                            .max_height(match height {
                                                Some(height) => match height.parse::<f32>() {
                                                    Ok(height) => height,
                                                    _ => f32::INFINITY,
                                                },
                                                None => f32::INFINITY,
                                            }),
                                    );
                                }
                            }
                        });
                    }
                    ui.allocate_space(egui::Vec2 {
                        x: ui.max_rect().width(),
                        y: 0.0,
                    });
                });
        });
        Ok(())
    }
}
