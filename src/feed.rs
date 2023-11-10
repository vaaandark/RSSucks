use crate::opml;
use anyhow::{anyhow, Context, Error, Ok, Result};
use reqwest::Url;
use std::cmp::{Eq, PartialEq};
use std::ops::Deref;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct EntryUuid(Uuid);

impl Deref for EntryUuid {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Uuid> for EntryUuid {
    fn from(value: Uuid) -> Self {
        EntryUuid(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct FolderUuid(Uuid);

impl Deref for FolderUuid {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Uuid> for FolderUuid {
    fn from(value: Uuid) -> Self {
        FolderUuid(value)
    }
}

#[derive(Debug)]
pub struct Head {
    pub title: Option<String>,
}

impl From<opml::Head> for Head {
    fn from(value: opml::Head) -> Self {
        Head { title: value.title }
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Entry {
    pub text: String,
    pub title: Option<String>,
    pub xml_url: Url,
    pub html_url: Option<Url>,
    uuid: EntryUuid,
}

impl Entry {
    #[allow(unused)]
    fn new(text: String, xml_url: Url) -> Self {
        Entry {
            title: Some(text.to_owned()),
            text,
            html_url: xml_url.join("/").ok(),
            xml_url,
            uuid: Uuid::new_v4().into(),
        }
    }

    #[allow(unused)]
    fn new_detailed(text: String, xml_url: Url, html_url: Url) -> Self {
        Entry {
            title: Some(text.to_owned()),
            text,
            html_url: Some(html_url),
            xml_url,
            uuid: Uuid::new_v4().into(),
        }
    }
}

impl TryFrom<opml::Entry> for Entry {
    type Error = Error;
    fn try_from(value: opml::Entry) -> Result<Self> {
        Ok(Entry {
            xml_url: value.xml_url.with_context(|| {
                format!("Doesn't exist or is an invalid XML URL at {}", value.text)
            })?,
            text: value.text,
            uuid: Uuid::new_v4().into(),
            title: value.title,
            html_url: value.html_url,
        })
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Folder {
    text: String,
    title: Option<String>,
    entries: Vec<Rc<Entry>>,
    uuid: FolderUuid,
}

impl Folder {
    fn add_entry(&mut self, entry: Rc<Entry>) {
        self.entries.push(entry)
    }
}

impl TryFrom<opml::Folder> for Folder {
    type Error = Error;
    fn try_from(value: opml::Folder) -> Result<Self> {
        let mut entries = vec![];
        for entry in value.entries {
            entries.push(Rc::new(
                Entry::try_from(entry).with_context(|| format!("At folder {}", value.text))?,
            ));
        }
        Ok(Folder {
            text: value.text,
            title: value.title,
            entries,
            uuid: Uuid::new_v4().into(),
        })
    }
}

impl TryFrom<opml::Opml> for Feed {
    type Error = Error;
    fn try_from(value: opml::Opml) -> Result<Self> {
        let version = value.version;
        let head = value.head.map(Head::from);
        let mut folders = vec![];
        let mut orphans = vec![];
        let mut entries_map = HashMap::new();
        let mut folders_map = HashMap::new();
        for outline in value.body.outlines {
            match outline {
                opml::OutLine::Entry(e) => {
                    let entry = Rc::new(Entry::try_from(e)?);
                    entries_map.insert(entry.uuid, entry.clone());
                    orphans.push(entry);
                }
                opml::OutLine::Folder(f) => {
                    let folder = Folder::try_from(f)?;
                    for entry in &folder.entries {
                        entries_map.insert(entry.uuid, entry.clone());
                    }
                    let uuid = folder.uuid;
                    let folder = Rc::new(RefCell::new(folder));
                    folders_map.insert(uuid, folder.clone());
                    folders.push(folder);
                }
            }
        }
        Ok(Feed {
            head,
            version,
            folders,
            orphans,
            folders_map,
            entries_map,
        })
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Feed {
    version: String,
    head: Option<Head>,
    folders: Vec<Rc<RefCell<Folder>>>,
    orphans: Vec<Rc<Entry>>,
    entries_map: HashMap<EntryUuid, Rc<Entry>>,
    folders_map: HashMap<FolderUuid, Rc<RefCell<Folder>>>,
}

impl Feed {
    #[allow(unused)]
    fn add_entry(&mut self, entry: Entry) -> EntryUuid {
        let entry = Rc::new(entry);
        self.entries_map.insert(entry.uuid, entry.clone());
        self.orphans.push(entry.clone());
        entry.uuid
    }

    #[allow(unused)]
    fn try_add_entry_to_folder(
        &mut self,
        entry: Entry,
        to_folder_uuid: FolderUuid,
    ) -> Result<EntryUuid> {
        let entry = Rc::new(entry);
        self.entries_map.insert(entry.uuid, entry.clone());
        let to_folder = self
            .folders_map
            .get(&to_folder_uuid)
            .ok_or(anyhow!("No such folder UUID `{}`", *to_folder_uuid))?;
        to_folder
            .try_borrow_mut()
            .context("Failed to get folder by UUID")?
            .add_entry(entry.clone());
        Ok(entry.uuid)
    }
}

#[cfg(test)]
mod test {
    use crate::feed::Entry;
    use crate::feed::Feed;
    use crate::opml::Opml;
    use reqwest::Url;
    use std::fs::read_to_string;

    #[test]
    fn new_entry() {
        let entry1 = Entry::new(
            "sspai".to_owned(),
            Url::parse("https://sspai.com/feed").unwrap(),
        );
        let entry2 = Entry::new_detailed(
            "sspai".to_owned(),
            Url::parse("https://sspai.com/feed").unwrap(),
            Url::parse("https://sspai.com").unwrap(),
        );
        assert_eq!(
            format!("{:?}", entry1.html_url),
            format!("{:?}", entry2.html_url)
        );
        assert_ne!(entry1.uuid, entry2.uuid);
    }

    #[test]
    fn parse_opml() {
        let opml = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        let feed: Feed = opml.try_into().unwrap();
        println!("{:?}", feed);
    }
}
