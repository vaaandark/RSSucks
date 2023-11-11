//! Data structures and operating interfaces for Rss feeds.
use crate::opml;
use anyhow::{Context, Error, Ok, Result};
use reqwest::Url;
use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
use std::ops::Deref;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use uuid::Uuid;

/// Universally Unique Identifier for [`Entry`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EntryUuid(Uuid);

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

/// Universally Unique Identifier for [`Folder`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FolderUuid(Uuid);

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

/// OPML head information,
/// which can be converted from [`opml::Head`].
#[derive(Debug)]
pub struct Head {
    pub title: Option<String>,
}

impl From<opml::Head> for Head {
    fn from(value: opml::Head) -> Self {
        Head { title: value.title }
    }
}

/// Feed entry, the basic unit for getting subsciptions from feed,
/// which can be converted from [`opml::Entry`] (see [`Entry::try_from`]).
#[allow(unused)]
#[derive(Debug)]
pub struct Entry {
    /// The title of the feed.
    pub text: String,
    /// Also the title.
    pub title: Option<String>,
    /// URL of the RSS feed.
    pub xml_url: Url,
    /// Homepage URL of the feed.
    pub html_url: Option<Url>,
    /// The UUID of the folder to which this entry belongs.
    /// > Note that if it's `None`, it belongs to no folder and is called **orphan** entry.
    pub belong_to: Option<FolderUuid>,
    /// UUID of this feed.
    pub uuid: EntryUuid,
}

impl Entry {
    /// Creates an `Entry` for a feed.
    #[allow(unused)]
    pub fn new(text: String, xml_url: Url) -> Self {
        Entry {
            title: Some(text.to_owned()),
            text,
            html_url: xml_url.join("/").ok(),
            xml_url,
            belong_to: None,
            uuid: Uuid::new_v4().into(),
        }
    }

    /// Creates an `Entry` with homepage URL of the feed.
    #[allow(unused)]
    pub fn new_with_html_url(text: String, xml_url: Url, html_url: Url) -> Self {
        Entry {
            title: Some(text.to_owned()),
            text,
            html_url: Some(html_url),
            xml_url,
            belong_to: None,
            uuid: Uuid::new_v4().into(),
        }
    }

    /// Set entry belongs to a folder.
    pub fn set_belonging(&mut self, belong_to: &FolderUuid) {
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

/// A folder for containing a series of subscriptions on similar topics,
/// which can **not** be converted from [`opml::Folder`] directly.
#[allow(unused)]
#[derive(Debug)]
pub struct Folder {
    /// The title of the feed.
    pub text: String,
    /// Also the title, can be `None`.
    pub title: Option<String>,
    /// The IDs of entries which belong to this folder.
    pub entries: HashSet<EntryUuid>,
    /// UUID of this feed folder.
    pub uuid: FolderUuid,
}

impl TryFrom<opml::Opml> for Feed {
    type Error = Error;
    fn try_from(value: opml::Opml) -> Result<Self> {
        let version = value.version;
        let head = value.head.map(Head::from);
        let mut orphans = HashSet::new();
        let mut entries_map = HashMap::new();
        let mut folders_map = HashMap::new();
        for outline in value.body.outlines {
            match outline {
                opml::Outline::Entry(e) => {
                    let entry = Rc::new(Entry::try_from(e)?);
                    entries_map.insert(entry.uuid, entry.clone());
                    orphans.insert(entry.uuid);
                }
                opml::Outline::Folder(f) => {
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
                }
            }
        }
        Ok(Feed {
            head,
            version,
            orphans,
            folders_map,
            entries_map,
        })
    }
}

/// Main data structure for RSS feeds,
/// which contains orphan entries directly and folders with entries inside.
/// Feed can be converted from [`opml::Opml`].
#[allow(unused)]
#[derive(Debug)]
pub struct Feed {
    /// OPML version.
    pub version: String,
    /// OPML head.
    pub head: Option<Head>,
    /// IDs of orphan feed entries which don't belong to any folders.
    pub orphans: HashSet<EntryUuid>,
    /// Map for all entries.
    pub entries_map: HashMap<EntryUuid, Rc<Entry>>,
    /// Map for all folders.
    pub folders_map: HashMap<FolderUuid, Rc<RefCell<Folder>>>,
}

impl Feed {
    /// Returns all folders.
    #[allow(unused)]
    pub fn get_all_folders(&self) -> Vec<Rc<RefCell<Folder>>> {
        self.folders_map.values().map(Rc::clone).collect()
    }

    /// Returns the IDs of all folders.
    #[allow(unused)]
    pub fn get_all_folder_ids(&self) -> Vec<FolderUuid> {
        self.folders_map
            .keys()
            .map(FolderUuid::clone)
            .collect::<Vec<_>>()
    }

    /// Attempts to return the IDs of all entries in a folder by giving folder ID.
    #[allow(unused)]
    pub fn try_get_entry_ids_by_folder_id(&self, folder_id: &FolderUuid) -> Result<Vec<EntryUuid>> {
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

    /// Returns all entries.
    #[allow(unused)]
    pub fn get_all_entries(&self) -> Vec<Rc<Entry>> {
        self.entries_map.values().map(Rc::clone).collect()
    }

    /// Returns the title and the feed url of all entries.
    #[allow(unused)]
    pub fn get_all_entry_basic_infos(&self) -> Vec<(String, Url)> {
        self.entries_map
            .values()
            .map(|e| (e.text.to_owned(), e.xml_url.to_owned()))
            .collect()
    }

    /// Returns the IDs of all entries.
    #[allow(unused)]
    pub fn get_all_entry_ids(&self) -> Vec<EntryUuid> {
        self.entries_map.keys().map(EntryUuid::clone).collect()
    }

    /// Returns the IDs of all orphan entries.
    #[allow(unused)]
    pub fn get_all_orphan_entry_ids(&self) -> Vec<EntryUuid> {
        self.orphans.iter().map(EntryUuid::clone).collect()
    }

    /// Attempts to return an folder by giving its ID.
    pub fn try_get_folder_by_id(&self, id: &FolderUuid) -> Result<Rc<RefCell<Folder>>> {
        let folder = self
            .folders_map
            .get(id)
            .with_context(|| format!("Failed to get folder by UUID `{}`", **id))?;
        Ok(folder.clone())
    }

    /// Attempts to return an entry by giving its ID.
    #[allow(unused)]
    pub fn try_get_entry_by_id(&self, id: &EntryUuid) -> Result<Rc<Entry>> {
        let entry = self
            .entries_map
            .get(id)
            .with_context(|| format!("Failed to get entry by UUID `{}`", **id))?;
        Ok(entry.clone())
    }

    /// Attempts to remove an entry by giving its ID.
    #[allow(unused)]
    pub fn try_remove_entry_by_id(&mut self, id: &EntryUuid) -> Result<()> {
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

    /// Attempts to remove a folder by giving its ID.
    #[allow(unused)]
    pub fn try_remove_folder_by_id(&mut self, id: &FolderUuid) -> Result<()> {
        let folder = self.try_get_folder_by_id(id)?;
        // Remove all the entries of the folder from the general map of entries.
        folder
            .try_borrow_mut()
            .with_context(|| format!("Failed to borrow folder (UUID `{}`).", **id))?
            .entries
            .iter()
            .for_each(|e| {
                self.entries_map.remove(e);
            });
        self.folders_map.remove(id);
        Ok(())
    }

    /// Addes an orphan entry which doesn't belong to any folder.
    #[allow(unused)]
    pub fn add_orphan_entry(&mut self, entry: Entry) -> EntryUuid {
        let entry = Rc::new(entry);
        self.entries_map.insert(entry.uuid, entry.clone());
        self.orphans.insert(entry.uuid);
        entry.uuid
    }

    /// Attempts to add an entry belonging to a folder by giving folder ID.
    #[allow(unused)]
    pub fn try_add_entry_to_folder(
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

    /// Attempts to move an entry to another folder or make an entry orphan.
    /// > Note that when `to_folder_id` is `None`, it will attempt to make the
    /// entry belong to **no** folder.
    #[allow(unused)]
    pub fn try_move_entry_to_folder(
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

    /// Returns the IDs of all entries with matching name.
    #[allow(unused)]
    pub fn get_entry_ids_by_name(&self, name: &str) -> Vec<EntryUuid> {
        let mut res = vec![];
        for (id, entry) in &self.entries_map {
            if entry.text.contains(name) {
                res.push(*id);
            }
        }
        res
    }

    /// Returns the IDs of all folders with matching name.
    #[allow(unused)]
    pub fn get_folder_ids_by_name(&self, name: &str) -> Vec<FolderUuid> {
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
