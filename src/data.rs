use anyhow::Result;
use jiff::{civil::DateTime, tz::TimeZone, Zoned};
use serde::{Deserialize, Serialize};

fn deserialize_dt<'de, D>(deserializer: D) -> Result<Option<Zoned>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|strtime| {
            let dt = DateTime::strptime("%Y-%m-%dT%H:%M:%S%.3f%z", strtime).map_err(serde::de::Error::custom)?;
            let zoned = dt.to_zoned(TimeZone::UTC).map_err(serde::de::Error::custom)?;
            zoned
                .intz(TimeZone::system().iana_name().expect("Couldn't get system tz name"))
                .map_err(serde::de::Error::custom)
        })
        .transpose()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
    pub closed: Option<bool>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "viewMode")]
    pub view_mode: Option<String>,
    pub permission: Option<String>,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChecklistItem {
    pub id: String,
    pub title: String,
    pub status: i32,
    #[serde(rename = "completedTime", deserialize_with = "deserialize_dt", default)]
    pub completed_time: Option<Zoned>,
    #[serde(rename = "isAllDay")]
    pub is_all_day: bool,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
    #[serde(rename = "startDate", deserialize_with = "deserialize_dt", default)]
    pub start_date: Option<Zoned>,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub title: String,
    #[serde(rename = "isAllDay")]
    pub is_all_day: bool,
    #[serde(rename = "completedTime", deserialize_with = "deserialize_dt", default)]
    pub completed_time: Option<Zoned>,
    pub content: String,
    pub desc: String,
    #[serde(rename = "dueDate", deserialize_with = "deserialize_dt", default)]
    pub due_date: Option<Zoned>,
    pub items: Option<Vec<ChecklistItem>>,
    pub priority: i32,
    pub reminders: Option<Vec<String>>,
    #[serde(rename = "repeatFlag")]
    pub repeat_flag: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
    #[serde(rename = "startDate", deserialize_with = "deserialize_dt", default)]
    pub start_date: Option<Zoned>,
    pub status: u32,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Column {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub name: String,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectData {
    pub project: Project,
    pub tasks: Vec<Task>,
    pub columns: Vec<Column>,
}
