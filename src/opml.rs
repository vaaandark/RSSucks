use anyhow::{Context, Ok, Result};
use opml::OPML;
use reqwest::Url;

#[derive(Debug)]
#[allow(unused)]
struct Opml {
    pub head: Option<Head>,
    pub body: Body,
}

#[derive(Debug)]
#[allow(unused)]
struct Head {
    pub title: Option<String>,
}

#[derive(Debug)]
#[allow(unused)]
struct Entry {
    pub text: String,
    pub url: Option<Url>,
}

#[derive(Debug)]
#[allow(unused)]
struct Folder {
    pub text: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
#[allow(unused)]
enum OutLine {
    Folder(Folder),
    Entry(Entry),
}

#[derive(Debug)]
#[allow(unused)]
struct Body {
    pub outlines: Vec<OutLine>,
}

impl Opml {
    #[allow(unused)]
    fn flatten_nested_folder(outline: &opml::Outline) -> Vec<Entry> {
        if let Some(url) = outline.xml_url.as_ref() {
            vec![Entry {
                text: outline.text.to_owned(),
                url: Url::parse(url).ok(),
            }]
        } else {
            outline
                .outlines
                .iter()
                .flat_map(Self::flatten_nested_folder)
                .collect::<Vec<Entry>>()
        }
    }

    #[allow(unused)]
    fn from_str(xml: &str) -> Result<Self> {
        let document = OPML::from_str(xml).with_context(|| "Failed to parse OPML file.")?;
        let mut head = document.head.map(|h| Head { title: h.title });
        let mut outlines = vec![];
        for outline in document.body.outlines {
            // is an entry?
            if let Some(url) = outline.xml_url.as_ref() {
                let entry = Entry {
                    text: outline.text,
                    url: Url::parse(url).ok(),
                };
                outlines.push(OutLine::Entry(entry));
            } else {
                // a folder?
                let folder = Folder {
                    text: outline.text.to_owned(),
                    entries: Self::flatten_nested_folder(&outline),
                };
                outlines.push(OutLine::Folder(folder));
            }
        }
        let body = Body { outlines };
        Ok(Opml { head, body })
    }
}

#[cfg(test)]
mod test {
    use crate::opml::Opml;
    use std::fs::read_to_string;

    #[test]
    fn parse_opml() {
        let opml1 = Opml::from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        let opml2 = Opml::from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        assert_eq!(format!("{:?}", opml1), format!("{:?}", opml2));
    }

    #[test]
    fn parse_complex_opml() {
        let opml = Opml::from_str(&read_to_string("./OPMLs/complex.opml").unwrap()).unwrap();
        println!("{:?}", opml);
    }
}
