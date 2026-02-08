//! Database module for tickit-sync server

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

use crate::models::{List, Priority, RecordType, SyncRecord, Tag, Task, TaskTagLink};

/// Thread-safe database wrapper
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open or create the database
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let conn = Connection::open(path).context("Failed to open database")?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init()?;

        Ok(db)
    }

    /// Initialize the database schema
    fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            -- Lists table
            CREATE TABLE IF NOT EXISTS lists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                icon TEXT NOT NULL DEFAULT '📋',
                color TEXT,
                is_inbox INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Tags table  
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT
            );

            -- Tasks table
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                url TEXT,
                priority TEXT NOT NULL DEFAULT 'medium',
                completed INTEGER NOT NULL DEFAULT 0,
                list_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT,
                due_date TEXT,
                FOREIGN KEY (list_id) REFERENCES lists(id)
            );

            -- Task-Tag junction table
            CREATE TABLE IF NOT EXISTS task_tags (
                task_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (task_id, tag_id),
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            -- Tombstones for deleted records
            CREATE TABLE IF NOT EXISTS tombstones (
                id TEXT PRIMARY KEY,
                record_type TEXT NOT NULL,
                deleted_at TEXT NOT NULL
            );

            -- Device sync state
            CREATE TABLE IF NOT EXISTS device_sync (
                device_id TEXT PRIMARY KEY,
                last_sync TEXT NOT NULL
            );

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks(list_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_updated ON tasks(updated_at);
            CREATE INDEX IF NOT EXISTS idx_lists_updated ON lists(updated_at);
            CREATE INDEX IF NOT EXISTS idx_tombstones_deleted ON tombstones(deleted_at);
            "#,
        )?;

        Ok(())
    }

    /// Get all changes since a given timestamp
    pub fn get_changes_since(&self, since: Option<&str>) -> Result<Vec<SyncRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut changes = Vec::new();

        // Get lists
        let lists = if let Some(since) = since {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, icon, color, is_inbox, sort_order, created_at, updated_at 
                 FROM lists WHERE updated_at > ?1"
            )?;
            self.collect_lists(&mut stmt, params![since])?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, icon, color, is_inbox, sort_order, created_at, updated_at FROM lists"
            )?;
            self.collect_lists(&mut stmt, [])?
        };

        for list in lists {
            changes.push(SyncRecord::List(list));
        }

        // Get tags
        let tags = if let Some(since) = since {
            let mut stmt = conn.prepare(
                "SELECT id, name, color, created_at, updated_at FROM tags WHERE created_at > ?1",
            )?;
            self.collect_tags(&mut stmt, params![since])?
        } else {
            let mut stmt =
                conn.prepare("SELECT id, name, color, created_at, updated_at FROM tags")?;
            self.collect_tags(&mut stmt, [])?
        };

        for tag in tags {
            changes.push(SyncRecord::Tag(tag));
        }

        // Get tasks
        let tasks = if let Some(since) = since {
            let mut stmt = conn.prepare(
                "SELECT id, title, description, url, priority, completed, list_id, 
                 created_at, updated_at, completed_at, due_date FROM tasks WHERE updated_at > ?1",
            )?;
            self.collect_tasks(&conn, &mut stmt, params![since])?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, title, description, url, priority, completed, list_id, 
                 created_at, updated_at, completed_at, due_date FROM tasks",
            )?;
            self.collect_tasks(&conn, &mut stmt, [])?
        };

        for task in tasks {
            changes.push(SyncRecord::Task(task));
        }

        // Get tombstones
        let tombstones = if let Some(since) = since {
            let mut stmt = conn.prepare(
                "SELECT id, record_type, deleted_at FROM tombstones WHERE deleted_at > ?1",
            )?;
            self.collect_tombstones(&mut stmt, params![since])?
        } else {
            let mut stmt = conn.prepare("SELECT id, record_type, deleted_at FROM tombstones")?;
            self.collect_tombstones(&mut stmt, [])?
        };

        for (id, record_type, deleted_at) in tombstones {
            changes.push(SyncRecord::Deleted {
                id,
                record_type,
                deleted_at,
            });
        }

        Ok(changes)
    }

    fn collect_lists<P: rusqlite::Params>(
        &self,
        stmt: &mut rusqlite::Statement,
        params: P,
    ) -> Result<Vec<List>> {
        let rows = stmt.query_map(params, |row| {
            Ok(List {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                is_inbox: row.get::<_, i32>(5)? != 0,
                sort_order: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn collect_tags<P: rusqlite::Params>(
        &self,
        stmt: &mut rusqlite::Statement,
        params: P,
    ) -> Result<Vec<Tag>> {
        let rows = stmt.query_map(params, |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn collect_tasks<P: rusqlite::Params>(
        &self,
        conn: &Connection,
        stmt: &mut rusqlite::Statement,
        params: P,
    ) -> Result<Vec<Task>> {
        let rows = stmt.query_map(params, |row| {
            let priority_str: String = row.get(4)?;
            let priority = match priority_str.as_str() {
                "low" => Priority::Low,
                "high" => Priority::High,
                "urgent" => Priority::Urgent,
                _ => Priority::Medium,
            };

            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                url: row.get(3)?,
                priority,
                completed: row.get::<_, i32>(5)? != 0,
                list_id: row.get(6)?,
                tag_ids: Vec::new(), // Filled below
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                completed_at: row.get(9)?,
                due_date: row.get(10)?,
            })
        })?;

        let mut tasks: Vec<Task> = rows.collect::<Result<Vec<_>, _>>()?;

        // Fill in tag_ids
        for task in &mut tasks {
            let mut tag_stmt = conn.prepare("SELECT tag_id FROM task_tags WHERE task_id = ?1")?;
            let tag_ids: Vec<String> = tag_stmt
                .query_map(params![&task.id], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;
            task.tag_ids = tag_ids;
        }

        Ok(tasks)
    }

    fn collect_tombstones<P: rusqlite::Params>(
        &self,
        stmt: &mut rusqlite::Statement,
        params: P,
    ) -> Result<Vec<(String, RecordType, String)>> {
        let rows = stmt.query_map(params, |row| {
            let record_type_str: String = row.get(1)?;
            let record_type = match record_type_str.as_str() {
                "task" => RecordType::Task,
                "list" => RecordType::List,
                "tag" => RecordType::Tag,
                "task_tag" => RecordType::TaskTag,
                _ => RecordType::Task,
            };

            Ok((row.get(0)?, record_type, row.get(2)?))
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Apply incoming changes from a client
    pub fn apply_changes(&self, changes: &[SyncRecord]) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut conflicts = Vec::new();

        // Disable foreign key checks during sync to avoid ordering issues
        conn.execute("PRAGMA foreign_keys = OFF", [])?;

        // Sort changes: lists first, then tags, then tasks, then deletions last
        // This ensures foreign key constraints are satisfied
        let mut sorted_changes: Vec<_> = changes.iter().collect();
        sorted_changes.sort_by_key(|change| match change {
            SyncRecord::List(_) => 0,
            SyncRecord::Tag(_) => 1,
            SyncRecord::TaskTag(_) => 2,
            SyncRecord::Task(_) => 3,
            SyncRecord::Deleted { .. } => 4,
        });

        for change in sorted_changes {
            match change {
                SyncRecord::Task(task) => {
                    if let Some(conflict) = self.upsert_task(&conn, task)? {
                        conflicts.push(conflict);
                    }
                }
                SyncRecord::List(list) => {
                    if let Some(conflict) = self.upsert_list(&conn, list)? {
                        conflicts.push(conflict);
                    }
                }
                SyncRecord::Tag(tag) => {
                    self.upsert_tag(&conn, tag)?;
                }
                SyncRecord::TaskTag(link) => {
                    self.upsert_task_tag(&conn, link)?;
                }
                SyncRecord::Deleted {
                    id,
                    record_type,
                    deleted_at,
                } => {
                    self.apply_delete(&conn, id, *record_type, deleted_at)?;
                }
            }
        }

        // Re-enable foreign key checks
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        Ok(conflicts)
    }

    fn upsert_task(&self, conn: &Connection, task: &Task) -> Result<Option<String>> {
        // Check existing
        let existing: Option<String> = conn
            .query_row(
                "SELECT updated_at FROM tasks WHERE id = ?1",
                params![&task.id],
                |row| row.get(0),
            )
            .ok();

        if let Some(existing_updated) = existing {
            if task.updated_at <= existing_updated {
                // Conflict: server has newer
                return Ok(Some(task.id.clone()));
            }

            // Update existing
            conn.execute(
                r#"UPDATE tasks SET title = ?2, description = ?3, url = ?4, priority = ?5,
                   completed = ?6, list_id = ?7, updated_at = ?8, completed_at = ?9, due_date = ?10
                   WHERE id = ?1"#,
                params![
                    &task.id,
                    &task.title,
                    &task.description,
                    &task.url,
                    format!("{:?}", task.priority).to_lowercase(),
                    task.completed as i32,
                    &task.list_id,
                    &task.updated_at,
                    &task.completed_at,
                    &task.due_date,
                ],
            )?;
        } else {
            // Insert new
            conn.execute(
                r#"INSERT INTO tasks (id, title, description, url, priority, completed, list_id,
                   created_at, updated_at, completed_at, due_date)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
                params![
                    &task.id,
                    &task.title,
                    &task.description,
                    &task.url,
                    format!("{:?}", task.priority).to_lowercase(),
                    task.completed as i32,
                    &task.list_id,
                    &task.created_at,
                    &task.updated_at,
                    &task.completed_at,
                    &task.due_date,
                ],
            )?;
        }

        // Update tags
        conn.execute(
            "DELETE FROM task_tags WHERE task_id = ?1",
            params![&task.id],
        )?;

        let now = Utc::now().to_rfc3339();
        for tag_id in &task.tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
                params![&task.id, tag_id, &now],
            )?;
        }

        Ok(None)
    }

    fn upsert_list(&self, conn: &Connection, list: &List) -> Result<Option<String>> {
        let existing: Option<String> = conn
            .query_row(
                "SELECT updated_at FROM lists WHERE id = ?1",
                params![&list.id],
                |row| row.get(0),
            )
            .ok();

        if let Some(existing_updated) = existing {
            if list.updated_at <= existing_updated {
                return Ok(Some(list.id.clone()));
            }

            conn.execute(
                r#"UPDATE lists SET name = ?2, description = ?3, icon = ?4, color = ?5,
                   sort_order = ?6, updated_at = ?7 WHERE id = ?1"#,
                params![
                    &list.id,
                    &list.name,
                    &list.description,
                    &list.icon,
                    &list.color,
                    list.sort_order,
                    &list.updated_at,
                ],
            )?;
        } else {
            conn.execute(
                r#"INSERT INTO lists (id, name, description, icon, color, is_inbox, sort_order, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
                params![
                    &list.id,
                    &list.name,
                    &list.description,
                    &list.icon,
                    &list.color,
                    list.is_inbox as i32,
                    list.sort_order,
                    &list.created_at,
                    &list.updated_at,
                ],
            )?;
        }

        Ok(None)
    }

    fn upsert_tag(&self, conn: &Connection, tag: &Tag) -> Result<()> {
        conn.execute(
            r#"INSERT OR REPLACE INTO tags (id, name, color, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
            params![
                &tag.id,
                &tag.name,
                &tag.color,
                &tag.created_at,
                &tag.updated_at
            ],
        )?;
        Ok(())
    }

    fn upsert_task_tag(&self, conn: &Connection, link: &TaskTagLink) -> Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO task_tags (task_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
            params![&link.task_id, &link.tag_id, &link.created_at],
        )?;
        Ok(())
    }

    fn apply_delete(
        &self,
        conn: &Connection,
        id: &str,
        record_type: RecordType,
        deleted_at: &str,
    ) -> Result<()> {
        let type_str = match record_type {
            RecordType::Task => "task",
            RecordType::List => "list",
            RecordType::Tag => "tag",
            RecordType::TaskTag => "task_tag",
        };

        // Record tombstone
        conn.execute(
            "INSERT OR REPLACE INTO tombstones (id, record_type, deleted_at) VALUES (?1, ?2, ?3)",
            params![id, type_str, deleted_at],
        )?;

        // Delete the actual record
        match record_type {
            RecordType::Task => {
                conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
            }
            RecordType::List => {
                // Don't delete inbox
                conn.execute(
                    "DELETE FROM lists WHERE id = ?1 AND is_inbox = 0",
                    params![id],
                )?;
            }
            RecordType::Tag => {
                conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
            }
            RecordType::TaskTag => {
                // id is task_id for task_tag tombstones
                conn.execute("DELETE FROM task_tags WHERE task_id = ?1", params![id])?;
            }
        }

        Ok(())
    }

    /// Update device sync timestamp
    pub fn update_device_sync(&self, device_id: &str, timestamp: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO device_sync (device_id, last_sync) VALUES (?1, ?2)",
            params![device_id, timestamp],
        )?;
        Ok(())
    }
}
