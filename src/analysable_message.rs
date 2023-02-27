use mail_parser::{Addr, HeaderValue, Message};
use scraper::{Html, Selector};

pub trait AnalysableMessage {
    fn get_links(&self) -> Vec<String>;
    fn get_from(&self) -> Vec<String>;
    fn get_reply_to(&self) -> Vec<String>;
    fn get_return_path(&self) -> Vec<String>;
    fn get_subject(&self) -> Option<String>;
    fn get_received_headers(&self) -> Vec<String>;
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

        assert_eq!(expected, parsed_mail.get_return_path());
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

        assert_eq!(expected, parsed_mail.get_return_path());
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

        assert_eq!(expected, parsed_mail.get_reply_to())
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

        assert_eq!(vec![String::from("scammer@evildomain.zzz")], parsed_mail.get_reply_to())
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

        assert_eq!(expected, parsed_mail.get_reply_to())
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

        assert_eq!(expected, parsed_mail.get_from());
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
            parsed_mail.get_from()
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

        assert_eq!(None, parsed_mail.get_subject());
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
            parsed_mail.get_subject()
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
            parsed_mail.get_links()
        )
    }

    #[test]
    fn returns_received_header_values() {
        let input = "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
X-Google-Smtp-Source: AA6agR7rW0ljZbXj2cQn9NgC8m6wViE4veg3Wroa/sb4ZEQMZAmVYdUGb9EAPvGvoF9UkmUip/o+\r
X-Received: by 2002:a05:6402:35cf:b0:448:84a9:12cf with SMTP id z15-20020a05640235cf00b0044884a912cfmr745701edc.51.1662506240653;\r
        Tue, 06 Sep 2022 16:17:20 -0700 (PDT)\r
ARC-Authentication-Results: i=1; mx.google.com;\r
       spf=pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) smtp.mailfrom=info@xxx.fr\r
Return-Path: <info@xxx.fr>\r
Received: from foo.bar.com (foo.bar.com. [10.10.10.10])\r
        by mx.google.com with ESMTP id jg8-2002\r
        for <victim@gmail.com>;\r
        Tue, 06 Sep 2022 16:17:20 -0700 (PDT)\r
Received-SPF: pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) client-ip=10.10.10.10;\r
Authentication-Results: mx.google.com;\r
       spf=pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) smtp.mailfrom=info@xxx.fr\r
Received: from not-real-one.com (not-real-one.com )\r
        (envelope-from <g-123-456-789-012@blah.not-real-two.com (g-123-456-789-012@blah.not-real-two.com)>)\r
        by gp13mtaq123 (mtaq-receiver/2.20190311.1) with ESMTP id yA3jJ-_S5g8Z\r
        for <not.real.three@comcast.net>; Thu, 30 May 2019 19:00:22 +0200\r
Date: Tue, 6 Sep 2022 19:17:19 -0400\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@gmail.com>\r
To: victim@gmail.com\r
Message-ID: <Ctht0YgNZJDaAVPvcU36z2Iw9f7Bs7Jge.ecdasmtpin_added_missing@mx.google.com>\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r
MIME-Version: 1.0\r
Content-Type: text/html\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
<a href=\"https://foo.bar/baz\">Click Me</a>
</div>\r";

        let parsed_mail = Message::parse(input.as_bytes()).unwrap();

        let expected: Vec<String> = vec![
            header_value(vec![
                "by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp;\r\n",
                "Tue, 6 Sep 2022 16:17:20 -0700 (PDT)"
            ]),
            header_value(vec![
                "from foo.bar.com (foo.bar.com. [10.10.10.10])\r\n",
                "by mx.google.com with ESMTP id jg8-2002\r\n",
                "for <victim@gmail.com>;\r\n",
                "Tue, 06 Sep 2022 16:17:20 -0700 (PDT)"
            ]),
            header_value(vec![
                "from not-real-one.com (not-real-one.com )\r\n",
                "(envelope-from <g-123-456-789-012@blah.not-real-two.com (g-123-456-789-012@blah.not-real-two.com)>)\r\n",
                "by gp13mtaq123 (mtaq-receiver/2.20190311.1) with ESMTP id yA3jJ-_S5g8Z\r\n",
                "for <not.real.three@comcast.net>; Thu, 30 May 2019 19:00:22 +0200",
            ])
        ];

        assert_eq!(
            expected,
            parsed_mail.get_received_headers()
        )
    }

    fn header_value(lines: Vec<&str>) -> String {
        let mut output: Vec<String> = vec![];

        for (pos, s) in lines.into_iter().enumerate() {
            if pos >  0 {
                output.push("        ".into());
            }

            output.push(s.into())
        }

        output.concat()
    }
}

impl AnalysableMessage for Message<'_> {
    fn get_from(&self) -> Vec<String> {
        // TODO Cover other options
        match self.from() {
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

    fn get_reply_to(&self) -> Vec<String> {
        // TODO Cover other options
        match self.reply_to() {
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

    fn get_return_path(&self) -> Vec<String> {
        // TODO Cover other options
        match self.return_path() {
            HeaderValue::Text(address) => {
                vec![address.to_string()]
            },
            _ => {
                vec![]
            }
        }
    }

    fn get_subject(&self) -> Option<String> {
        self.subject().map(String::from)
    }

    fn get_links(&self) -> Vec<String> {
        let collector: Vec<String> = vec![];
        let selector = Selector::parse("a").unwrap();

        self
            .html_bodies()
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

    fn get_received_headers(&self) -> Vec<String> {
        use std::borrow::Cow;

        self
            .headers()
            .iter()
            .filter(|header| {
                matches!(
                    header.name, mail_parser::HeaderName::Rfc(mail_parser::RfcHeader::Received)
                )
            })
            .map(|header| {
                match &header.value {
                    mail_parser::HeaderValue::Text(val) => {
                        match val {
                            Cow::Borrowed(header) => String::from(*header),
                            Cow::Owned(header) => header.clone()
                        }
                    },
                    _ => String::from("not_supported")
                }
            })
            .collect()
    }
}
