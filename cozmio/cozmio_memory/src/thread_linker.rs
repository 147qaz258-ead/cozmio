use crate::db::Database;
use crate::error::MemoryError;

/// Imported events are not mechanically assigned to semantic task threads.
///
/// Thread membership is a semantic interpretation. This crate may store factual
/// event rows and mechanical ids, but it must not infer a project, workflow, or
/// task thread from window-title/content keywords. A model or execution agent can
/// later write thread links with provenance.
pub fn link_events_to_threads(_db: &Database) -> Result<usize, MemoryError> {
    Ok(0)
}

/// Re-running the linker keeps the same boundary: no automatic semantic
/// reclassification is performed in system code.
pub fn relink_all_events(_db: &Database) -> Result<usize, MemoryError> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::run_migrations;
    use tempfile::tempdir;

    #[test]
    fn thread_linker_does_not_assign_semantic_threads() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_thread_linker.db");
        let db = Database::new(&db_path).unwrap();
        run_migrations(&db).unwrap();

        assert_eq!(link_events_to_threads(&db).unwrap(), 0);
        assert_eq!(relink_all_events(&db).unwrap(), 0);
    }
}
