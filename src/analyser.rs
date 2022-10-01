use crate::analysable_message::AnalysableMessage;

pub struct Analyser<'a, T> {
    parsed_mail: &'a T
}

#[cfg(test)]
mod  sender_address_tests {
    use super::*;

    #[test]
    fn return_with_address() {
        let parsed_mail = TestParsedMail::new(
            "return@test.com",
            "reply@test.com",
            "from@test.com"
        );
        let analyser = Analyser::new(&parsed_mail);

        let expected_result = SenderAddresses {
            from: Some("from@test.com".into()),
            reply_to: Some("reply@test.com".into()),
            return_path: Some("return@test.com".into()),
        };

        assert_eq!(expected_result, analyser.sender_email_addresses())
    }

    struct TestParsedMail<'a> {
        reply_to: &'a str,
        return_path: &'a str,
        from: &'a str,
    }

    impl<'a> TestParsedMail<'a> {
        fn new(return_path: &'a str, reply_to: &'a str, from: &'a str) -> Self {
            Self {
                reply_to,
                return_path,
                from,
            }
        }
    }

    impl<'a> AnalysableMessage for TestParsedMail<'a> {
        fn from(&self) -> Option<String> {
            Some(self.from.into())
        }

        fn reply_to(&self) -> Option<String> {
            Some(self.reply_to.into())
        }

        fn return_path(&self) -> Option<String> {
            Some(self.return_path.into())
        }
    }
}

impl<'a, T: AnalysableMessage> Analyser<'a, T> {
    pub fn new(parsed_mail: &'a T) -> Self {
        Self { parsed_mail }
    }

    pub fn sender_email_addresses(&self) -> SenderAddresses {
        SenderAddresses {
            from: self.parsed_mail.from(),
            reply_to: self.parsed_mail.reply_to(),
            return_path: self.parsed_mail.return_path()
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SenderAddresses {
    pub from: Option<String>,
    pub reply_to: Option<String>,
    pub return_path: Option<String>
}
