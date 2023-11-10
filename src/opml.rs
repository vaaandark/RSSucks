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

impl From<&Entry> for opml::Outline {
    fn from(value: &Entry) -> Self {
        opml::Outline {
            text: value.text.to_owned(),
            title: value.title.as_ref().map(|t| t.to_owned()),
            xml_url: value.xml_url.as_ref().map(|u| u.as_str().to_owned()),
            html_url: value.html_url.as_ref().map(|u| u.as_str().to_owned()),
            r#type: Some("rss".to_owned()),
            ..Default::default()
        }
    }
}

impl From<&Folder> for opml::Outline {
    fn from(value: &Folder) -> Self {
        let sub_outlines = value
            .entries
            .iter()
            .map(opml::Outline::from)
            .collect::<Vec<_>>();
        opml::Outline {
            text: value.text.to_owned(),
            title: value.title.as_ref().map(|t| t.to_owned()),
            outlines: sub_outlines,
            ..Default::default()
        }
    }
}

impl From<&OutLine> for opml::Outline {
    fn from(value: &OutLine) -> Self {
        match value {
            OutLine::Entry(e) => opml::Outline::from(e),
            OutLine::Folder(f) => opml::Outline::from(f),
        }
    }
}

impl From<&Body> for opml::Body {
    fn from(value: &Body) -> Self {
        opml::Body {
            outlines: value
                .outlines
                .iter()
                .map(opml::Outline::from)
                .collect::<Vec<_>>(),
        }
    }
}

impl From<&Head> for opml::Head {
    fn from(value: &Head) -> Self {
        opml::Head {
            title: value.title.to_owned(),
            ..Default::default()
        }
    }
}

impl From<&opml::Outline> for Entry {
    fn from(value: &opml::Outline) -> Self {
        Entry {
            text: value.text.to_owned(),
            title: value.title.as_ref().map(|t| t.to_owned()),
            xml_url: value.xml_url.as_ref().and_then(|u| Url::parse(u).ok()),
            html_url: value.html_url.as_ref().and_then(|u| Url::parse(u).ok()),
        }
    }
}

impl From<&opml::Outline> for Folder {
    fn from(value: &opml::Outline) -> Self {
        Folder {
            text: value.text.to_owned(),
            title: value.title.as_ref().map(|t| t.to_owned()),
            entries: Opml::flatten_nested_folder(value),
        }
    }
}

impl From<&opml::Body> for Body {
    fn from(value: &opml::Body) -> Self {
        Body {
            outlines: value.outlines.iter().map(OutLine::from).collect::<Vec<_>>(),
        }
    }
}

impl From<&opml::Outline> for OutLine {
    fn from(value: &opml::Outline) -> Self {
        // Is an entry or a folder?
        if value.xml_url.is_some() {
            OutLine::Entry(Entry::from(value))
        } else {
            OutLine::Folder(Folder::from(value))
        }
    }
}

impl From<&opml::Head> for Head {
    fn from(value: &opml::Head) -> Self {
        Head {
            title: value.title.to_owned(),
        }
    }
}

impl From<&OPML> for Opml {
    fn from(value: &OPML) -> Self {
        let version = value.version.to_owned();
        let head = value.head.as_ref().map(Head::from);
        let body = Body::from(&value.body);
        Opml {
            version,
            head,
            body,
        }
    }
}

impl From<&Opml> for OPML {
    fn from(value: &Opml) -> Self {
        let version = value.version.to_owned();
        let head = value.head.as_ref().map(opml::Head::from);
        let body = opml::Body::from(&value.body);
        OPML {
            version,
            head,
            body,
        }
    }
}

impl Opml {
    #[allow(unused)]
    fn flatten_nested_folder(outline: &opml::Outline) -> Vec<Entry> {
        if outline.xml_url.is_some() {
            vec![Entry::from(outline)]
        } else {
            outline
                .outlines
                .iter()
                .flat_map(Self::flatten_nested_folder)
                .collect::<Vec<Entry>>()
        }
    }

    #[allow(unused)]
    fn try_from_str(xml: &str) -> Result<Self> {
        Ok(Opml::from(
            &OPML::from_str(xml).with_context(|| "Failed to parse OPML file.")?,
        ))
    }

    #[allow(unused)]
    fn try_dump(&self) -> Result<String> {
        OPML::from(self)
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
        let opml1 = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        let opml2 = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        assert_eq!(format!("{:?}", opml1), format!("{:?}", opml2));
    }

    #[test]
    fn parse_complex_opml() {
        let opml = Opml::try_from_str(&read_to_string("./OPMLs/complex.opml").unwrap()).unwrap();
        println!("{:?}", opml);
    }

    #[test]
    fn dump_opml() {
        let xml = read_to_string("./OPMLs/example1.opml").unwrap();
        let opml = Opml::try_from_str(&xml).unwrap();
        assert_eq!(xml, opml.try_dump().unwrap());
    }
}
