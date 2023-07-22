use rusqlite::Connection;
use std::path::Path;

use crate::result::AppResult;

#[cfg(test)]
mod connect_tests {
    use assert_fs::fixture::TempDir;
    use super::*;

    #[test]
    fn returns_connection_to_db_instance() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("db.sqlite3");

        assert!(connect(&db_path).is_ok());
    }

    #[test]
    fn returns_none_if_cannot_create_connection() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("/unobtainium/db.sqlite3");

        assert!(connect(&db_path).is_err())
    }
}

pub fn connect(path: &Path) -> AppResult<Connection> {
    Ok(Connection::open(path)?)
}

#[cfg(test)]
mod persist_message_source_tests {
    use super::*;

    #[test]
    fn creates_the_messages_sources_table() {
        let conn = connection();

        persist_message_source(&conn, &message_source_1());

        assert!(table_exists(&conn))
    }

    #[test]
    fn inserts_records_into_the_table() {
        let conn = connection();

        persist_message_source(&conn, &message_source_1());
        persist_message_source(&conn, &message_source_2());

        assert_eq!(
            vec![(1, message_source_1()), (2, message_source_2())],
            table_contents(&conn)
        );
    }

    fn connection() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn message_source_1() -> String {
        "Message Source 1".into()
    }

    fn message_source_2() -> String {
        "Message Source 2".into()
    }

    fn table_exists(conn: &Connection) -> bool {
        conn.prepare("SELECT * FROM message_sources").is_ok()
    }

    fn table_contents(conn: &Connection) -> Vec<(u32, String)> {
        let mut stmt = conn.prepare("SELECT id, contents FROM message_sources").unwrap();
        let rows = stmt.query_map(
            [],
            |row| Ok((row.get::<usize, u32>(0).unwrap(), row.get::<usize, String>(1).unwrap()))
        ).unwrap();

        rows.flatten().collect()
    }
}

pub fn persist_message_source(conn: &Connection, source: &str) {
    // TODO Think about ways the below can actually fail and then replace the `.unwrap()` calls
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_sources (id INTEGER PRIMARY KEY, contents TEXT NOT NULL)",
        []
    ).unwrap();

    conn.execute("INSERT INTO message_sources (contents) VALUES (?1)", (source,)).unwrap();
}
