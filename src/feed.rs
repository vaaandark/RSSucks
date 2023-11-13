//! Data structures and operating interfaces for Rss feeds.
use crate::article::{Article, ArticleUuid};
use crate::opml;
use anyhow::{anyhow, Context, Error, Ok, Result};
use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::{cell::RefCell, rc::Rc};
use url::Url;
use uuid::Uuid;

/// Universally Unique Identifier for [`Entry`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct Entry {
    /// The title of the feed.
    text: String,
    /// Also the title, can be `None`,
    /// for compatibility with the OPML standard.
    title: Option<String>,
    /// URL of the RSS feed.
    pub xml_url: Url,
    /// Homepage URL of the feed.
    pub html_url: Option<Url>,
    /// The IDs of entries which belong to this folder.
    articles: Arc<Mutex<BTreeSet<ArticleUuid>>>,
    /// The UUID of the folder to which this entry belongs.
    /// > Note that if it's `None`, it belongs to no folder and is called **orphan** entry.
    belong_to: Option<FolderUuid>,
    /// UUID of this feed.
    uuid: EntryUuid,
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
            articles: Arc::new(Mutex::new(BTreeSet::new())),
            belong_to: None,
            uuid: Uuid::new_v4().into(),
        }
    }

    /// Sets homepage URL of a entry.
    #[allow(unused)]
    pub fn set_html_url(mut self, html_url: Url) -> Self {
        self.html_url = Some(html_url);
        self
    }

    /// Set entry belongs to a folder.
    /// > Note that this function just sets entry's belonging attribute,
    /// > so it won't affect the folder with the id.
    pub fn set_belonging(mut self, belong_to: &FolderUuid) -> Self {
        self.belong_to = Some(*belong_to);
        self
    }

    /// Returns the title of the entry.
    #[allow(unused)]
    pub fn title(&self) -> &str {
        &self.text
    }

    /// Set the title of the entry.
    #[allow(unused)]
    pub fn rename(&mut self, name: String) {
        if self.title.is_some() {
            self.title = Some(name.to_owned());
        }
        self.text = name;
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
            articles: Arc::new(Mutex::new(BTreeSet::new())),
            title: value.title,
            html_url: value.html_url,
            belong_to: None,
        })
    }
}

/// A folder for containing a series of subscriptions on similar topics,
/// which can **not** be converted from [`opml::Folder`] directly.
#[allow(unused)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Folder {
    /// The title of the feed.
    text: String,
    /// Also the title, can be `None`,
    /// for compatibility with the OPML standard.
    title: Option<String>,
    /// The IDs of entries which belong to this folder.
    entries: HashSet<EntryUuid>,
    /// UUID of this feed folder.
    uuid: FolderUuid,
}

impl Folder {
    /// Returns the title of the folder.
    #[allow(unused)]
    pub fn title(&self) -> &str {
        &self.text
    }

    /// Set the title of the folder.
    #[allow(unused)]
    pub fn rename(&mut self, name: String) {
        if self.title.is_some() {
            self.title = Some(name.to_owned());
        }
        self.text = name;
    }

    /// Returns the IDs of all entries in the folder.
    #[allow(unused)]
    pub fn get_entry_ids(&self) -> impl Iterator<Item = &EntryUuid> {
        self.entries.iter()
    }
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
                    let entry = Entry::try_from(e)?;
                    let uuid = entry.uuid;
                    let entry = Rc::new(RefCell::new(entry));
                    entries_map.insert(uuid, entry);
                    orphans.insert(uuid);
                }
                opml::Outline::Folder(f) => {
                    let uuid = Uuid::new_v4().into();
                    let mut entries = HashSet::new();
                    for e in f.entries {
                        let entry = Entry::try_from(e)
                            .with_context(|| format!("At folder {}", f.text))?
                            .set_belonging(&uuid);
                        let uuid = entry.uuid;
                        let entry = Rc::new(RefCell::new(entry));
                        entries_map.insert(uuid, entry);
                        entries.insert(uuid);
                    }
                    let folder = Rc::new(RefCell::new(Folder {
                        text: f.text,
                        title: f.title,
                        entries,
                        uuid,
                    }));
                    folders_map.insert(uuid, folder);
                }
            }
        }
        Ok(Feed {
            head,
            version,
            orphans,
            folders_map,
            entries_map,
            articles_map: Arc::new(Mutex::new(BTreeMap::new())),
        })
    }
}

type ArticlesMap = Arc<Mutex<BTreeMap<ArticleUuid, Arc<Mutex<RefCell<Article>>>>>>;

/// Main data structure for RSS feeds,
/// which contains orphan entries directly and folders with entries inside.
/// Feed can be converted from [`opml::Opml`].
#[allow(unused)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Feed {
    /// OPML version.
    pub version: String,
    /// OPML head.
    pub head: Option<Head>,
    /// IDs of orphan feed entries which don't belong to any folders.
    orphans: HashSet<EntryUuid>,
    /// Map for all entries.
    entries_map: HashMap<EntryUuid, Rc<RefCell<Entry>>>,
    /// Map for all folders.
    folders_map: HashMap<FolderUuid, Rc<RefCell<Folder>>>,
    /// Map for all articles.
    articles_map: ArticlesMap,
}

impl Feed {
    /// Returns all folders.
    #[allow(unused)]
    pub fn get_all_folders(&self) -> impl Iterator<Item = &Rc<RefCell<Folder>>> {
        self.folders_map.values()
    }

    /// Returns the IDs of all folders.
    #[allow(unused)]
    pub fn get_all_folder_ids(&self) -> Vec<FolderUuid> {
        self.folders_map.keys().map(FolderUuid::clone).collect()
    }

    /// Returns all entries.
    #[allow(unused)]
    pub fn get_all_entries(&self) -> impl Iterator<Item = &Rc<RefCell<Entry>>> {
        self.entries_map.values()
    }

    /// Returns the title and the feed url of all entries.
    #[allow(unused)]
    pub fn get_all_entry_basic_infos(&self) -> impl Iterator<Item = (String, Url)> + '_ {
        self.entries_map
            .values()
            .map(|e| (e.borrow().text.to_owned(), e.borrow().xml_url.to_owned()))
    }

    /// Returns the IDs of all entries.
    #[allow(unused)]
    pub fn get_all_entry_ids(&self) -> Vec<EntryUuid> {
        self.entries_map.keys().map(EntryUuid::clone).collect()
    }

    /// Returns the IDs of all entries in the folder.
    #[allow(unused)]
    pub fn try_get_entry_ids_by_folder_id(&self, folder_id: &FolderUuid) -> Result<Vec<EntryUuid>> {
        let folder = self.try_get_folder_by_id(folder_id)?;
        let entry_ids = folder
            .try_borrow_mut()
            .with_context(|| format!("Failed to borrow folder (UUID `{}`).", **folder_id))?
            .entries
            .iter()
            .map(EntryUuid::clone)
            .collect();
        Ok(entry_ids)
    }

    /// Returns the IDs of all orphan entries.
    #[allow(unused)]
    pub fn get_all_orphan_entry_ids(&self) -> Vec<EntryUuid> {
        self.orphans.iter().map(EntryUuid::clone).collect()
    }

    /// Attempts to return an folder by giving its ID.
    pub fn try_get_folder_by_id(&self, id: &FolderUuid) -> Result<Rc<RefCell<Folder>>> {
        Ok(self
            .folders_map
            .get(id)
            .with_context(|| format!("Failed to get folder by UUID `{}`", **id))?
            .clone())
    }

    /// Attempts to return an entry by giving its ID.
    #[allow(unused)]
    pub fn try_get_entry_by_id(&self, id: &EntryUuid) -> Result<Rc<RefCell<Entry>>> {
        Ok(self
            .entries_map
            .get(id)
            .with_context(|| format!("Failed to get entry by UUID `{}`", **id))?
            .clone())
    }

    /// Attempts to remove an entry by giving its ID.
    #[allow(unused)]
    pub fn try_remove_entry_by_id(&mut self, id: &EntryUuid) -> Result<Rc<RefCell<Entry>>> {
        let entry = self.try_get_entry_by_id(id)?;
        // Belong to a folder?
        if let Some(belong_to) = &entry
            .try_borrow()
            .with_context(|| format!("Failed to borrow entry (UUID `{}`).", **id))?
            .belong_to
        {
            self.try_remove_entry_id_from_folder_set(id, belong_to);
        } else {
            self.orphans.remove(id);
        }
        self.entries_map.remove(id);
        Ok(entry)
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
        let uuid = entry.uuid;
        let entry = Rc::new(RefCell::new(entry));
        self.entries_map.insert(uuid, entry);
        self.orphans.insert(uuid);
        uuid
    }

    /// Attempts to add an entry belonging to a folder by giving folder ID.
    #[allow(unused)]
    pub fn try_add_entry_to_folder(
        &mut self,
        entry: Entry,
        to_folder_uuid: &FolderUuid,
    ) -> Result<EntryUuid> {
        let uuid = entry.uuid;
        let entry = Rc::new(RefCell::new(entry.set_belonging(to_folder_uuid)));
        self.entries_map.insert(uuid, entry);
        self.try_add_entry_id_to_folder_set(&uuid, to_folder_uuid);
        Ok(uuid)
    }

    fn try_remove_entry_id_from_folder_set(
        &mut self,
        entry_id: &EntryUuid,
        old_folder_id: &FolderUuid,
    ) -> Result<()> {
        let old_folder = self.try_get_folder_by_id(old_folder_id)?;
        old_folder
            .try_borrow_mut()
            .with_context(|| format!("Failed to borrow folder (UUID `{}`).", **old_folder_id))?
            .entries
            .remove(entry_id);
        Ok(())
    }

    fn try_add_entry_id_to_folder_set(
        &mut self,
        entry_id: &EntryUuid,
        new_folder_id: &FolderUuid,
    ) -> Result<()> {
        let new_folder = self.try_get_folder_by_id(new_folder_id)?;
        new_folder
            .try_borrow_mut()
            .with_context(|| format!("Failed to borrow folder (UUID `{}`).", **new_folder_id))?
            .entries
            .insert(*entry_id);
        Ok(())
    }

    /// Attempts to move an entry to another folder or make an entry orphan.
    /// > Note that when `to_folder_id` is `None`, it will attempt to make the
    /// entry belong to **no** folder.
    #[allow(unused)]
    pub fn try_move_entry_to_folder(
        &mut self,
        entry_id: &EntryUuid,
        to_folder_id: Option<&FolderUuid>,
    ) -> Result<Rc<RefCell<Entry>>> {
        let unborrowed = self.try_get_entry_by_id(entry_id)?;
        let binding = unborrowed.clone();
        let mut entry = binding
            .try_borrow_mut()
            .with_context(|| format!("Failed to borrow entry (UUID `{}`).", **entry_id))?;
        match (&entry.belong_to, to_folder_id) {
            // From a folder to another folder.
            (Some(old_folder_id), Some(new_folder_id)) => {
                // Remove from old folder.
                self.try_remove_entry_id_from_folder_set(entry_id, old_folder_id)?;
                // Insert to new folder.
                self.try_add_entry_id_to_folder_set(entry_id, new_folder_id)?;
            }
            // From a folder to be an orphan.
            (Some(old_folder_id), None) => {
                // Remove from old folder.
                self.try_remove_entry_id_from_folder_set(entry_id, old_folder_id)?;
                self.orphans.insert(*entry_id);
            }
            // From an orphan to be owned by a folder.
            (None, Some(new_folder_id)) => {
                self.orphans.remove(entry_id);
                // Insert to new folder.
                self.try_add_entry_id_to_folder_set(entry_id, new_folder_id)?;
            }
            _ => (),
        }
        entry.belong_to = to_folder_id.map(FolderUuid::clone);
        Ok(unborrowed)
    }

    /// Returns the IDs of all entries with matching name.
    #[allow(unused)]
    pub fn get_entry_ids_by_name(&self, name: &str) -> Vec<EntryUuid> {
        self.entries_map
            .iter()
            .filter_map(move |(id, entry)| {
                if entry.borrow().text.contains(name) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the IDs of all folders with matching name.
    #[allow(unused)]
    pub fn get_folder_ids_by_name(&self, name: &str) -> Vec<FolderUuid> {
        self.folders_map
            .iter()
            .filter_map(move |(id, folder)| {
                if folder.borrow().text.contains(name) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Attempts to sync articles of a entry by giveing its ID.
    pub fn try_sync_entry_by_id(&mut self, id: &EntryUuid) -> Result<()> {
        let binding = self.try_get_entry_by_id(id)?;
        let entry = binding.try_borrow()?;
        let article_id_set = entry.articles.clone();
        let article_map = self.articles_map.to_owned();
        let url = entry.xml_url.to_string();
        let entry_uuid = entry.uuid;
        ehttp::fetch(ehttp::Request::get(url.as_str()), move |result| {
            let feed = feed_rs::parser::parse_with_uri(
                std::io::Cursor::new(result.expect("Failed to get response.").bytes),
                Some(url.as_str()),
            )
            .expect("Failed to parse feed.");
            feed.entries.iter().for_each(|item| {
                let article_id = ArticleUuid::new(&entry_uuid, item.id.to_owned());
                let mut article_id_set = article_id_set
                    .lock()
                    .expect("Failed to get the lock on article id set.");
                if !article_id_set.contains(&article_id) {
                    article_id_set.insert(article_id.clone());
                    article_map
                        .lock()
                        .expect("Failed to get the lock on article map")
                        .insert(
                            article_id,
                            Arc::new(Mutex::new(RefCell::new(
                                Article::from(item.to_owned()).set_belonging(&entry_uuid),
                            ))),
                        );
                }
            });
        });
        Ok(())
    }

    /// Attempts to sync articles of all entries.
    #[allow(unused)]
    pub fn try_sync_all(&mut self) -> Result<()> {
        let entry_ids = self.get_all_entry_ids();
        entry_ids.iter().for_each(|id| {
            self.try_sync_entry_by_id(id);
        });
        Ok(())
    }

    /// Returns the IDs of all articles.
    #[allow(unused)]
    pub fn get_all_article_ids(&self) -> Vec<ArticleUuid> {
        self.articles_map
            .lock()
            .expect("Failed to get the lock on article map")
            .keys()
            .map(ArticleUuid::clone)
            .collect()
    }

    /// Attempts to return the article by giveing its ID.
    #[allow(unused)]
    pub fn try_get_article_by_id(&self, id: &ArticleUuid) -> Result<Arc<Mutex<RefCell<Article>>>> {
        Ok(self
            .articles_map
            .lock()
            .expect("Failed to get the lock on article map")
            .get(id)
            .map(Arc::clone)
            .ok_or(anyhow!("Invalid article id"))?)
    }

    /// Attempts to return the IDs of all articles in a feed entry by giving entry ID.
    #[allow(unused)]
    pub fn try_get_all_article_ids_by_entry_id(
        &self,
        entry_id: &EntryUuid,
    ) -> Result<Vec<ArticleUuid>> {
        let entry = self.try_get_entry_by_id(entry_id)?;
        let article_ids = entry
            .borrow()
            .articles
            .lock()
            .expect("Failed to get the lock on article id set.")
            .iter()
            .map(ArticleUuid::clone)
            .collect();
        Ok(article_ids)
    }

    /// Attempts to return the IDs of all articles in a feed folder by giving folder ID.
    #[allow(unused)]
    pub fn get_all_article_ids_by_folder_id(
        &self,
        folder_id: &FolderUuid,
    ) -> Result<Vec<ArticleUuid>> {
        let entry_ids = self.try_get_entry_ids_by_folder_id(folder_id)?;
        let article_ids = entry_ids
            .iter()
            .flat_map(|entry_id| self.try_get_all_article_ids_by_entry_id(entry_id).unwrap())
            .collect();
        Ok(article_ids)
    }
}

#[cfg(test)]
mod test {
    use crate::feed::Entry;
    use crate::feed::Feed;
    use crate::opml::Opml;
    use std::fs::read_to_string;
    use url::Url;

    #[test]
    fn new_entry() {
        let entry1 = Entry::new(
            "sspai".to_owned(),
            Url::parse("https://sspai.com/feed").unwrap(),
        );
        let entry2 = Entry::new(
            "sspai".to_owned(),
            Url::parse("https://sspai.com/feed").unwrap(),
        )
        .set_html_url(Url::parse("https://sspai.com").unwrap());
        assert_eq!(
            format!("{:?}", entry1.html_url),
            format!("{:?}", entry2.html_url)
        );
        assert_ne!(entry1.uuid, entry2.uuid);
    }

    #[test]
    fn list_entries_in_folder() {
        let opml = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        let feed = Feed::try_from(opml).unwrap();
        let binding = feed.get_folder_ids_by_name("Software");
        let folder_id = binding.first().unwrap();
        let mut names = feed
            .try_get_folder_by_id(folder_id)
            .unwrap()
            .borrow()
            .get_entry_ids()
            .map(|id| {
                feed.try_get_entry_by_id(id)
                    .unwrap()
                    .borrow()
                    .text
                    .to_owned()
            })
            .collect::<Vec<_>>();
        names.sort();
        let mut expect = vec!["小众软件", "异次元软件世界"];
        expect.sort();
        assert_eq!(expect, names);
    }

    #[test]
    fn parse_opml() {
        let opml = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        Feed::try_from(opml).unwrap();
    }

    impl Feed {
        fn get_sorted_all_entry_basic_infos(&self) -> Vec<(String, Url)> {
            let mut names = self.get_all_entry_basic_infos().collect::<Vec<_>>();
            names.sort();
            names
        }
    }

    #[test]
    fn remove_entry() {
        let opml1 = Opml::try_from_str(&read_to_string("./OPMLs/example1.opml").unwrap()).unwrap();
        let mut feed1: Feed = opml1.try_into().unwrap();
        let found = *feed1.get_entry_ids_by_name("少数派").first().unwrap();
        feed1.try_remove_entry_by_id(&found).unwrap();
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
        let to_folder_id = *feed2.get_folder_ids_by_name("Software").first().unwrap();
        let entry_id = feed2.try_add_entry_to_folder(entry, &to_folder_id).unwrap();
        // Before move:
        // The newly created entry should belong to a folder.
        assert!(feed2
            .try_get_entry_by_id(&entry_id)
            .unwrap()
            .borrow()
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
            .borrow()
            .belong_to
            .is_none());
        // Feed should have an orphan entrie.
        assert_eq!(feed2.orphans.len(), 1);
        assert_eq!(
            feed1.get_sorted_all_entry_basic_infos(),
            feed2.get_sorted_all_entry_basic_infos()
        );
    }

    #[test]
    fn feed_update() {
        let opml = Opml::try_from_str(&read_to_string("./OPMLs/complex.opml").unwrap()).unwrap();
        let mut feed = Feed::try_from(opml).unwrap();
        feed.try_sync_all().unwrap();
        for i in 0..20 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let fetched_article = feed.articles_map.lock().unwrap().len();
            println!("Fetched {fetched_article} articles in {i} secs.");
        }
    }
}
