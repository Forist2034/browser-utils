use chrono::DateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo<S> {
    pub name: S,
    pub vendor: S,
    pub version: S,
    pub build_id: S,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info<S> {
    pub id: Uuid,
    pub hostname: S,
    pub browser: BrowserInfo<S>,
    pub start_time: DateTime<chrono::FixedOffset>,
    pub end_time: Option<DateTime<chrono::FixedOffset>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind<S> {
    Visit { url: Option<S>, title: Option<S> },
    TitleUpdate { visit_event: Uuid, title: Option<S> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<S, BId> {
    pub id: Uuid,
    pub browser_id: BId,
    pub timestamp: DateTime<chrono::FixedOffset>,
    pub event: EventKind<S>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryTitle<S> {
    pub event_id: Uuid,
    pub timestamp: DateTime<chrono::FixedOffset>,
    pub title: Option<S>,
}
#[derive(Debug, Serialize)]
pub struct Entry<S> {
    pub id: Uuid,
    pub visit_event_id: Uuid,
    pub timestamp: DateTime<chrono::FixedOffset>,
    pub url: Option<S>,
    pub titles: Vec<EntryTitle<S>>,
}
