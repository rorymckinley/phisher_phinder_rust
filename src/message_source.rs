#[cfg(test)]
mod message_source_new_tests {
    use super::*;

    #[test]
    fn returns_instance() {
        let data = "Delivered-To: blah";

        assert_eq!(
            MessageSource {
                id: None,
                data: data.into()
            },
            MessageSource::new(&data)
        );
    }
}

#[cfg(test)]
mod message_source_persisted_record_tests {
    use super::*;

    #[test]
    fn returns_instance() {
        let id = 1991;
        let data = "Delivered-To: blah";

        assert_eq!(
            MessageSource {
                id: Some(id),
                data: data.into()
            },
            MessageSource::persisted_record(id, data)
        );
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MessageSource {
    pub id: Option<u32>,
    pub data: String,
}

impl MessageSource {
    pub fn new(data: &str) -> Self {
        Self {
            id: None,
            data: data.into(),
        }
    }

    pub fn persisted_record(id: u32, data: &str) -> Self {
        Self {
            id: Some(id),
            data: data.into(),
        }
    }
}
