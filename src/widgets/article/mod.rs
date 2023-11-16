mod detail;
mod preview;

use std::cell::RefCell;
use std::rc::Rc;

use crate::article::Article;
use crate::article::ArticleUuid;
use crate::feed::Feed;
use ego_tree::iter::Edge;
use egui::RichText;
use lazy_static::lazy_static;
use regex::Regex;
use scraper;

pub use self::detail::Detail;
pub use self::preview::Preview;

lazy_static! {
    static ref CONTINUOUS_WHITESPACE_PATTERN: Regex = Regex::new(r"\s+").unwrap();
}

#[derive(Clone, PartialEq, Debug)]
enum ElementType {
    Paragraph,
    Heading,
    ListItem,
    Image,
    Separator,
    CodeBlock,
    LineBreak,
    Others,
}

impl Default for ElementType {
    fn default() -> Self {
        ElementType::Others
    }
}

#[derive(Default, Clone)]
struct Element {
    typ: ElementType,
    // boolean variables for text style
    bold: bool,
    code: bool,
    deleted: bool,
    emphasized: bool,
    small: bool,
    strong: bool,
    ol: bool,
    ul: bool,
    newline: bool,
    // data variables for widgets
    text: Option<RichText>,
    // destination url of hyperlinks
    destination: Option<String>,
    // triple tuple of images
    // (src, width, height)
    image_tuple: (Option<String>, Option<f32>, Option<f32>),
    // level of headings
    heading_level: Option<u8>,
}
// style table:
// tag  fontsize    margin      others
// p    16.0        19.2, 0
// h1   32.0        10.72, 0
// h2   24.0        24.0, 0
// h3   18.72       18.72, 0
// h4   16.0        19.2, 0
// h5   13.28       24.0, 0
// h6   10.72       26.72, 0
// li   padding-left: 16.0

impl Element {
    fn new() -> Self {
        Element::default()
    }
}

fn stylize_text(element: &Element, text: String) -> RichText {
    let mut richtext = RichText::new(text).size(16.0);
    if element.bold || element.strong {
        richtext = richtext.strong();
    }
    if element.emphasized {
        richtext = richtext.italics();
    }
    if element.deleted {
        richtext = richtext.strikethrough();
    }
    if element.small {
        richtext = richtext.small()
    }
    if element.code {
        richtext = richtext.code()
    }
    richtext
}

// A builder helps you to get article details and previews.
pub struct Builder<'a> {
    entry_title: Option<String>,
    title: &'a str,
    link: Option<&'a str>,
    updated: Option<&'a str>,
    published: Option<&'a str>,
    elements: Option<Vec<Element>>,
    // For preview
    max_rows: usize,
    break_anywhere: bool,
    overflow_character: Option<char>,
    fulltext: Option<String>,
    article_id: ArticleUuid,
}

impl<'a> Builder<'a> {
    pub fn from_article(
        article: &'a Article,
        article_id: ArticleUuid,
        feed: Rc<RefCell<Feed>>,
    ) -> Self {
        let updated = article.updated.as_ref().map(|s| s.as_str());
        let published = article.published.as_ref().map(|s| s.as_str());
        let title = &article.title;
        let link = article.links.get(0).map(|link| link.as_str());
        let summary = article.summary.as_ref();
        let _catrgories = &article.categories;
        let entry_title = article.belong_to.and_then(|entry_uuid| {
            feed.borrow()
                .try_get_entry_by_id(&entry_uuid)
                .ok()
                .map(|entry_rc| entry_rc.borrow().title().to_owned())
        });
        let _unread = article.unread;

        let (elements, fulltext) = if let Some(summary) = summary {
            let fragment = scraper::Html::parse_fragment(summary);
            let mut dom_stack: Vec<String> = Vec::new();
            let mut elements = Vec::new();
            let mut fulltext = String::new();
            let mut element_stack = vec![Element::new()];

            for edge in fragment.root_element().traverse() {
                match edge {
                    Edge::Open(node) => match node.value() {
                        scraper::Node::Text(ref text) => {
                            if text.trim().len() == 0 {
                                // in case that it is not a meaningless new empty line in html document
                                continue;
                            }
                            let mut element = element_stack.last().unwrap().clone();
                            let text = if !dom_stack.iter().any(|tag| tag == "pre") {
                                // the text is not preformatted
                                // delete continuous whitespace, \n and \r
                                CONTINUOUS_WHITESPACE_PATTERN
                                    .replace_all(&text, " ")
                                    .trim_matches(|ch: char| ch == '\n' || ch == 'r')
                                    .to_owned()
                            } else {
                                // or else, remain the raw text entirely
                                if dom_stack.iter().any(|tag| tag == "code") {
                                    // in case of code blocks
                                    element.typ = ElementType::CodeBlock;
                                }
                                text.to_string()
                            };
                            fulltext += &text;
                            element.text = Some(stylize_text(&element, text));
                            elements.push(element);
                        }
                        scraper::Node::Element(tag) => {
                            dom_stack.push(tag.name().to_owned());
                            let mut element = element_stack.last().cloned().unwrap();
                            match tag.name() {
                                "p" => element.typ = ElementType::Paragraph,
                                "b" => element.bold = true,
                                "code" => element.code = true,
                                "del" => element.deleted = true,
                                "em" => element.emphasized = true,
                                "small" => element.small = true,
                                "strong" => element.strong = true,
                                "a" => {
                                    element.destination =
                                        tag.attr("href").map(|dest| dest.to_owned());
                                }
                                "ol" => element.ol = true,
                                "ul" => element.ul = true,
                                "li" => element.typ = ElementType::ListItem,
                                "hr" => element.typ = ElementType::Separator,
                                "br" => elements.push(Element {
                                    typ: ElementType::LineBreak,
                                    ..Default::default()
                                }),
                                "img" => {
                                    element.typ = ElementType::Image;
                                    element.image_tuple = (
                                        tag.attr("src").map(|s| s.to_owned()),
                                        tag.attr("width").and_then(|s| s.parse::<f32>().ok()),
                                        tag.attr("height").and_then(|s| s.parse::<f32>().ok()),
                                    );
                                    elements.push(element.clone())
                                }
                                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                                    element.typ = ElementType::Heading;
                                    element.heading_level = tag.name()[1..].parse().ok();
                                }
                                _ => {}
                            }
                            // block display tags
                            match tag.name() {
                                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ol" | "ul" | "p"
                                | "hr" | "pre" => element.newline = true,
                                _ => {}
                            }
                            element_stack.push(element);
                        }
                        _ => {}
                    },

                    Edge::Close(node) => {
                        if let scraper::Node::Element(_) = node.value() {
                            element_stack.pop();
                        }
                    }
                }
            }
            (Some(elements), Some(fulltext))
        } else {
            (None, None)
        };
        Builder {
            entry_title,
            title,
            link,
            updated,
            published,
            elements,
            max_rows: 3,
            break_anywhere: true,
            overflow_character: Some('…'),
            fulltext,
            article_id,
        }
    }

    // fn to_preview(self) -> Preview {
    //     Preview::from(self)
    // }
}
