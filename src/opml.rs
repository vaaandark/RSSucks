use anyhow::{Context, Ok, Result};
use opml::OPML;

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
    pub url: String,
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
    fn flat_nested_folder(folder: &mut Folder, outline: &opml::Outline) {
        for o in &outline.outlines {
            Self::flat_nested_folder(folder, o);
        }
        if let Some(url) = outline.xml_url.as_ref() {
            folder.entries.push(Entry {
                text: outline.text.to_owned(),
                url: url.to_owned(),
            });
        }
    }

    #[allow(unused)]
    fn from_str(xml: &str) -> Result<Self> {
        let document = OPML::from_str(xml).with_context(|| "Failed to parse OPML file.")?;
        let mut head = None;
        if let Some(doc_head) = document.head {
            head = Some(Head {
                title: doc_head.title,
            })
        }
        let mut outlines = vec![];
        for outline in document.body.outlines {
            // is an entry?
            if let Some(url) = outline.xml_url {
                let entry = Entry {
                    text: outline.text,
                    url,
                };
                outlines.push(OutLine::Entry(entry));
            } else {
                // a folder?
                let mut folder = Folder {
                    text: outline.text.to_owned(),
                    entries: vec![],
                };
                Self::flat_nested_folder(&mut folder, &outline);
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
