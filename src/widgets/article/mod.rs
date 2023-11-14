use crate::article::Article;
use crate::feed::Feed;
use ego_tree::iter::Edge;
use lazy_static::lazy_static;
use regex::Regex;
use scraper;

lazy_static! {
    static ref CONTINUOUS_WHITESPACE_PATTERN: Regex = Regex::new(r"\s+").unwrap();
}

#[derive(Default, Clone)]
struct Element {
    // boolean variables for text style
    bold: bool,
    code: bool,
    deleted: bool,
    emphasized: bool,
    small: bool,
    strong: bool,
    hyperlink: bool,
    ol: bool,
    ul: bool,
    li: bool,
    // these variables are not for text style
    // they represents the existence of the components
    separator: bool,
    newline: bool,
    // data variables for other widgets
    text: Option<String>,
    // destination url of hyperlinks
    destination: Option<String>,
    // triple tuple of images
    // (src, width, height)
    image_tuple: (Option<String>, Option<f32>, Option<f32>),
    // level of headings
    heading: Option<u8>,
}

impl Element {
    fn new() -> Self {
        Element::default()
    }
}

// A builder helps you to get article details and previews.
pub struct Builder<'a> {
    entry_title: Option<&'a str>,
    title: &'a str,
    links: &'a Vec<String>,
    updated: Option<&'a str>,
    published: Option<&'a str>,
    elements: Option<Vec<Element>>,
    // For preview
    max_rows: usize,
    break_anywhere: bool,
    overflow_character: Option<char>,
    fulltext: Option<String>,
}

impl<'a> Builder<'a> {
    pub fn from_article(article: &Article, feed: &Feed) -> Self {
        let updated = article.updated.map(|s| s.as_str());
        let published = article.published.map(|s| s.as_str());
        let title = &article.title;
        let links = &article.links;
        let summary = article.summary.as_ref();
        let catrgories = article.categories;
        let entry_title = article.belong_to.and_then(|entry_uuid| {
            feed.try_get_entry_by_id(&entry_uuid)
                .ok()
                .map(|entry_rc| entry_rc.borrow().title())
        });
        let unread = article.unread;

        let (elements, fulltext) = if let Some(summary) = summary {
            let fragment = scraper::Html::parse_fragment(summary);
            let mut dom_stack: Vec<String> = Vec::new();
            let mut elements: Vec<_> = Vec::new();
            let mut fulltext = String::new();
            let mut element_stack: Vec<_> = vec![Element::new()];

            for edge in fragment.root_element().traverse() {
                match edge {
                    Edge::Open(node) => match node.value() {
                        scraper::Node::Text(ref text) => {
                            if text.trim().len() == 0 {
                                // in case that it is not a meaningless new empty line in html document
                                continue;
                            }
                            let text = if !dom_stack.iter().any(|tag| tag == "pre") {
                                // the text is not preformatted
                                // delete continuous whitespace, \n and \r
                                CONTINUOUS_WHITESPACE_PATTERN
                                    .replace_all(&text, " ")
                                    .trim_matches(|ch: char| ch == '\n' || ch == 'r')
                                    .to_owned()
                            } else {
                                // or else, remain the raw text entirely
                                text.to_string()
                            };
                            fulltext += &text;
                            // TODO: remove unwrap
                            elements.push(element_stack.last().unwrap().clone())
                        }
                        scraper::Node::Element(tag) => {
                            dom_stack.push(tag.name().to_owned());
                            let mut element = element_stack.last().unwrap().clone();
                            match tag.name() {
                                "b" => element.bold = true,
                                "code" => element.code = true,
                                "del" => element.deleted = true,
                                "em" => element.emphasized = true,
                                "small" => element.small = true,
                                "strong" => element.strong = true,
                                "a" => {
                                    element.hyperlink = true;
                                    element.destination =
                                        tag.attr("href").map(|dest| dest.to_owned());
                                }
                                "ol" => element.ol = true,
                                "ul" => element.ul = true,
                                "li" => element.li = true,
                                "hr" => element.separator = true,
                                "br" => element.newline = true,
                                "img" => {
                                    element.image_tuple = (
                                        tag.attr("src").map(|s| s.to_owned()),
                                        tag.attr("width").and_then(|s| s.parse::<f32>().ok()),
                                        tag.attr("height").and_then(|s| s.parse::<f32>().ok()),
                                    )
                                }
                                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                                    element.heading = tag.name()[1..].parse().ok()
                                }
                            }
                            element_stack.push(element);
                        }
                        _ => {}
                    },

                    Edge::Close(node) => {
                        if let scraper::Node::Element(tag) = node.value() {
                            element_stack.pop();
                            match tag.name() {
                                // block display tags
                                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ol" | "ul" | "p"
                                | "hr" | "pre" => elements.push(Element {
                                    newline: true,
                                    ..Element::default()
                                }),
                            }
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
            links,
            updated,
            published,
            elements,
            max_rows: 3,
            break_anywhere: true,
            overflow_character: Some('…'),
            fulltext,
        }
    }
}
