use crate::opml;
use anyhow::{Context, Error, Ok, Result};
use reqwest::Url;
use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
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
    pub belong_to: Option<FolderUuid>,
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
            belong_to: None,
            uuid: Uuid::new_v4().into(),
        }
    }

    #[allow(unused)]
    fn new_with_html_url(text: String, xml_url: Url, html_url: Url) -> Self {
        Entry {
            title: Some(text.to_owned()),
            text,
            html_url: Some(html_url),
            xml_url,
            belong_to: None,
            uuid: Uuid::new_v4().into(),
        }
    }

    fn set_belonging(&mut self, belong_to: &FolderUuid) {
        self.belong_to = Some(*belong_to);
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
            belong_to: None,
        })
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Folder {
    text: String,
    title: Option<String>,
    entries: HashSet<EntryUuid>,
    uuid: FolderUuid,
}

impl TryFrom<opml::Opml> for Feed {
    type Error = Error;
    fn try_from(value: opml::Opml) -> Result<Self> {
        let version = value.version;
        let head = value.head.map(Head::from);
        let mut folders = HashSet::new();
        let mut orphans = HashSet::new();
        let mut entries_map = HashMap::new();
        let mut folders_map = HashMap::new();
        for outline in value.body.outlines {
            match outline {
                opml::OutLine::Entry(e) => {
                    let entry = Rc::new(Entry::try_from(e)?);
                    entries_map.insert(entry.uuid, entry.clone());
                    orphans.insert(entry.uuid);
                }
                opml::OutLine::Folder(f) => {
                    let uuid = Uuid::new_v4().into();
                    let mut entries = HashSet::new();
                    for e in f.entries {
                        let mut entry =
                            Entry::try_from(e).with_context(|| format!("At folder {}", f.text))?;
                        entry.set_belonging(&uuid);
                        let entry = Rc::new(entry);
                        entries_map.insert(entry.uuid, entry.clone());
                        entries.insert(entry.uuid);
                    }
                    let folder = Folder {
                        text: f.text,
                        title: f.title,
                        entries,
                        uuid,
                    };
                    let folder = Rc::new(RefCell::new(folder));
                    folders_map.insert(uuid, folder.clone());
                    folders.insert(uuid);
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
    folders: HashSet<FolderUuid>,
    orphans: HashSet<EntryUuid>,
    entries_map: HashMap<EntryUuid, Rc<Entry>>,
    folders_map: HashMap<FolderUuid, Rc<RefCell<Folder>>>,
}

impl Feed {
    #[allow(unused)]
    fn get_all_folders(&self) -> Vec<Rc<RefCell<Folder>>> {
        self.folders_map.values().map(Rc::clone).collect()
    }

    #[allow(unused)]
    fn get_all_folder_ids(&self) -> Vec<FolderUuid> {
        self.folders_map
            .keys()
            .map(FolderUuid::clone)
            .collect::<Vec<_>>()
    }

    #[allow(unused)]
    fn try_get_entry_ids_by_folder_id(&self, folder_id: &FolderUuid) -> Result<Vec<EntryUuid>> {
        let folder = self.try_get_folder_by_id(folder_id)?;
        let entries = folder
            .try_borrow_mut()
            .with_context(|| format!("Failed to borrow folder (UUID `{}`).", **folder_id))?
            .entries
            .iter()
            .map(EntryUuid::clone)
            .collect();
        Ok(entries)
    }

    #[allow(unused)]
    fn get_all_entries(&self) -> Vec<Rc<Entry>> {
        self.entries_map.values().map(Rc::clone).collect()
    }

    #[allow(unused)]
    fn get_all_entry_basic_infos(&self) -> Vec<(String, Url)> {
        self.entries_map
            .values()
            .map(|e| (e.text.to_owned(), e.xml_url.to_owned()))
            .collect()
    }

    #[allow(unused)]
    fn get_all_entry_ids(&self) -> Vec<EntryUuid> {
        self.entries_map.keys().map(EntryUuid::clone).collect()
    }

    #[allow(unused)]
    fn get_all_orphan_entry_ids(&self) -> Vec<EntryUuid> {
        self.orphans.iter().map(EntryUuid::clone).collect()
    }

    fn try_get_folder_by_id(&self, id: &FolderUuid) -> Result<Rc<RefCell<Folder>>> {
        let folder = self
            .folders_map
            .get(id)
            .with_context(|| format!("Failed to get folder by UUID `{}`", **id))?;
        Ok(folder.clone())
    }

    #[allow(unused)]
    fn try_get_entry_by_id(&self, id: &EntryUuid) -> Result<Rc<Entry>> {
        let entry = self
            .entries_map
            .get(id)
            .with_context(|| format!("Failed to get entry by UUID `{}`", **id))?;
        Ok(entry.clone())
    }

    #[allow(unused)]
    fn try_remove_entry_by_id(&mut self, id: &EntryUuid) -> Result<()> {
        let entry = self.try_get_entry_by_id(id)?;
        // Belong to a folder?
        if let Some(belong_to) = &entry.belong_to {
            let folder = self.try_get_folder_by_id(belong_to).with_context(|| {
                format!(
                    "Entry {} (UUID `{}`) belongs to a invalid folder.",
                    entry.text, *entry.uuid
                )
            })?;
            folder
                .try_borrow_mut()
                .with_context(|| format!("Failed to borrow folder (UUID `{}`).", **belong_to))?
                .entries
                .remove(id);
        } else {
            self.orphans.remove(id);
        }
        self.entries_map.remove(id);
        Ok(())
    }

    #[allow(unused)]
    fn try_remove_folder_by_id(&mut self, id: &FolderUuid) -> Result<()> {
        let folder = self.try_get_folder_by_id(id)?;
        folder
            .try_borrow_mut()
            .with_context(|| format!("Failed to borrow folder (UUID `{}`).", **id))?
            .entries
            .iter()
            .for_each(|e| {
                self.entries_map.remove(e);
            });
        self.folders.remove(id);
        self.folders_map.remove(id);
        Ok(())
    }

    #[allow(unused)]
    fn add_orphan_entry(&mut self, entry: Entry) -> EntryUuid {
        let entry = Rc::new(entry);
        self.entries_map.insert(entry.uuid, entry.clone());
        self.orphans.insert(entry.uuid);
        entry.uuid
    }

    #[allow(unused)]
    fn try_add_entry_to_folder(
        &mut self,
        entry: Entry,
        to_folder_uuid: &FolderUuid,
    ) -> Result<EntryUuid> {
        let mut entry = entry;
        let folder = self
            .try_get_folder_by_id(to_folder_uuid)
            .context("Failed when adding entry.")?;
        entry.set_belonging(to_folder_uuid);
        let entry = Rc::new(entry);
        self.entries_map.insert(entry.uuid, entry.clone());
        Ok(entry.uuid)
    }

    #[allow(unused)]
    fn try_move_entry_to_folder(
        &mut self,
        entry_id: &EntryUuid,
        to_folder_id: Option<&FolderUuid>,
    ) -> Result<()> {
        let mut entry = self.try_get_entry_by_id(entry_id)?;
        // Will be moved to a folder?
        if let Some(to_folder_id) = to_folder_id {
            // If targeted folder is invalid, just return.
            let _ = self.try_get_folder_by_id(to_folder_id)?;
        }
        // Belong to a folder?
        if let Some(from_folder_id) = &entry.belong_to {
            let from_folder = self.try_get_folder_by_id(from_folder_id)?;
            from_folder
                .try_borrow_mut()
                .with_context(|| format!("Failed to borrow folder (UUID `{}`).", **from_folder_id))?
                .entries
                .remove(entry_id);
        } else if to_folder_id.is_some() {
            // Is orphan and will be moved to a folder.
            self.orphans.remove(entry_id);
        } else {
            // Is orphan and will still be orphan. Do nothing and return.
            return Ok(());
        }
        // Move to targeted folder.
        // TODO: Ugly implementation, should be changed someday.
        unsafe {
            let mut entry = (entry.as_ref() as *const Entry as *mut Entry);
            (*entry).belong_to = to_folder_id.map(FolderUuid::clone);
        }
        // to be orphan?
        if to_folder_id.is_none() {
            self.orphans.insert(*entry_id);
        }
        Ok(())
    }

    #[allow(unused)]
    fn get_entry_ids_by_name(&self, name: &str) -> Vec<EntryUuid> {
        let mut res = vec![];
        for (id, entry) in &self.entries_map {
            if entry.text.contains(name) {
                res.push(*id);
            }
        }
        res
    }

    #[allow(unused)]
    fn get_folder_ids_by_name(&self, name: &str) -> Vec<FolderUuid> {
        let mut res = vec![];
        for (id, folder) in &self.folders_map {
            if folder.borrow().text.contains(name) {
                res.push(*id);
            }
        }
        res
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
        let entry2 = Entry::new_with_html_url(
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
        Feed::try_from(opml).unwrap();
    }

    impl Feed {
        fn get_sorted_all_entry_basic_infos(&self) -> Vec<(String, Url)> {
            let mut names = self.get_all_entry_basic_infos();
            names.sort();
            names
        }
    }

    #[test]
    fn remove_entry() {
        let opml1 = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        let mut feed1: Feed = opml1.try_into().unwrap();
        let found = feed1.get_entry_ids_by_name("少数派");
        feed1
            .try_remove_entry_by_id(found.first().unwrap())
            .unwrap();
        let opml2 = Opml::try_from_str(&read_to_string("./OPMLs/example3.opml").unwrap()).unwrap();
        let feed2: Feed = opml2.try_into().unwrap();
        assert_eq!(
            feed1.get_sorted_all_entry_basic_infos(),
            feed2.get_sorted_all_entry_basic_infos()
        );
    }

    #[test]
    fn add_entry() {
        let opml1 = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        let feed1: Feed = opml1.try_into().unwrap();
        let opml2 = Opml::try_from_str(&read_to_string("./OPMLs/example3.opml").unwrap()).unwrap();
        let mut feed2: Feed = opml2.try_into().unwrap();
        let entry = Entry::new(
            "少数派".to_owned(),
            Url::parse("https://sspai.com/feed").unwrap(),
        );
        feed2.add_orphan_entry(entry);
        assert_eq!(
            feed1.get_sorted_all_entry_basic_infos(),
            feed2.get_sorted_all_entry_basic_infos()
        );
    }

    #[test]
    fn move_entry_to_another() {
        let opml1 = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        let feed1: Feed = opml1.try_into().unwrap();
        let opml2 = Opml::try_from_str(&read_to_string("./OPMLs/example3.opml").unwrap()).unwrap();
        let mut feed2: Feed = opml2.try_into().unwrap();
        let entry = Entry::new(
            "少数派".to_owned(),
            Url::parse("https://sspai.com/feed").unwrap(),
        );
        let found = feed2.get_folder_ids_by_name("Software");
        let to_folder_id = found.first().unwrap();
        let entry_id = feed2.try_add_entry_to_folder(entry, to_folder_id).unwrap();
        // Before move:
        // The newly created entry should belong to a folder.
        assert!(feed2
            .try_get_entry_by_id(&entry_id)
            .unwrap()
            .belong_to
            .is_some());
        // Feed should have no orphan entries.
        assert_eq!(feed2.orphans.len(), 0);
        assert_eq!(
            feed1.get_sorted_all_entry_basic_infos(),
            feed2.get_sorted_all_entry_basic_infos()
        );
        feed2.try_move_entry_to_folder(&entry_id, None).unwrap();
        // Before move:
        // The newly moved entry should be orphan.
        assert!(feed2
            .try_get_entry_by_id(&entry_id)
            .unwrap()
            .belong_to
            .is_none());
        // Feed should have an orphan entrie.
        assert_eq!(feed2.orphans.len(), 1);
        assert_eq!(
            feed1.get_sorted_all_entry_basic_infos(),
            feed2.get_sorted_all_entry_basic_infos()
        );
    }
}
