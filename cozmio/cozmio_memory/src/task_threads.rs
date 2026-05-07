use crate::db::Database;
use crate::error::MemoryError;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct TaskThread {
    pub id: Option<i64>,
    pub name: String,
    pub current_state: Option<String>,
    pub open_questions: Vec<String>,
    pub decisions: Vec<String>,
    pub recent_slice_ids: Vec<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskThreadUpdate {
    pub name: String,
    pub current_state: Option<String>,
    pub open_questions: Option<Vec<String>>,
    pub decisions: Option<Vec<String>>,
    pub recent_slice_ids: Option<Vec<i64>>,
}

pub struct TaskThreadsStore<'a>(&'a Database);

impl<'a> TaskThreadsStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    pub fn get_all(&self) -> Result<Vec<TaskThread>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, current_state, open_questions, decisions, recent_slice_ids, created_at, updated_at
             FROM task_threads ORDER BY id ASC"
        )?;
        let rows = stmt.query_map([], |row| self.row_to_thread(row))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<TaskThread>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, current_state, open_questions, decisions, recent_slice_ids, created_at, updated_at
             FROM task_threads WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_thread(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_name(&self, name: &str) -> Result<Option<TaskThread>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, current_state, open_questions, decisions, recent_slice_ids, created_at, updated_at
             FROM task_threads WHERE name = ?1"
        )?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_thread(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn upsert(&self, thread: &TaskThreadUpdate) -> Result<TaskThread, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO task_threads (name, current_state, open_questions, decisions, recent_slice_ids)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(name) DO UPDATE SET
               current_state = excluded.current_state,
               open_questions = excluded.open_questions,
               decisions = excluded.decisions,
               recent_slice_ids = excluded.recent_slice_ids,
               updated_at = datetime('now')",
            params![
                thread.name,
                thread.current_state,
                serde_json::to_string(thread.open_questions.as_ref().unwrap_or(&vec![])).unwrap_or_default(),
                serde_json::to_string(thread.decisions.as_ref().unwrap_or(&vec![])).unwrap_or_default(),
                serde_json::to_string(thread.recent_slice_ids.as_ref().unwrap_or(&vec![])).unwrap_or_default(),
            ],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.get_by_id(id).map(|opt| opt.expect("just inserted"))
    }

    fn row_to_thread(&self, row: &rusqlite::Row) -> Result<TaskThread, rusqlite::Error> {
        let open_q_str: String = row.get(3)?;
        let decisions_str: String = row.get(4)?;
        let recent_str: String = row.get(5)?;
        Ok(TaskThread {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            current_state: row.get(2)?,
            open_questions: serde_json::from_str(&open_q_str).unwrap_or_default(),
            decisions: serde_json::from_str(&decisions_str).unwrap_or_default(),
            recent_slice_ids: serde_json::from_str(&recent_str).unwrap_or_default(),
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}
