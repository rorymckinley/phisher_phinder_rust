use crate::analyser::Analyser;
use crate::data::OutputData;
use crate::message_source::MessageSource;
use crate::result::AppResult;
use mail_parser::*;

pub fn analyse_message_source(
    message_source: MessageSource,
    trusted_recipient: Option<&str>
) -> AppResult<OutputData> {
    // TODO Better error handling
    let parsed_mail = Message::parse(message_source.data.as_bytes()).unwrap();

    let analyser = Analyser::new(&parsed_mail);

    // TODO Better error handling
    let parsed_mail = analyser.analyse(trusted_recipient).unwrap();

    //TODO rework analyser.delivery_nodes to take service configuration
    Ok(OutputData::new(parsed_mail, message_source))
}

#[cfg(test)]
mod analyse_message_source_tests {
    use super::*;

    #[test]
    fn performs_analysis_of_the_message_source() {
        let trusted_recipient = "mx.google.com";
        let data = mail_data(trusted_recipient);
        let expected = expected_output_data(message_source(&data), Some(trusted_recipient));

        assert_eq!(
            analyse_message_source(message_source(&data), Some(trusted_recipient)).unwrap(),
            expected
        );
    }

    fn message_source(mail_data: &str) -> MessageSource {
        MessageSource::new(mail_data)
    }

    fn mail_data(trusted_recipient: &str) -> String {
        format!(
            "From 123@xxx Sun Jun 11 20:53:34 +0000 2023\r\n{}",
            mail_body(trusted_recipient)
        )
    }

    fn mail_body(trusted_recipient: &str) -> String {
        format!(
            "{}\r\n{}",
            received_header(trusted_recipient),
            "Subject: Dodgy Subject 1"
        )
    }

    fn received_header(trusted_recipient: &str) -> String {
        let advertised_host = "mail.example.com";
        let actual_host = "dodgy.zzz";
        let ip = "10.0.0.1";
        let by_host = trusted_recipient;
        let date = "Tue, 06 Sep 2022 16:17:21 -0700 (PDT)";

        let from = format!("{advertised_host} ({actual_host} [{ip}])");
        let by = format!("{by_host} with ESMTP id jg8-2002");
        let f_o_r = "<victim@gmail.com>";

        format!(
            "Received: from {from}\r\n        by {by}\r\n        for {f_o_r};\r\n        {date}"
        )
    }

    fn expected_output_data(message_source: MessageSource, trusted_recipient: Option<&str>) -> OutputData {
        let parsed_mail = Message::parse(message_source.data.as_bytes()).unwrap();

        let analyser = Analyser::new(&parsed_mail);

        // TODO Better error handling
        let mail_analysis = analyser.analyse(trusted_recipient).unwrap();

        //TODO rework analyser.delivery_nodes to take service configuration
        OutputData::new(mail_analysis, message_source)
    }
}
