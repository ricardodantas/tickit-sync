//! Sync data models (shared types between client and server)
//!
//! Uses String for IDs and timestamps for maximum compatibility with clients.

use serde::{Deserialize, Serialize};

/// Priority level for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Urgent,
}

/// A task/todo item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    pub priority: Priority,
    pub completed: bool,
    pub list_id: String,
    #[serde(default)]
    pub tag_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
}

/// A list/project that contains tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub is_inbox: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_icon() -> String {
    "📁".to_string()
}

/// A tag that can be attached to tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Link between task and tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTagLink {
    pub task_id: String,
    pub tag_id: String,
    pub created_at: String,
}

/// Type of record (for tombstones)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Task,
    List,
    Tag,
    TaskTag,
}

/// A record that can be synced
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncRecord {
    Task(Task),
    List(List),
    Tag(Tag),
    TaskTag(TaskTagLink),
    Deleted {
        id: String,
        record_type: RecordType,
        deleted_at: String,
    },
}

/// Request to sync changes with server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Device identifier
    pub device_id: String,
    /// Timestamp of last successful sync (None = full sync)
    pub last_sync: Option<String>,
    /// Changes from this client since last sync
    pub changes: Vec<SyncRecord>,
}

/// Response from sync server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Server timestamp for this sync
    pub server_time: String,
    /// Changes from other devices to apply locally
    pub changes: Vec<SyncRecord>,
    /// IDs of records that had conflicts (server won)
    pub conflicts: Vec<String>,
}
