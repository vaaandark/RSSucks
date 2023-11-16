use egui::{Image, Margin, RichText, Rounding, Widget};

use super::{Builder, Element, ElementType};

pub struct Detail {
    entry_title: Option<String>,
    title: String,
    link: Option<String>,
    updated: Option<String>,
    published: Option<String>,
    elements: Option<Vec<Element>>,
}

impl<'a> From<Builder<'a>> for Detail {
    fn from(value: Builder<'a>) -> Self {
        Detail {
            entry_title: value.entry_title.map(|s| s.to_owned()),
            title: value.title.to_owned(),
            link: value.link.map(|s| s.to_owned()),
            updated: value.updated.map(|s| s.to_owned()),
            published: value.published.map(|s| s.to_owned()),
            elements: value.elements,
        }
    }
}

// fn richtext_generator(text: &str, dom_stack: &[ElementType<'_>]) -> egui::RichText {
//     let richtext = dom_stack.iter().fold(
//         egui::RichText::new(text).size(16.0).line_height(Some(28.0)),
//         |richtext, element| match element {
//             ElementType::H1 => richtext.strong().size(32.0),
//             ElementType::H2 => richtext.strong().size(24.0),
//             ElementType::H3 => richtext.strong().size(18.72),
//             ElementType::H4 => richtext.strong().size(16.0),
//             ElementType::H5 => richtext.strong().size(13.28),
//             ElementType::H6 => richtext.strong().size(10.72),
//             ElementType::Em => richtext.italics(),
//             ElementType::Strong => richtext.strong(),
//             ElementType::Code => richtext.code(),
//             _ => richtext,
//         },
//     );
//     richtext
// }

impl<'a> Widget for &Detail {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.allocate_ui(ui.available_size(), |ui| {
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
                    // we will control the spacing manually later
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 16.0);

                    // Render header:
                    egui::Frame::none()
                        .outer_margin(Margin::same(16.0))
                        // .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                        .show(ui, |ui| {
                            const HEADER_LARGE_TEXT_SIZE: f32 = 32.0;
                            const HEADER_SMALL_TEXT_SIZE: f32 = 12.0;
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);

                            // title
                            if let Some(link) = &self.link {
                                ui.hyperlink_to(
                                    RichText::new(&self.title)
                                        .size(HEADER_LARGE_TEXT_SIZE)
                                        .strong(),
                                    link,
                                );
                            } else {
                                ui.label(
                                    RichText::new(&self.title)
                                        .size(HEADER_LARGE_TEXT_SIZE)
                                        .strong(),
                                );
                            }

                            // publish information
                            ui.horizontal_wrapped(|ui| {
                                ui.add_space(4.0);
                                // entry_title: Option<String>,
                                // updated: Option<String>,
                                // published: Option<String>,
                                if let Some(entry_title) = &self.entry_title {
                                    ui.label(
                                        RichText::new(entry_title).size(HEADER_SMALL_TEXT_SIZE),
                                    );
                                }
                                if let Some(published) = &self.published {
                                    ui.label(
                                        RichText::new("\tpublished at ").size(HEADER_SMALL_TEXT_SIZE),
                                    );
                                    ui.label(RichText::new(published).size(HEADER_SMALL_TEXT_SIZE));
                                }
                                if let Some(updated) = &self.updated {
                                    ui.label(
                                        RichText::new("\tupdated at ").size(HEADER_SMALL_TEXT_SIZE),
                                    );
                                    ui.label(RichText::new(updated).size(HEADER_SMALL_TEXT_SIZE));
                                }
                            });
                        });
                    // ui.separator();

                    // Render content:
                    ui.scope(|ui| {
                        egui::Frame::none()
                            .outer_margin(Margin::symmetric(16.0, 4.0))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    if let Some(elements) = &self.elements {
                                        let elements_len = elements.len();
                                        let mut idx: usize = 0;
                                        while idx < elements_len {
                                            ui.horizontal_wrapped(|ui| {
                                                while let Some(element) = elements.get(idx) {
                                                    // println!("{:?}", element.typ);
                                                    match element.typ {
                                                        ElementType::Paragraph => {
                                                            if let Some(richtext) = &element.text {
                                                                if let Some(dest) =
                                                                    &element.destination
                                                                {
                                                                    ui.hyperlink_to(
                                                                        richtext.to_owned(),
                                                                        dest,
                                                                    );
                                                                } else {
                                                                    ui.label(richtext.to_owned());
                                                                }
                                                            }
                                                        }
                                                        ElementType::Heading => {
                                                            if let Some(heading) =
                                                                element.text.to_owned()
                                                            {
                                                                ui.label(
                                                                    match element.heading_level {
                                                                        Some(level) => {
                                                                            match level {
                                                                                1 => heading
                                                                                    .size(32.0),
                                                                                2 => heading
                                                                                    .size(24.0),
                                                                                3 => heading
                                                                                    .size(18.72),
                                                                                4 => heading
                                                                                    .size(16.0),
                                                                                5 => heading
                                                                                    .size(13.28),
                                                                                6 => heading
                                                                                    .size(10.72),
                                                                                _ => heading,
                                                                            }
                                                                        }
                                                                        None => heading,
                                                                    },
                                                                );
                                                            }
                                                        }
                                                        ElementType::CodeBlock => {
                                                            // TODO
                                                        }
                                                        ElementType::ListItem => {
                                                            // TODO
                                                        }
                                                        ElementType::LineBreak => {
                                                            ui.end_row();
                                                        }
                                                        ElementType::Separator => {
                                                            ui.separator();
                                                        }
                                                        ElementType::Others => {
                                                            // unsupported
                                                        }
                                                        ElementType::Image => {
                                                            break;
                                                        }
                                                    }
                                                    if element.newline {
                                                        ui.end_row();
                                                    }
                                                    idx += 1;
                                                }
                                            });

                                            ui.vertical_centered(|ui| {
                                                while let Some(element) = elements.get(idx) {
                                                    if element.typ != ElementType::Image {
                                                        break;
                                                    }
                                                    if let Some(src) = &element.image_tuple.0 {
                                                        ui.add_space(4.0);
                                                        ui.add(
                                                            Image::from(src)
                                                                .fit_to_original_size(1.0)
                                                                .max_width(
                                                                    match element.image_tuple.1 {
                                                                        Some(width) => f32::min(
                                                                            width,
                                                                            ui.max_rect().width(),
                                                                        ),
                                                                        None => {
                                                                            ui.max_rect().width()
                                                                        }
                                                                    },
                                                                )
                                                                .max_height(
                                                                    match element.image_tuple.2 {
                                                                        Some(height) => height,
                                                                        None => f32::INFINITY,
                                                                    },
                                                                )
                                                                .rounding(
                                                                    Rounding::ZERO.at_least(10.0),
                                                                )
                                                                .show_loading_spinner(true),
                                                        );
                                                        ui.add_space(4.0);
                                                        idx += 1;
                                                    }
                                                }
                                            });
                                        }
                                    } else {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label("No content...");
                                        });
                                    }
                                });
                            });
                    });
                });
        })
        .response
    }
}
