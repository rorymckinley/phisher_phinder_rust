use crate::message_source::MessageSource;
use rusqlite::Connection;

pub fn persist_message_source(
    connection: &Connection,
    message_source: &MessageSource
) -> MessageSource {
    if message_source.id.is_none() {
        crate::persistence::persist_message_source(connection, message_source)
    } else {
        // TODO Ugh - make a better plan
        MessageSource {
            id: message_source.id,
            data: message_source.data.clone()
        }
    }
}

#[cfg(test)]
mod persist_message_source_tests {
    use crate::persistence::{create_message_sources_table, get_record, sha256};
    use super::*;

    #[test]
    fn message_source_has_no_id_no_matching_sha_creates_new_message_source() {
        let connection = Connection::open_in_memory().unwrap();
        let expected_message_source = MessageSource::persisted_record(1, "foo");

        let persisted_source = persist_message_source(&connection, &MessageSource::new("foo"));

        assert_eq!(persisted_source, expected_message_source);
    }

    #[test]
    fn message_has_id_returns_identical_message_source() {
        let connection = Connection::open_in_memory().unwrap();
        create_message_sources_table(&connection);
        let expected_message_source = MessageSource::persisted_record(999, "foo");
        let hash = sha256("foo");

        let persisted_source = persist_message_source(
            &connection,
            &MessageSource::persisted_record(999, "foo")
        );

        assert!(get_record(&connection, &hash).is_none());
        assert_eq!(persisted_source, expected_message_source);
    }
}
