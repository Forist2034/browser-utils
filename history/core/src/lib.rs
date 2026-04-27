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
pub struct Event<Ev> {
    pub id: Uuid,
    pub timestamp: DateTime<chrono::FixedOffset>,
    pub event: Ev,
}
