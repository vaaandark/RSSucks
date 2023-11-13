pub mod rss_client {
    use anyhow::{anyhow, Result};
    use ehttp;
    use feed_rs;
    use std::collections::HashSet;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use uuid::Uuid;

    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub struct FolderId(Uuid);

    impl From<Uuid> for FolderId {
        fn from(value: Uuid) -> Self {
            Self(value)
        }
    }

    impl FolderId {
        pub fn new() -> Self {
            Self::from(Uuid::new_v4())
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub struct FeedId(Uuid);

    impl From<Uuid> for FeedId {
        fn from(value: Uuid) -> Self {
            Self(value)
        }
    }

    impl FeedId {
        pub fn new() -> Self {
            Self::from(Uuid::new_v4())
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub struct EntryId(Uuid);

    impl From<Uuid> for EntryId {
        fn from(value: Uuid) -> Self {
            Self(value)
        }
    }

    impl EntryId {
        pub fn new() -> Self {
            Self::from(Uuid::new_v4())
        }
    }

    struct Folder {
        pub id: FolderId,
        pub name: String,
    }

    impl Folder {
        pub fn new_with_name(name: impl ToString) -> Self {
            let id = FolderId::new();
            Self {
                id,
                name: name.to_string(),
            }
        }
    }

    struct Feed {
        pub id: FeedId,
        pub folder_id: Option<FolderId>,
        pub url: url::Url,
        pub model: Arc<Mutex<Option<feed_rs::model::Feed>>>,
    }

    impl Feed {
        pub fn new_with_url(url: url::Url) -> Self {
            let id = FeedId::new();
            Self {
                id,
                folder_id: None,
                url,
                model: Arc::new(Mutex::new(None)),
            }
        }

        pub fn move_no_folder(&mut self) -> &mut Self {
            self.folder_id = None;
            self
        }

        pub fn move_to_folder(&mut self, folder_id: FolderId) -> &mut Self {
            self.folder_id = Some(folder_id);
            self
        }
    }

    struct Entry {
        pub id: EntryId,
        pub feed_id: FeedId,
        pub model: feed_rs::model::Entry,
    }

    #[derive(Default)]
    pub struct RssClient {
        folders: HashMap<FolderId, Folder>,
        feeds: HashMap<FeedId, Feed>,
        entry_cache: Arc<Mutex<HashMap<EntryId, Entry>>>,
    }

    impl RssClient {
        pub fn create_folder(&mut self, name: impl ToString) -> FolderId {
            let folder = Folder::new_with_name(name);
            let id = folder.id;
            self.folders.insert(id, folder);
            id
        }

        pub fn add_folder(&mut self, folder: Folder) -> Option<Folder> {
            self.folders.insert(folder.id, folder)
        }

        pub fn delete_folder(&mut self, id: FolderId) -> Option<Folder> {
            self.folders.remove(&id)
        }

        pub fn list_folder(&self) -> impl Iterator + '_ {
            self.folders.iter()
        }

        pub fn create_feed(&mut self, url: url::Url) -> FeedId {
            let feed = Feed::new_with_url(url);
            let id = feed.id;
            self.feeds.insert(id, feed);
            id
        }

        pub fn add_feed(&mut self, feed: Feed) -> Option<Feed> {
            self.feeds.insert(feed.id, feed)
        }

        pub fn delete_feed(&mut self, id: FeedId) -> Option<Feed> {
            self.feeds.remove(&id)
        }

        pub fn list_feed(&self) -> impl Iterator + '_ {
            self.feeds.iter()
        }

        pub fn try_start_sync_feed(&self, id: FeedId) -> Result<()> {
            let feed = self
                .feeds
                .get(&id)
                .ok_or_else(|| anyhow!("feed not found"))?;
            let url = feed.url.to_string();
            let model = Arc::clone(&feed.model);
            let entry_cache = Arc::clone(&self.entry_cache);

            ehttp::fetch(ehttp::Request::get(url.as_str()), move |result| {
                let entry = feed_rs::parser::parse_with_uri(
                    std::io::Cursor::new(result.expect("failed to get response").bytes),
                    Some(url.as_str()),
                )
                .expect("failed to parse feed");
                model
                    .lock()
                    .expect("rare error : peer thread panic detected")
                    .replace(entry);
            });

            Ok(())
        }
    }
}
