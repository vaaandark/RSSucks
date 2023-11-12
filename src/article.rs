use crate::feed::EntryUuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Universally Unique Identifier for [`Article`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ArticleUuid {
    updated: Option<DateTime<Utc>>,
    feed_id: EntryUuid,
    id: String,
}

impl ArticleUuid {
    pub fn new(updated: Option<DateTime<Utc>>, feed_id: &EntryUuid, id: String) -> Self {
        Self {
            updated,
            feed_id: *feed_id,
            id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Article {
    pub updated: Option<DateTime<Utc>>,
    pub published: Option<DateTime<Utc>>,
    pub id: String,
    pub title: String,
    pub links: String,
    pub summary: Option<String>,
    pub categories: Vec<String>,
    pub belong_to: Option<EntryUuid>,
    pub unread: bool,
}

impl From<feed_rs::model::Entry> for Article {
    fn from(value: feed_rs::model::Entry) -> Self {
        Article {
            id: value.id,
            title: value
                .title
                .map_or("No Title".to_owned(), |text| text.content),
            updated: value.updated,
            links: value.links.into_iter().map(|link| link.href).collect(),
            summary: value.summary.map(|summary| summary.content),
            categories: value
                .categories
                .into_iter()
                .filter_map(|category| category.label)
                .collect(),
            published: value.published,
            belong_to: None,
            unread: true,
        }
    }
}

impl Article {
    #[allow(unused)]
    fn set_belonging(mut self, id: &EntryUuid) -> Self {
        self.belong_to = Some(*id);
        self
    }
}

#[cfg(test)]
mod test {}
