use chrono::prelude::*;
use crate::data::OutputData;
use crate::message_source::MessageSource;

#[derive(Debug, PartialEq)]
pub struct Run {
    pub created_at: DateTime<Utc>,
    pub data: OutputData,
    pub id: u32,
    pub message_source: MessageSource
}

impl Run {
    pub fn persisted_record(
        id: u32,
        data_string: String,
        created_at: String,
        message_source_id: u32,
        message_source_string: String
    ) -> Self {

        Self {
            id,
            data: serde_json::from_str(&data_string).unwrap(),
            created_at: DateTime::parse_from_str(
                &format!("{} +0000", created_at),
                "%Y-%m-%d %H:%M:%S %z"
            ).unwrap().into(),
            message_source: MessageSource::persisted_record(
                message_source_id,
                &message_source_string
            )
        }
    }
}

#[cfg(test)]
mod persisted_record_tests {
    use serde_json::json;
    use super::*;

    #[test]
    fn it_builds_a_run_from_a_persisted_record() {
        let run = Run::persisted_record(
            999,
            data_string(),
            String::from("2023-08-29 09:41:00"),
            1001,
            String::from("xx")
        );

        assert_eq!(run.id, 999);
        assert_eq!(run.data, expected_output_data(data_string()));
        assert_eq!(run.message_source, expected_message_source());
        assert_eq!(run.created_at, expected_created_at());
    }

    fn data_string() -> String {
        json!({
            "parsed_mail": {
                "authentication_results": null,
                "delivery_nodes": [],
                "email_addresses": {
                    "from": [],
                    "links": [],
                    "reply_to": [],
                    "return_path": [],
                },
                "fulfillment_nodes": [],
                "subject": "Dodgy Subject",
            },
            "message_source": {
                "data": "xx",
                "id": 1001,
            },
            "notifications": [],
            "reportable_entities": null,
            "run_id": null,
        }).to_string()
    }

    fn expected_output_data(serialized_data: String) -> OutputData {
        serde_json::from_str(&serialized_data).unwrap()
    }

    fn expected_message_source() -> MessageSource {
        MessageSource::persisted_record(1001, "xx")
    }

    fn expected_created_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, 0).unwrap()
    }
}
