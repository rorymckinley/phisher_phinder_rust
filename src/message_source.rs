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

    #[test]
    fn treats_input_as_single_source_if_does_not_start_with_from() {
        assert_eq!(single_source_expected(), parse(&single_source_input()))
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

    fn expected() -> Vec<String> {
        vec![mail_1(), mail_containing_from(), mail_3()]
    }

    fn single_source_expected() -> Vec<String> {
        vec![mail_containing_from()]
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

pub fn parse(mbox_contents: &str) -> Vec<&str> {
    if is_mbox_file(mbox_contents) {
        let re = Regex::new(r"(?ms).+?(Delivered-To:.+)\z").unwrap();

        mbox_contents
            .split("\r\nFrom ")
            .filter_map(|snippet| {
                re.captures(snippet)
                    .map(|caps| caps.get(1).unwrap().as_str())
            })
        .collect()
    } else {
        vec![mbox_contents]
    }
}

fn is_mbox_file(mbox_contents: &str) -> bool {
    let re = Regex::new(r"\AFrom ").unwrap();

    re.is_match(mbox_contents)
}
