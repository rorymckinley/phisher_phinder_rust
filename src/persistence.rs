use crate::data::OutputData;
use crate::errors::AppError;
use crate::message_source::MessageSource;
use crate::result::AppResult;
use chrono::Utc;
use rusqlite::{Connection, Statement};
use sha2::{Digest, Sha256};
use std::path::Path;

#[cfg(test)]
mod connect_tests {
    use super::*;
    use assert_fs::fixture::TempDir;

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
    use chrono::prelude::*;
    use chrono::{Duration, Utc};

    #[test]
    fn creates_the_messages_sources_table() {
        let conn = connection();

        persist_message_source(&conn, message_source_1());

        assert!(table_exists(&conn))
    }

    #[test]
    fn inserts_records_into_the_table() {
        let conn = connection();

        persist_message_source(&conn, message_source_1());
        persist_message_source(&conn, message_source_2());

        assert_eq!(
            vec![
                (1, message_source_data_1(), message_1_hash()),
                (2, message_source_data_2(), message_2_hash())
            ],
            everything_except_created_at(table_contents(&conn))
        );
    }

    #[test]
    fn sets_created_at_to_time_inserted() {
        let conn = connection();
        let now = Utc::now();

        persist_message_source(&conn, message_source_1());

        let (_, _, _, created_at_string) = table_contents(&conn).pop().unwrap();

        let created_at = Utc
            .datetime_from_str(&created_at_string, "%Y-%m-%d %H:%M:%S")
            .unwrap();

        assert!(created_at.signed_duration_since(now) <= Duration::seconds(1));
    }

    #[test]
    fn returns_message_source_with_persisted_id() {
        let conn = connection();

        let message_source = persist_message_source(&conn, message_source_1());

        assert_eq!(expected_message_source(), message_source);
    }

    #[test]
    fn does_not_store_duplicate_messages() {
        let conn = connection();

        persist_message_source(&conn, message_source_1());
        persist_message_source(&conn, message_source_1());

        assert_eq!(
            vec![(1, message_source_data_1(), message_1_hash()),],
            everything_except_created_at(table_contents(&conn))
        );
    }

    #[test]
    fn returns_message_source_if_record_already_exists() {
        let conn = connection();

        persist_message_source(&conn, message_source_1());
        let message_source = persist_message_source(&conn, message_source_1());

        assert_eq!(expected_message_source(), message_source);
    }

    fn connection() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn message_source_data_1() -> String {
        "Message Source 1".into()
    }

    fn message_source_1() -> MessageSource {
        MessageSource::new(&message_source_data_1())
    }

    fn expected_message_source() -> MessageSource {
        MessageSource {
            id: Some(1),
            data: message_source_data_1(),
        }
    }

    fn message_1_hash() -> String {
        "41bea4496bda7a9eab66ca2f5e5a094992eaa4a98a81191d198ebdb115eed5f5".into()
    }

    fn message_source_data_2() -> String {
        "Message Source 2".into()
    }

    fn message_source_2() -> MessageSource {
        MessageSource::new(&message_source_data_2())
    }

    fn message_2_hash() -> String {
        "e473a80a3a2767edfe6c2800139a68ab8a47c5139909bb59b61a438a8c12fb73".into()
    }

    fn table_exists(conn: &Connection) -> bool {
        conn.prepare("SELECT * FROM message_sources").is_ok()
    }

    fn table_contents(conn: &Connection) -> Vec<(u32, String, String, String)> {
        let mut stmt = conn
            .prepare("SELECT id, data, hash, created_at FROM message_sources")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<usize, u32>(0).unwrap(),
                    row.get::<usize, String>(1).unwrap(),
                    row.get::<usize, String>(2).unwrap(),
                    row.get::<usize, String>(3).unwrap(),
                ))
            })
            .unwrap();

        rows.flatten().collect()
    }

    fn everything_except_created_at(
        records: Vec<(u32, String, String, String)>,
    ) -> Vec<(u32, String, String)> {
        records
            .into_iter()
            .map(|(id, data, hash, _created_at)| (id, data, hash))
            .collect()
    }
}

pub fn persist_message_source(conn: &Connection, source: MessageSource) -> MessageSource {
    create_message_sources_table(conn);

    let hash = sha256(&source.data);

    if let Some(message_source) = get_record(conn, &hash) {
        message_source
    } else {
        conn.execute(
            "INSERT INTO message_sources (data, hash, created_at) VALUES (?1, ?2, ?3)",
            (&source.data, &hash, created_at_string()),
        )
        .unwrap();

        get_record(conn, &hash).unwrap()
    }
}

fn create_message_sources_table(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_sources \
        ( \
            id INTEGER PRIMARY KEY, \
            data TEXT NOT NULL, \
            hash TEXT NOT NULL, \
            created_at TEXT NOT NULL
        )",
        [],
    )
    .unwrap();
}

fn sha256(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text);
    let sha = hasher.finalize();

    sha.iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join("")
}

fn created_at_string() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn get_record(conn: &Connection, hash: &str) -> Option<MessageSource> {
    let mut stmt = get_record_by_hash_statement(conn);

    stmt.query_row([hash], |row| {
        let id = row.get::<usize, u32>(0).unwrap();
        let data = row.get::<usize, String>(1).unwrap();

        Ok(MessageSource::persisted_record(id, &data))
    })
    .ok()
}

fn get_record_by_hash_statement(conn: &Connection) -> Statement {
    conn.prepare("SELECT id, data FROM message_sources WHERE hash = ?")
        .unwrap()
}

#[cfg(test)]
mod persist_run_tests {
    use super::*;
    use crate::data::{EmailAddresses, ParsedMail};
    use chrono::prelude::*;
    use chrono::{Duration, Utc};

    #[test]
    fn creates_the_runs_table() {
        let conn = connection();

        create_message_sources_table(&conn);
        let message_source_id_1 = create_message_source_entry(&conn, 100, "x");

        persist_run(&conn, &input(message_source_id_1)).unwrap();

        assert!(table_exists(&conn));
    }

    #[test]
    fn creates_run_entries() {
        let conn = connection();

        create_message_sources_table(&conn);

        let _other_message_source_id_1 = create_message_source_entry(&conn, 100, "w");
        let message_source_id_1 = create_message_source_entry(&conn, 101, "x");
        let message_source_id_2 = create_message_source_entry(&conn, 102, "y");
        let _other_message_source_id_2 = create_message_source_entry(&conn, 103, "z");

        persist_run(&conn, &input(message_source_id_1)).unwrap();
        persist_run(&conn, &input(message_source_id_2)).unwrap();

        assert_eq!(
            vec![
                (1, serialised_input(&input(message_source_id_1)), 101),
                (2, serialised_input(&input(message_source_id_2)), 102),
            ],
            everything_except_created_at(table_contents(&conn))
        );
    }

    #[test]
    fn sets_created_at_time_inserted() {
        let conn = connection();
        let now = Utc::now();

        create_message_sources_table(&conn);
        let message_source_id_1 = create_message_source_entry(&conn, 100, "x");

        persist_run(&conn, &input(message_source_id_1)).unwrap();

        let (_, _, _, created_at_string) = table_contents(&conn).pop().unwrap();

        let created_at = Utc
            .datetime_from_str(&created_at_string, "%Y-%m-%d %H:%M:%S")
            .unwrap();

        assert!(created_at.signed_duration_since(now) <= Duration::seconds(1));
    }

    #[test]
    fn returns_run_id() {
        let conn = connection();

        create_message_sources_table(&conn);
        let message_source_id_1 = create_message_source_entry(&conn, 100, "x");

        assert_eq!(1, persist_run(&conn, &input(message_source_id_1)).unwrap())
    }

    #[test]
    fn returns_an_error_if_message_source_id_does_not_map() {
        let conn = connection();

        create_message_sources_table(&conn);

        match persist_run(&conn, &input(9999)) {
            Err(AppError::Persistence(message)) => {
                assert!(message.contains("creating run"));
            }
            Err(e) => {
                panic!("Unexpected error #{e:?}");
            }
            Ok(_) => {
                panic!("No error!")
            }
        }
    }

    fn connection() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn input(message_source_id: u32) -> OutputData {
        OutputData {
            message_source: MessageSource {
                id: Some(message_source_id),
                data: "x".into(),
            },
            parsed_mail: ParsedMail::new(
                None,
                vec![],
                EmailAddresses {
                    from: vec![],
                    links: vec![],
                    reply_to: vec![],
                    return_path: vec![],
                },
                vec![],
                Some("dodgy subject".into()),
            ),
            reportable_entities: None,
            run_id: None,
        }
    }

    fn serialised_input(output_data: &OutputData) -> String {
        serde_json::to_string(output_data).unwrap()
    }

    fn table_exists(conn: &Connection) -> bool {
        conn.prepare("SELECT * FROM runs").is_ok()
    }

    fn create_message_source_entry(conn: &Connection, id: u32, data: &str) -> u32 {
        let hash = sha256(data);

        conn.execute(
            "INSERT INTO message_sources (id, data, hash, created_at) VALUES (?1, ?2, ?3, ?4)",
            (id, data, &hash, "2023-07-30 10:00:00"),
        )
        .unwrap();

        get_record(conn, &hash).unwrap().id.unwrap()
    }

    fn table_contents(conn: &Connection) -> Vec<(u32, String, u32, String)> {
        let mut stmt = conn
            .prepare("SELECT id, data, message_source_id, created_at FROM runs")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<usize, u32>(0).unwrap(),
                    row.get::<usize, String>(1).unwrap(),
                    row.get::<usize, u32>(2).unwrap(),
                    row.get::<usize, String>(3).unwrap(),
                ))
            })
            .unwrap();

        rows.flatten().collect()
    }

    fn everything_except_created_at(
        records: Vec<(u32, String, u32, String)>,
    ) -> Vec<(u32, String, u32)> {
        records
            .into_iter()
            .map(|(id, data, message_source_id, _created_at)| (id, data, message_source_id))
            .collect()
    }
}

pub fn persist_run(conn: &Connection, run_data: &OutputData) -> AppResult<u32> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS runs \
        ( \
            id INTEGER PRIMARY KEY, \
            data TEXT NOT NULL, \
            message_source_id INTEGER NOT NULL, \
            created_at TEXT NOT NULL, \
            FOREIGN KEY(message_source_id) REFERENCES message_sources(id)
        )",
        [],
    )
    .unwrap();

    let execute_result = conn.execute(
        "INSERT INTO runs (data, message_source_id, created_at) VALUES (?1, ?2, ?3)",
        (
            serde_json::to_string(run_data).unwrap(),
            run_data.message_source.id.unwrap(),
            created_at_string(),
        ),
    );

    match execute_result {
        Ok(_) => {
            // TODO This will break if there is another process writing to the DB
            let mut stmt = conn.prepare("SELECT last_insert_rowid()").unwrap();

            let last_id = stmt.query_row([], |row| row.get::<usize, u32>(0)).unwrap();

            Ok(last_id)
        }
        Err(rusqlite::Error::SqliteFailure(_, message_option)) => {
            let error_text = match message_option {
                Some(message) => format!("Error creating run - SQLite returned: {message}"),
                None => "Error creating run - SQLite did not return a message".into(),
            };
            Err(AppError::Persistence(error_text))
        }
        _ => {
            // TODO Not tested - find a way to trigger this
            Err(AppError::Persistence(String::from("foo")))
        }
    }
}
