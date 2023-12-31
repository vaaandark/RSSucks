use super::feed::EntryUuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Universally Unique Identifier for [`Article`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ArticleUuid {
    updated: Option<DateTime<Utc>>,
    published: Option<DateTime<Utc>>,
    feed_id: EntryUuid,
    id: String,
}

impl PartialOrd for ArticleUuid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArticleUuid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.updated.cmp(&self.updated) {
            std::cmp::Ordering::Equal => match other.published.cmp(&self.published) {
                std::cmp::Ordering::Equal => self.id.cmp(&other.id),
                std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            },
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
        }
    }
}

impl ArticleUuid {
    pub fn new(
        update: Option<DateTime<Utc>>,
        publish: Option<DateTime<Utc>>,
        feed_id: &EntryUuid,
        id: impl ToString,
    ) -> Self {
        Self {
            updated: update,
            published: publish,
            feed_id: *feed_id,
            id: id.to_string(),
        }
    }
}

/// Article, which can be convertec from [`feed_rs::model::Entry`]
#[derive(Debug, Deserialize, Serialize)]
pub struct Article {
    pub updated: Option<String>,
    pub published: Option<String>,
    pub id: String,
    pub title: String,
    pub links: Vec<String>,
    pub summary: Option<String>,
    pub categories: Vec<String>,
    pub belong_to: Option<EntryUuid>,
    pub unread: bool,
}

fn utc_to_local_date_string(time_utc: Option<DateTime<Utc>>) -> Option<String> {
    time_utc.map(|time_utc| {
        time_utc
            .with_timezone(&chrono::Local)
            .format("%Y/%m/%d %H:%M")
            .to_string()
    })
}

impl From<feed_rs::model::Entry> for Article {
    fn from(value: feed_rs::model::Entry) -> Self {
        Article {
            id: value.id,
            title: value
                .title
                .map_or("No Title".to_owned(), |text| text.content),
            updated: utc_to_local_date_string(value.updated),
            links: value.links.into_iter().map(|link| link.href).collect(),
            summary: value.summary.map(|summary| summary.content),
            categories: value
                .categories
                .into_iter()
                .filter_map(|category| category.label)
                .collect(),
            published: utc_to_local_date_string(value.published),
            belong_to: None,
            unread: true,
        }
    }
}

impl Article {
    #[allow(unused)]
    pub fn set_belonging(mut self, id: &EntryUuid) -> Self {
        self.belong_to = Some(*id);
        self
    }

    #[allow(unused)]
    pub fn set_read(&mut self) {
        self.unread = false;
    }
}

#[cfg(test)]
mod test {}
