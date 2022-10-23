use mail_parser::{Addr, HeaderValue, Message};

pub trait AnalysableMessage {
    fn from(&self) -> Option<String>;
    fn reply_to(&self) -> Option<String>;
    fn return_path(&self) -> Option<String>;
}

#[cfg(test)]
mod analysable_message_for_message_tests {
    use super::*;

    #[test]
    fn returns_none_if_not_return_path() {
        let input = "\
            Delivered-To: victim@gmail.com\n\
            Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\n\
                    Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\n\
            From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\n\
            To: victim@gmail.com\n\
            Subject: We’re sorry that we didn’t touch base with you earlier. f309\n\n\
            <div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\n\
            </div>\n
        ";
        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            None,
            parsed_mail.return_path()
        );
    }

    #[test]
    fn returns_the_return_path() {
        let input = "\
            Delivered-To: victim@gmail.com\n\
            Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\n\
                    Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\n\
            Return-Path: <info@xxx.fr>\n\
            From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\n\
            To: victim@gmail.com\n\
            Subject: We’re sorry that we didn’t touch base with you earlier. f309\n\n\
            <div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\n\
            </div>\n\
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            Some("info@xxx.fr".into()),
            parsed_mail.return_path()
        );
    }

    #[test]
    fn returns_none_if_no_reply_to() {
        let input = "\
            Delivered-To: victim@gmail.com\n\
            Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\n\
                    Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\n\
            Return-Path: <info@xxx.fr>\n\
            From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\n\
            To: victim@gmail.com\n\
            Subject: We’re sorry that we didn’t touch base with you earlier. f309\n\n\
            <div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\n\
            </div>\n\
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            None,
            parsed_mail.reply_to()
        )
    }

    #[test]
    fn returns_the_reply_to() {
        let input = "\
            Delivered-To: victim@gmail.com\n\
            Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\n\
                    Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\n\
            Reply-To: scammer@evildomain.zzz\n\
            Return-Path: <info@xxx.fr>\n\
            From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\n\
            To: victim@gmail.com\n\
            Subject: We’re sorry that we didn’t touch base with you earlier. f309\n\n\
            <div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\n\
            </div>\n\
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            Some("scammer@evildomain.zzz".into()),
            parsed_mail.reply_to()
        )
    }

    #[test]
    fn returns_none_if_no_from() {
        let input = "\
            Delivered-To: victim@gmail.com\n\
            Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\n\
                    Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\n\
            To: victim@gmail.com\n\
            Subject: We’re sorry that we didn’t touch base with you earlier. f309\n\n\
            <div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\n\
            </div>\n\
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            None,
            parsed_mail.from()
        );
    }

    #[test]
    fn returns_the_from() {
        let input = "\
            Delivered-To: victim@gmail.com\n\
            Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\n\
                    Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\n\
            Return-Path: <info@xxx.fr>\n\
            From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\n\
            To: victim@gmail.com\n\
            Subject: We’re sorry that we didn’t touch base with you earlier. f309\n\n\
            <div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\n\
            </div>\n\
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            Some("PIBIeSRqUtiEw1NCg4@fake.net".into()),
            parsed_mail.from()
        );
    }
}

impl AnalysableMessage for Message<'_> {
    fn from(&self) -> Option<String> {
        // TODO Cover other options
        match self.get_from() {
            HeaderValue::Address(Addr {name: _, address}) => {
                address.as_deref().map(String::from)
            },
            _ => {
                None
            }
        }
    }

    fn reply_to(&self) -> Option<String> {
        // TODO Cover other options
        match self.get_reply_to() {
            HeaderValue::Address(Addr {name: _, address}) => {
                address.as_deref().map(String::from)
            },
            _ => {
                None
            }
        }
    }

    fn return_path(&self) -> Option<String> {
        // TODO Cover other options
        match self.get_return_path() {
            HeaderValue::Text(address) => {
                Some(address.to_string())
            },
            _ => {
                None
            }
        }
    }
}
