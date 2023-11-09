use anyhow::{Context, Ok, Result};
use opml::OPML;
use reqwest::Url;

#[derive(Debug)]
#[allow(unused)]
struct Opml {
    pub version: String,
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
    pub title: Option<String>,
    pub xml_url: Option<Url>,
    pub html_url: Option<Url>,
}

#[derive(Debug)]
#[allow(unused)]
struct Folder {
    pub text: String,
    pub title: Option<String>,
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
                title: outline.title.as_ref().map(|t| t.to_owned()),
                xml_url: Url::parse(url).ok(),
                html_url: outline.html_url.as_ref().and_then(|u| Url::parse(u).ok()),
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
        let version = document.version;
        let mut head = document.head.map(|h| Head { title: h.title });
        let mut outlines = vec![];
        for outline in document.body.outlines {
            // is an entry?
            if let Some(url) = outline.xml_url.as_ref() {
                let entry = Entry {
                    text: outline.text,
                    title: outline.title.as_ref().map(|t| t.to_owned()),
                    xml_url: Url::parse(url).ok(),
                    html_url: outline.html_url.as_ref().and_then(|u| Url::parse(u).ok()),
                };
                outlines.push(OutLine::Entry(entry));
            } else {
                // a folder?
                let folder = Folder {
                    text: outline.text.to_owned(),
                    title: outline.title.as_ref().map(|t| t.to_owned()),
                    entries: Self::flatten_nested_folder(&outline),
                };
                outlines.push(OutLine::Folder(folder));
            }
        }
        let body = Body { outlines };
        Ok(Opml {
            version,
            head,
            body,
        })
    }

    #[allow(unused)]
    fn to_string(&self) -> Result<String> {
        let version = self.version.to_owned();
        let head = self.head.as_ref().map(|h| opml::Head {
            title: h.title.to_owned(),
            ..Default::default()
        });
        let body = opml::Body {
            outlines: self
                .body
                .outlines
                .iter()
                .map(|outline| match outline {
                    OutLine::Folder(f) => {
                        let sub_outlines = f
                            .entries
                            .iter()
                            .map(|e| opml::Outline {
                                text: e.text.to_owned(),
                                title: e.title.as_ref().map(|t| t.to_owned()),
                                xml_url: e.xml_url.as_ref().map(|u| u.as_str().to_owned()),
                                html_url: e.html_url.as_ref().map(|u| u.as_str().to_owned()),
                                r#type: Some("rss".to_owned()),
                                ..Default::default()
                            })
                            .collect::<Vec<_>>();
                        opml::Outline {
                            text: f.text.to_owned(),
                            title: f.title.as_ref().map(|t| t.to_owned()),
                            outlines: sub_outlines,
                            ..Default::default()
                        }
                    }
                    OutLine::Entry(e) => opml::Outline {
                        text: e.text.to_owned(),
                        title: e.title.as_ref().map(|t| t.to_owned()),
                        xml_url: e.xml_url.as_ref().map(|u| u.as_str().to_owned()),
                        html_url: e.html_url.as_ref().map(|u| u.as_str().to_owned()),
                        r#type: Some("rss".to_owned()),
                        ..Default::default()
                    },
                })
                .collect::<Vec<_>>(),
        };
        OPML {
            version,
            head,
            body,
        }
        .to_string()
        .with_context(|| "Failed to dump OPML.")
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

    #[test]
    fn dump_opml() {
        let xml = read_to_string("./OPMLs/example1.opml").unwrap();
        let opml = Opml::from_str(&xml).unwrap();
        assert_eq!(xml, opml.to_string().unwrap());
    }
}
