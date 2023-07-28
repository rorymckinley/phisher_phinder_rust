use regex::Regex;

#[cfg(test)]
mod mbox_parse_tests {
    use super::*;

    #[test]
    fn splits_string_on_from_marker() {
        assert_eq!(expected(), parse(&input()))
    }

    #[test]
    fn deals_with_broken_mail_body() {
        assert_eq!(expected(), parse(&input_with_broken_entry()))
    }

    fn input() -> String {
        format!("{}\r\n{}\r\n{}", entry_1(), entry_2(), entry_3())
    }

    fn input_with_broken_entry() -> String {
        format!(
            "{}\r\n{}\r\n{}\r\n{}",
            entry_1(),
            broken_entry(),
            entry_2(),
            entry_3()
        )
    }

    fn expected() -> Vec<String> {
        let mail1 = mail_1();
        let mail2 = mail_2();
        let mail3 = mail_3();

        vec![mail1, mail2, mail3]
    }

    fn entry_1() -> String {
        format!(
            "From 123@xxx Sun Jun 11 20:53:34 +0000 2023\r\n{}",
            mail_1()
        )
    }

    fn entry_2() -> String {
        format!(
            "From 123@xxx Sun Jun 11 20:53:35 +0000 2023\r\n{}",
            mail_2()
        )
    }

    fn entry_3() -> String {
        format!(
            "From 123@xxx Sun Jun 11 20:53:36 +0000 2023\r\n{}",
            mail_3()
        )
    }

    fn broken_entry() -> String {
        format!(
            "From 123@xxx Sun Jun 11 20:53:35 +0000 2023\r\n{}",
            broken_mail()
        )
    }

    fn mail_1() -> String {
        "Delivered-To: victim1@test.zzz\r
From: scammer@dodgy.zzz\r
Subject: Dodgy Subject 1\r
\r
blahblah"
            .into()
    }

    fn mail_2() -> String {
        "Delivered-To: victim2@test.zzz\r
From: scammer@dodgy.zzz\r
Subject: Dodgy Subject 2\r
\r
>From the very beginning our goal"
            .into()
    }

    fn mail_3() -> String {
        "Delivered-To: victim3@test.zzz\r
From: scammer@dodgy.zzz\r
Subject: Dodgy Subject 1\r
\r
blahblah"
            .into()
    }

    fn broken_mail() -> String {
        "Delivered-To-Not: victim3@test.zzz\r
From: scammer@dodgy.zzz\r
Subject: Dodgy Subject 1\r
\r
blahblah"
            .into()
    }
}

pub fn parse(mbox_contents: &str) -> Vec<&str> {
    let re = Regex::new(r"(?ms).+?(Delivered-To:.+)\z").unwrap();

    mbox_contents
        .split("\r\nFrom ")
        .filter_map(|snippet| {
            re.captures(snippet)
                .map(|caps| caps.get(1).unwrap().as_str())
        })
        .collect()
}
