use mail_parser::{Addr, HeaderValue, Message};

pub trait AnalysableMessage {
    fn from(&self) -> Vec<String>;
    fn reply_to(&self) -> Vec<String>;
    fn return_path(&self) -> Vec<String>;
    fn subject(&self) -> Option<String>;
}

#[cfg(test)]
mod analysable_message_for_message_tests {
    use super::*;

    #[test]
    fn returns_empty_collection_if_not_return_path() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
";
        let expected: Vec<String> = vec![];

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(expected, parsed_mail.return_path());
    }

    #[test]
    fn returns_the_return_path() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
Return-Path: <info@xxx.fr>\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
";
        let expected = vec![String::from("info@xxx.fr")];

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(expected, parsed_mail.return_path());
    }

    #[test]
    fn returns_none_if_no_reply_to() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
Return-Path: <info@xxx.fr>\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
        ";

        let expected: Vec<String> = vec![];

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(expected, parsed_mail.reply_to())
    }

    #[test]
    fn returns_the_reply_to() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
Reply-To: scammer@evildomain.zzz\r
Return-Path: <info@xxx.fr>\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(vec![String::from("scammer@evildomain.zzz")], parsed_mail.reply_to())
    }

    #[test]
    fn returns_none_if_no_from() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
To: victim@gmail.com\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
        ";

        let expected: Vec<String> = vec![];
        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(expected, parsed_mail.from());
    }

    #[test]
    fn returns_the_from() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
Return-Path: <info@xxx.fr>\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            vec![String::from("PIBIeSRqUtiEw1NCg4@fake.net")],
            parsed_mail.from()
        );
    }

    #[test]
    fn returns_none_if_no_subject() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
Return-Path: <info@xxx.fr>\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(None, parsed_mail.subject());
    }

    #[test]
    fn returns_subject() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
Return-Path: <info@xxx.fr>\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            Some(String::from("We’re sorry that we didn’t touch base with you earlier. f309")),
            parsed_mail.subject()
        );
    }
}

impl AnalysableMessage for Message<'_> {
    fn from(&self) -> Vec<String> {
        // TODO Cover other options
        match self.get_from() {
            HeaderValue::Address(Addr {name: _, address}) => {
                if let Some(addr) = address.as_deref() {
                    vec![String::from(addr)]
                } else {
                    // TODO can this branch be tested?
                    vec![]
                }
            },
            _ => {
                vec![]
            }
        }
    }

    fn reply_to(&self) -> Vec<String> {
        // TODO Cover other options
        match self.get_reply_to() {
            HeaderValue::Address(Addr {name: _, address}) => {
                if let Some(addr) = address.as_deref() {
                    vec![String::from(addr)]
                } else {
                    // TODO can this branch be tested?
                    vec![]
                }
            },
            _ => {
                vec![]
            }
        }
    }

    fn return_path(&self) -> Vec<String> {
        // TODO Cover other options
        match self.get_return_path() {
            HeaderValue::Text(address) => {
                vec![address.to_string()]
            },
            _ => {
                vec![]
            }
        }
    }

    fn subject(&self) -> Option<String> {
        self.get_subject().map(String::from)
    }
}
