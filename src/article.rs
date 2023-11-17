use crate::feed::EntryUuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Universally Unique Identifier for [`Article`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Deserialize, Serialize)]
pub struct ArticleUuid {
    updated: Option<DateTime<Utc>>,
    published: Option<DateTime<Utc>>,
    feed_id: EntryUuid,
    id: String,
}

impl Ord for ArticleUuid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let get_time_fn = |uuid: &ArticleUuid| {
            if uuid.updated.is_some() {
                uuid.updated
            } else if uuid.published.is_some() {
                uuid.published
            } else {
                None
            }
        };
        match (get_time_fn(self), get_time_fn(other)) {
            (Some(self_time), Some(other_time)) => other_time.cmp(&self_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => self.id.cmp(&other.id),
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
