use crate::db::Database;
use crate::error::MemoryError;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct Skill {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub procedure: String,
    pub success_context: Option<String>,
    pub usage_count: i64,
    pub last_used_at: Option<String>,
    pub evidence_source: String,
}

pub struct SkillMemoryStore<'a>(&'a Database);

impl<'a> SkillMemoryStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    pub fn insert(&self, skill: &Skill) -> Result<i64, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO skill_memory (name, description, procedure, success_context, usage_count, last_used_at, evidence_source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                skill.name,
                skill.description,
                skill.procedure,
                skill.success_context,
                skill.usage_count,
                skill.last_used_at,
                skill.evidence_source,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all(&self) -> Result<Vec<Skill>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, procedure, success_context, usage_count, last_used_at, evidence_source
             FROM skill_memory ORDER BY usage_count DESC, id ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Skill {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                procedure: row.get(3)?,
                success_context: row.get(4)?,
                usage_count: row.get(5)?,
                last_used_at: row.get(6)?,
                evidence_source: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn increment_usage(&self, id: i64) -> Result<(), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "UPDATE skill_memory SET usage_count = usage_count + 1, last_used_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
}
