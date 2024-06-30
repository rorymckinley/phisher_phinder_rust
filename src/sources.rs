use crate::message_source::MessageSource;
use regex::Regex;


pub fn create_from_str(mbox_contents: &str) -> Vec<MessageSource> {
    if mbox_contents.is_empty() {
        return vec![];
    }

    if is_mbox_file(mbox_contents) {
        let re = Regex::new(r"(?ms).+?(Delivered-To:.+)\z").unwrap();

        mbox_contents
            .split("\r\nFrom ")
            .filter_map(|snippet| {
                re.captures(snippet)
                    .map(|caps| caps.get(1).unwrap().as_str())
            })
            .map(MessageSource::new)
            .collect()
    } else {
        vec![MessageSource::new(mbox_contents)]
    }
}

fn is_mbox_file(mbox_contents: &str) -> bool {
    let re = Regex::new(r"\AFrom ").unwrap();

    re.is_match(mbox_contents)
}

#[cfg(test)]
mod mbox_create_from_str_tests {
    use super::*;

    #[test]
    fn splits_string_on_from_marker() {
        assert_eq!(expected(), create_from_str(&input()))
    }

    #[test]
    fn deals_with_broken_mail_body() {
        assert_eq!(expected(), create_from_str(&input_with_broken_entry()))
    }

    #[test]
    fn treats_input_as_single_source_if_does_not_start_with_from() {
        assert_eq!(
            single_source_expected(),
            create_from_str(&single_source_input())
        )
    }

    #[test]
    fn returns_empty_collection_if_input_is_empty() {
        assert!(create_from_str("").is_empty());
    }

    fn input() -> String {
        format!("{}\r\n{}\r\n{}", entry_1(), entry_2(), entry_3())
    }

    fn single_source_input() -> String {
        mail_containing_from()
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

    fn expected() -> Vec<MessageSource> {
        vec![
            MessageSource::new(&mail_1()),
            MessageSource::new(&mail_containing_from()),
            MessageSource::new(&mail_3()),
        ]
    }

    fn single_source_expected() -> Vec<MessageSource> {
        vec![MessageSource::new(&mail_containing_from())]
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
            mail_containing_from()
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

    fn mail_containing_from() -> String {
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
