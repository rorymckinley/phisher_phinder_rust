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
            MessageSource::new(data)
        );
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MessageSource {
    id: Option<u32>,
    pub data: String,
}

impl MessageSource {
    pub fn new(data: &str) -> Self {
        Self {
            id: None,
            data: data.into(),
        }
    }
}
