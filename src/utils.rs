pub mod rss_client {
    use anyhow::{anyhow, Result};
    use ehttp;
    use feed_rs;
    use serde::{Deserialize, Serialize};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use uuid::Uuid;

    #[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
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

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Folder {
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

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Feed {
        pub id: FeedId,
        pub folder_id: Option<FolderId>,
        pub url: url::Url,

        pub is_syncing: bool,

        #[serde(skip)]
        pub model: Option<feed_rs::model::Feed>,
    }

    impl Feed {
        pub fn new_with_url(url: url::Url) -> Self {
            let id = FeedId::new();
            Self {
                id,
                folder_id: None,
                url,
                is_syncing: false,
                model: None,
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

        pub fn get_name(&self) -> String {
            self.model
                .as_ref()
                .and_then(|feed| feed.title.as_ref())
                .map(|title| title.content.to_string())
                .unwrap_or(self.url.to_string())
        }
    }

    pub struct Entry {
        pub id: EntryId,
        pub feed_id: FeedId,
        pub model: feed_rs::model::Entry,
    }

    #[derive(Clone, Default, Serialize, Deserialize)]
    pub struct RssClient {
        folders: Arc<Mutex<HashMap<FolderId, Folder>>>,
        feeds: Arc<Mutex<HashMap<FeedId, Feed>>>,

        #[serde(skip)]
        entry_cache: Arc<Mutex<HashMap<EntryId, Entry>>>,
    }

    impl RssClient {
        pub fn create_folder(&self, name: impl ToString) -> FolderId {
            let folder = Folder::new_with_name(name);
            let id = folder.id;
            self.folders.lock().unwrap().insert(id, folder);
            id
        }

        pub fn add_folder(&self, folder: Folder) -> Option<Folder> {
            self.folders.lock().unwrap().insert(folder.id, folder)
        }

        pub fn delete_folder(&self, id: FolderId) -> Option<Folder> {
            for feed_id in self.list_feed_by_folder(id) {
                self.set_feed_folder_id(&feed_id, None);
            }

            self.folders.lock().unwrap().remove(&id)
        }

        pub fn get_folder(&self, id: &FolderId) -> Option<Folder> {
            self.folders.lock().unwrap().get(id).cloned()
        }

        pub fn list_folder(&self) -> Vec<FolderId> {
            self.folders.lock().unwrap().keys().cloned().collect()
        }

        pub fn create_feed(&self, url: url::Url) -> FeedId {
            let feed = Feed::new_with_url(url);
            let id = feed.id;
            self.feeds.lock().unwrap().insert(id, feed);
            id
        }

        pub fn create_feed_with_folder(
            &self,
            url: url::Url,
            folder_id: Option<FolderId>,
        ) -> FeedId {
            let mut feed = Feed::new_with_url(url);
            feed.folder_id = folder_id;
            let id = feed.id;

            self.feeds.lock().unwrap().insert(id, feed);
            id
        }

        pub fn set_feed_folder_id(&self, id: &FeedId, folder_id: Option<FolderId>) -> &Self {
            self.feeds.lock().unwrap().get_mut(id).unwrap().folder_id = folder_id;
            return self;
        }

        pub fn get_feed(&self, id: &FeedId) -> Option<Feed> {
            self.feeds.lock().unwrap().get(id).cloned()
        }

        pub fn add_feed(&self, feed: Feed) -> Option<Feed> {
            self.feeds.lock().unwrap().insert(feed.id, feed)
        }

        pub fn replace_feed_model(
            &self,
            id: FeedId,
            model: feed_rs::model::Feed,
        ) -> Option<feed_rs::model::Feed> {
            self.feeds
                .lock()
                .unwrap()
                .get_mut(&id)
                .unwrap()
                .model
                .replace(model)
        }

        fn feed_set_syncing(&self, id: FeedId) {
            self.feeds.lock().unwrap().get_mut(&id).unwrap().is_syncing = true
        }

        fn feed_set_not_syncing(&self, id: FeedId) {
            self.feeds.lock().unwrap().get_mut(&id).unwrap().is_syncing = false
        }

        pub fn feed_is_syncing(&self, id: FeedId) -> bool {
            self.feeds.lock().unwrap().get(&id).unwrap().is_syncing
        }

        pub fn delete_feed(&self, id: FeedId) -> Option<Feed> {
            self.feeds.lock().unwrap().remove(&id)
        }

        pub fn list_feed(&self) -> Vec<FeedId> {
            self.feeds.lock().unwrap().keys().cloned().collect()
        }

        pub fn list_orphan_feed(&self) -> Vec<FeedId> {
            self.feeds
                .lock()
                .unwrap()
                .iter()
                .filter_map(|(id, feed)| (feed.folder_id == None).then_some(*id))
                .collect()
        }

        pub fn list_feed_by_folder(&self, folder_id: FolderId) -> Vec<FeedId> {
            self.feeds
                .lock()
                .unwrap()
                .iter()
                .filter_map(|(id, feed)| (feed.folder_id == Some(folder_id)).then_some(*id))
                .collect()
        }

        pub fn try_start_sync_folder(&self, id: FolderId) -> Result<()> {
            for feed_id in self.list_feed_by_folder(id) {
                self.try_start_sync_feed(feed_id)?;
            }

            Ok(())
        }

        pub fn try_start_sync_feed(&self, id: FeedId) -> Result<()> {
            if self.feed_is_syncing(id) {
                return Ok(());
            }
            self.feed_set_syncing(id);
            let feed = self.get_feed(&id).unwrap();

            let client = self.clone();

            ehttp::fetch(
                ehttp::Request::get(feed.url.to_string().as_str()),
                move |result| {
                    client.feed_set_not_syncing(id);
                    let model = feed_rs::parser::parse_with_uri(
                        std::io::Cursor::new(result.expect("failed to get response").bytes),
                        Some(feed.url.as_str()),
                    )
                    .unwrap();
                    client.replace_feed_model(id, model);
                },
            );

            Ok(())
        }
    }
}
