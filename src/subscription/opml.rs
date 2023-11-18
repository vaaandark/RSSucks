//! Wrapper for external crate [`opml`].
use anyhow::{Context, Ok, Result};
use opml::OPML;
use url::Url;

/// Main data structure for OPML,
/// which can be converted from [`opml::OPML`].
#[derive(Debug)]
#[allow(unused)]
pub struct Opml {
    pub version: String,
    pub head: Option<Head>,
    pub body: Body,
}

/// OPML head, which can be converted from [`opml::Head`].
#[derive(Debug)]
#[allow(unused)]
pub struct Head {
    pub title: Option<String>,
}

/// Feed entry, which can be converted from [`opml::Outline`].
#[derive(Debug)]
#[allow(unused)]
pub struct Entry {
    pub text: String,
    pub title: Option<String>,
    pub xml_url: Option<Url>,
    pub html_url: Option<Url>,
}

/// A folder for containing a series of subscriptions on similar topics,
/// which can be converted from [`opml::Outline`]
#[derive(Debug)]
#[allow(unused)]
pub struct Folder {
    pub text: String,
    pub title: Option<String>,
    pub entries: Vec<Entry>,
}

/// OPML outline, which can be converted from [`opml::Outline`],
/// and can be a folder or an entry.
#[derive(Debug)]
#[allow(unused)]
pub enum Outline {
    Folder(Folder),
    Entry(Entry),
}

/// OPML body, which can be converted from [`opml::Body`].
#[derive(Debug)]
#[allow(unused)]
pub struct Body {
    pub outlines: Vec<Outline>,
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

impl From<&Outline> for opml::Outline {
    fn from(value: &Outline) -> Self {
        match value {
            Outline::Entry(e) => opml::Outline::from(e),
            Outline::Folder(f) => opml::Outline::from(f),
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
            outlines: value.outlines.iter().map(Outline::from).collect::<Vec<_>>(),
        }
    }
}

impl From<&opml::Outline> for Outline {
    fn from(value: &opml::Outline) -> Self {
        // Is an entry or a folder?
        if value.xml_url.is_some() {
            Outline::Entry(Entry::from(value))
        } else {
            Outline::Folder(Folder::from(value))
        }
    }
}

impl From<super::feed::Entry> for Entry {
    fn from(value: super::feed::Entry) -> Self {
        Entry {
            text: value.title(),
            title: Some(value.title()),
            xml_url: Some(value.xml_url),
            html_url: value.html_url,
        }
    }
}

impl From<super::feed::Head> for Head {
    fn from(value: super::feed::Head) -> Self {
        Head { title: value.title }
    }
}

impl From<super::feed::Feed> for Opml {
    fn from(value: super::feed::Feed) -> Self {
        let version = value.version.to_owned();
        let head = value.head.to_owned().map(Head::from);
        let mut outlines = vec![];
        for orphan_id in value.get_all_orphan_entry_ids() {
            let orphan = value.try_get_entry_by_id(&orphan_id).unwrap();
            let orphan_entry = Entry::from(orphan.borrow().to_owned());
            outlines.push(Outline::Entry(orphan_entry));
        }
        for folder_id in value.get_all_folder_ids() {
            let folder = value.try_get_folder_by_id(&folder_id).unwrap();
            let mut folder_outlines = vec![];
            for entry_id in value.try_get_entry_ids_by_folder_id(&folder_id).unwrap() {
                let entry = Entry::from(
                    value
                        .try_get_entry_by_id(&entry_id)
                        .unwrap()
                        .borrow()
                        .to_owned(),
                );
                folder_outlines.push(entry);
            }
            let folder = Folder {
                text: folder.borrow().title().to_owned(),
                title: Some(folder.borrow().title().to_owned()),
                entries: folder_outlines,
            };
            outlines.push(Outline::Folder(folder));
        }
        Opml {
            version,
            head,
            body: Body { outlines },
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

    /// Attempts to parse a OPML XML file.
    #[allow(unused)]
    pub fn try_from_str(xml: &str) -> Result<Self> {
        Ok(Opml::from(
            &OPML::from_str(xml).context("Failed to parse OPML file.")?,
        ))
    }

    /// Attempts to dump to a OPML XML file.
    #[allow(unused)]
    pub fn try_dump(&self) -> Result<String> {
        OPML::from(self).to_string().context("Failed to dump OPML.")
    }
}

#[cfg(test)]
mod test {
    use crate::subscription::opml::Opml;
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
