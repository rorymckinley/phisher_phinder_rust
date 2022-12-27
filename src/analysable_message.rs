use mail_parser::{Addr, HeaderValue, Message};
use scraper::{Html, Selector};

pub trait AnalysableMessage {
    fn links(&self) -> Vec<String>;
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
    fn returns_multiple_reply_to_addresses() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
Reply-To: scammer@evil.zzz;scammer@scam.zzz, scammer@dodgy.zzz;scammer@fraud.zzz\r
Return-Path: <info@xxx.fr>\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
</div>\r
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        let expected = vec![
            String::from("scammer@evil.zzz"),
            String::from("scammer@scam.zzz"),
            String::from("scammer@dodgy.zzz"),
            String::from("scammer@fraud.zzz"),
        ];

        assert_eq!(expected, parsed_mail.reply_to())
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

    #[test]
    fn returns_links_from_all_html_body_parts() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
Return-Path: <info@xxx.fr>\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@fake.net>\r
To: victim@gmail.com\r
Content-Type: multipart/mixed; boundary=\"bnd_123\"\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r\n\r
\r
--bnd_123
Content-Type: text/plain; charset=\"utf-8\"\r
\r
<html>\r
    <body>\r
        <a href=\"https://foo.bogus\" class=\"bar\">Link 0</a>\r
    </body>\r
</html>\r
\r
--bnd_123\r
Content-Type: text/html; charset=\"utf-8\"\r
\r
<html>\r
    <body>\r
        <a href=\"https://foo.bar\" class=\"bar\">Link 1</a>\r
        <a href=\"https://foo.baz\" class=\"baz\">Link 2</a>\r
    </body>\r
</html>\r
\r
--bnd_123\r
Content-Type: text/html; charset=\"utf-8\"\r
\r
<html>\r
    <body>\r
        <a href=\"https://foo.biz\" class=\"biz\">Link 3</a>\r
    </body>\r
</html>\r
--bnd_123\r
        ";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        assert_eq!(
            vec![
                String::from("https://foo.bar"),
                String::from("https://foo.baz"),
                String::from("https://foo.biz"),
            ],
            parsed_mail.links()
        )
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
            HeaderValue::GroupList(groups) => {
                groups
                    .iter()
                    .fold(vec![], |mut acc, mail_parser::Group {addresses, ..}| {
                        addresses
                            .iter()
                            .for_each(|Addr {address, ..}| {
                                acc.push(address.as_deref().unwrap().into())
                            });

                        acc
                    })
            }
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

    fn links(&self) -> Vec<String> {
        let collector: Vec<String> = vec![];
        let selector = Selector::parse("a").unwrap();

        self
            .get_html_bodies()
            .fold(collector, |mut memo, part| {
                if let mail_parser::PartType::Html(body) = &part.body {

                    let parsed_body = Html::parse_document(body);

                    for element in parsed_body.select(&selector) {
                        for (key, value) in element.value().attrs() {
                            if key == "href" {
                                memo.push(value.into());
                            }
                        }
                    }
                }

                memo
            })
    }
}
