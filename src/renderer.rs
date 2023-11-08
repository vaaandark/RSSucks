use anyhow::Result;
use ego_tree::iter::Edge;
use egui::Separator;
use scraper::Html;

// let mut feeds: Vec<feed_rs::model::Feed> = Vec::new();

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
    Others,
}

fn richtext_generator(text: &str, stack: &Vec<ElementType<'_>>) -> Result<egui::RichText> {
    let richtext = stack.iter().fold(
        egui::RichText::new(text),
        |richtext, element| match element {
            ElementType::H1 => richtext.heading().size(32.0),
            ElementType::H2 => richtext.heading().size(24.0),
            ElementType::H3 => richtext.heading().size(18.72),
            ElementType::H4 => richtext.heading().size(16.0),
            ElementType::H5 => richtext.heading().size(13.28),
            ElementType::H6 => richtext.heading().size(10.72),
            ElementType::Em => richtext.italics(),
            ElementType::Strong => richtext.strong(),
            ElementType::Code => richtext.code(),
            _ => richtext,
        },
    );
    Ok(richtext)
}

pub fn render_article_html_to_component(html: &str, ui: &mut egui::Ui) -> Result<()> {
    let fragment = Html::parse_fragment(html);
    let mut stack: Vec<_> = Vec::new();
    let edges: Vec<_> = fragment.root_element().traverse().collect();
    for edge in edges {
        match edge {
            Edge::Open(node) => {
                println!("{:?}, {:?}", node.value(), stack);
                match node.value() {
                    scraper::Node::Text(text) => {
                        let text = text.replace("\n", "");
                        if text.is_empty() {
                            continue;
                        }
                        let richtext = richtext_generator(&text, &stack)?;
                        // println!("{:?}", text);
                        let hyperlink_destination = stack.iter().fold(None, |dest, element| {
                            if let &ElementType::A { destination } = element {
                                destination
                            } else {
                                dest
                            }
                        });
                        if let Some(dest) = hyperlink_destination {
                            ui.hyperlink_to(richtext, dest);
                        } else {
                            ui.label(richtext);
                        }
                    }
                    scraper::Node::Element(tag) => match tag.name() {
                        "p" => stack.push(ElementType::P),
                        "h1" => stack.push(ElementType::H1),
                        "h2" => stack.push(ElementType::H2),
                        "h3" => stack.push(ElementType::H3),
                        "h4" => stack.push(ElementType::H4),
                        "h5" => stack.push(ElementType::H5),
                        "h6" => stack.push(ElementType::H6),
                        "a" => stack.push(ElementType::A {
                            destination: tag.attr("href"),
                        }),
                        "img" => {
                            stack.push(ElementType::Img);
                            // let (src, width, height) = (tag.attr("src"), tag.attr("width"), tag.attr("height"));
                            // ui.image(texture_id, size)
                        }
                        "em" => stack.push(ElementType::Em),
                        "strong" => stack.push(ElementType::Strong),
                        "hr" => {
                            stack.push(ElementType::Hr);
                            ui.add(Separator::horizontal(Separator::default()));
                        }
                        "code" => stack.push(ElementType::Code),
                        "br" => {
                            stack.push(ElementType::Br);
                            ui.end_row();
                        }
                        _ => stack.push(ElementType::Others),
                    },
                    _ => {}
                }
            }

            Edge::Close(node) => {
                println!("{:?}, {:?}", node.value(), stack);
				match node.value() {
					scraper::Node::Element(tag) => {
						stack.pop();
						if stack.len() == 1 || tag.name() == "li" {
							ui.end_row();
						}
					},
					_ => {}
				}
            }
        }
    }
    Ok(())
}