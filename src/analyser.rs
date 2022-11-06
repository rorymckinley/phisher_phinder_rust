use crate::analysable_message::AnalysableMessage;

use serde::Serialize;

pub struct Analyser<'a, T> {
    parsed_mail: &'a T
}

#[cfg(test)]
mod  sender_address_tests {
    use super::*;

    #[test]
    fn test_sender_email_addresses() {
        let parsed = parsed_mail();
        let analyser = Analyser::new(&parsed);

        let expected_result = SenderAddresses {
            from: Some("from@test.com".into()),
            reply_to: Some("reply@test.com".into()),
            return_path: Some("return@test.com".into()),
        };

        assert_eq!(expected_result, analyser.sender_email_addresses())
    }

    #[test]
    fn test_subject() {
        let parsed = parsed_mail();
        let analyser = Analyser::new(&parsed);

        assert_eq!(String::from("My First Phishing Email"), analyser.subject().unwrap());
    }

    fn parsed_mail() -> TestParsedMail {
        TestParsedMail::new(
            "from@test.com".into(),
            "reply@test.com".into(),
            "return@test.com".into(),
            "My First Phishing Email".into(),
        )
    }

    struct TestParsedMail {
        from: String,
        reply_to: String,
        return_path: String,
        subject: String,
    }

    impl TestParsedMail {
        fn new(
            from: String,
            reply_to: String,
            return_path: String,
            subject: String,
        ) -> Self {
            Self {
                from,
                reply_to,
                return_path,
                subject
            }
        }
    }

    impl AnalysableMessage for TestParsedMail {
        fn from(&self) -> Option<String> {
            Some(self.from.clone())
        }

        fn reply_to(&self) -> Option<String> {
            Some(self.reply_to.clone())
        }

        fn return_path(&self) -> Option<String> {
            Some(self.return_path.clone())
        }

        fn subject(&self) -> Option<String> {
            Some(self.subject.clone())
        }
    }
}

impl<'a, T: AnalysableMessage> Analyser<'a, T> {
    pub fn new(parsed_mail: &'a T) -> Self {
        Self { parsed_mail }
    }

    pub fn subject(&self) -> Option<String> {
        self.parsed_mail.subject()
    }

    pub fn sender_email_addresses(&self) -> SenderAddresses {
        SenderAddresses {
            from: self.parsed_mail.from(),
            reply_to: self.parsed_mail.reply_to(),
            return_path: self.parsed_mail.return_path()
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct SenderAddresses {
    pub from: Option<String>,
    pub reply_to: Option<String>,
    pub return_path: Option<String>
}
