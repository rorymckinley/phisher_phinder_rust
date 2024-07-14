use crate::errors::AppError;
use crate::mailer::Entity;
use crate::notification::Notification;
use crate::result::AppResult;
use crate::data::OutputData;
use crate::service::process_message::configuration::{AbuseNotificationConfiguration, Configuration};
use lettre::Message;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, MultiPart, SinglePart};

struct EmailConfiguration<'a> {
    pub author_name: &'a str,
    pub entity: &'a Entity,
    pub message_source: &'a str,
    pub recipient_address: &'a str,
    pub sender_address: &'a str,
    pub test_recipient: Option<&'a str>,
}

pub fn build_abuse_notifications(
    data: &OutputData,
    config: &Configuration
) -> AppResult<Vec<Message>> {
    if let Some(notification_config) = &config.abuse_notifications {
        let AbuseNotificationConfiguration {
            author_name,
            from_address,
            test_recipient
        } = notification_config;

        let notifications = data.notifications
            .iter()
            .map(|notification| {
                let Notification::Email(entity, recipient_address) = notification;

                let email_config = EmailConfiguration {
                    author_name,
                    entity,
                    message_source: &data.message_source.data,
                    recipient_address,
                    sender_address: from_address,
                    test_recipient: test_recipient.as_deref(),
                };

                build_email_to_provider(email_config)
            })
        .collect();

        Ok(notifications)
    } else {
        Err(AppError::NotificationConfigurationAbsent)
    }
}

fn build_email_to_provider(config: EmailConfiguration) -> Message {
    let entity = match config.entity {
        Entity::EmailAddress(email) => email,
        Entity::Node(node) => node,
    };

    let to = if let Some(address) = config.test_recipient {
        address
    } else {
        config.recipient_address
    };

    Message::builder()
        .from(config.sender_address.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(format!("Please investigate: {entity} potentially involved in fraud"))
        .multipart(
            MultiPart::mixed()
            .singlepart(build_body(entity, &config))
            .singlepart(build_attachment(config.message_source)),
        )
        .unwrap()

}

fn build_body(entity: &str, config: &EmailConfiguration) -> SinglePart {
    let original_recipient_header = match config.test_recipient {
        Some(_) => format!("This mail would be sent to: {}\n\n", config.recipient_address),
        None => "".into(),
    };
    let author_name = config.author_name;

    let text = format!(
        "\
        {original_recipient_header}\
        Hello\n\n\
        I recently received a phishing email that suggests that `{entity}` \
        may be supporting phishing activities.\n\n\
        The original email is attached, can you \
        please take the appropriate action?\n\
        \n\
        Thank you,\n\
        {author_name}\n\
        "
    );

    SinglePart::builder()
        .header(ContentType::TEXT_PLAIN)
        .body(text)
}

fn build_attachment(raw_email: &str) -> SinglePart {
    Attachment::new(String::from("suspect_email.eml"))
        .body(String::from(raw_email), ContentType::TEXT_PLAIN)
}

#[cfg(test)]
mod build_abuse_notifications_tests {
    use chrono::*;
    use crate::errors::AppError;
    use crate::data::{EmailAddresses, OutputData, ParsedMail};
    use crate::run::Run;
    use rusqlite::Connection;
    use test_support::*;

    use super::*;

    #[test]
    fn builds_email_messages_for_each_notification() {
        let notification_config = AbuseNotificationConfiguration {
            author_name: "Fred Flintstone".into(),
            from_address: "notifications@phishereagle.com".into(),
            test_recipient: None,
        };
        let run = build_run();
        let config = build_config(Some(notification_config), None);

        let notifications = build_abuse_notifications(&run.data, &config).unwrap();

        let mut notifications_as_text: Vec<Box<Vec<u8>>> = notifications
            .iter()
            .map(|notification| Box::new(notification.formatted()))
            .collect();

        let notification_2 = notifications_as_text.pop().unwrap();
        let notification_1 = notifications_as_text.pop().unwrap();

        let email_1 = parse_email(&notification_1);
        let email_2 = parse_email(&notification_2);

        assert_eq!(get_address(email_1.to()), "abuse@providerone.zzz");
        assert_eq!(get_address(email_2.to()), "abuse@providertwo.zzz");

        assert_eq!(
            email_1.subject().unwrap(),
            "Please investigate: scam@fake.zzz potentially involved in fraud"
        );
        assert_eq!(
            email_2.subject().unwrap(),
            "Please investigate: https://scam.zzz potentially involved in fraud"
        );
        assert_eq!(extract_attachment_body(&email_1), message_source_contents("\r\n"));
        assert_eq!(get_address(email_1.from()), "notifications@phishereagle.com");

        let email_1_body = extract_body_from(email_1);
        assert!(email_1_body.contains("Fred Flintstone"));
    }

    #[test]
    fn only_uses_trusted_recipient_if_provided() {
        let notification_config = AbuseNotificationConfiguration {
            author_name: "Fred Flintstone".into(),
            from_address: "notifications@phishereagle.com".into(),
            test_recipient: Some("recipient@test.zzz".into()),
        };
        let run = build_run();
        let config = build_config(Some(notification_config), None);

        let notifications = build_abuse_notifications(&run.data, &config).unwrap();

        let mut notifications_as_text: Vec<Box<Vec<u8>>> = notifications
            .iter()
            .map(|notification| Box::new(notification.formatted()))
            .collect();

        let notification_2 = notifications_as_text.pop().unwrap();
        let notification_1 = notifications_as_text.pop().unwrap();

        let email_1 = parse_email(&notification_1);
        let email_2 = parse_email(&notification_2);

        assert_eq!(get_address(email_1.to()), "recipient@test.zzz");
        assert_eq!(get_address(email_2.to()), "recipient@test.zzz");

        assert_eq!(
            email_1.subject().unwrap(),
            "Please investigate: scam@fake.zzz potentially involved in fraud"
        );
        assert_eq!(
            email_2.subject().unwrap(),
            "Please investigate: https://scam.zzz potentially involved in fraud"
        );
        assert_eq!(extract_attachment_body(&email_1), message_source_contents("\r\n"));
        assert_eq!(get_address(email_1.from()), "notifications@phishereagle.com");

        let email_1_body = extract_body_from(email_1);
        assert!(email_1_body.contains("Fred Flintstone"));
    }

    #[test]
    fn returns_an_error_if_no_abuse_notification_configuration() {
        let run = build_run();
        let config = build_config(None, None);

        match build_abuse_notifications(&run.data, &config) {
            Ok(_) => panic!("Did not return an error"),
            Err(e) => {
                match e {
                    AppError::NotificationConfigurationAbsent => (),
                    _ => panic!("Unexpected error: {e}")
                }
            }
        }
    }

    fn build_run() -> Run {
        Run {
            id: 1234,
            created_at: Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, 30).unwrap(),
            data: OutputData {
                message_source: message_source(),
                notifications: vec![
                    Notification::via_email(
                        Entity::EmailAddress("scam@fake.zzz".into()),
                        "abuse@providerone.zzz".into()
                    ),
                    Notification::via_email(
                        Entity::Node("https://scam.zzz".into()),
                        "abuse@providertwo.zzz".into()
                    ),
                ],
                parsed_mail: ParsedMail {
                    authentication_results: None,
                    delivery_nodes: vec![],
                    email_addresses: EmailAddresses {
                        from: vec![],
                        links: vec![],
                        reply_to: vec![],
                        return_path: vec![],
                    },
                    fulfillment_nodes: vec![],
                    subject: None,
                },
                reportable_entities: None,
                run_id: None,
            },
            message_source: message_source()
        }
    }

    fn build_config(
        abuse_notifications: Option<AbuseNotificationConfiguration>,
        trusted_recipient: Option<&str>
    ) -> Configuration {

        Configuration {
            abuse_notifications,
            db_connection: Connection::open_in_memory().unwrap(),
            email_notifications: None,
            inputs: vec![],
            trusted_recipient,
        }
    }
}

#[cfg(test)]
mod build_email_to_provider_tests {
    use crate::mailer::Entity;
    use crate::message_source::MessageSource;
    use mail_parser::{Message, MimeHeaders};
    use test_support::*;
    use super::*;

    #[test]
    fn builds_mail_with_recipient_in_to() {
        let entity = Entity::EmailAddress("fake@scammer.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: None
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(get_address(email.to()), "abuse@providerone.zzz");
    }

    #[test]
    fn builds_mail_with_sender_in_from() {
        let entity = Entity::EmailAddress("fake@scammer.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: None
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(get_address(email.from()), "sender@phishereagle.com");
    }

    #[test]
    fn builds_subject_for_email_address_entity() {
        let entity = Entity::EmailAddress("fake@scammer.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: None
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(
            email.subject().unwrap(),
            "Please investigate: fake@scammer.zzz potentially involved in fraud"
        );
    }

    #[test]
    fn builds_subject_for_node_entity() {
        let entity = Entity::Node("http://scam.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: None,
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(
            email.subject().unwrap(),
            "Please investigate: http://scam.zzz potentially involved in fraud"
        );
    }

    #[test]
    fn builds_body_for_email_address_entity() {
        let entity = Entity::EmailAddress("fake@scammer.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: None,
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(
            extract_body_from(email),
            expected_body("fake@scammer.zzz", None)
        );
    }

    #[test]
    fn includes_actual_recipient_in_body_for_email_address_if_test_recipient_is_set() {
        let entity = Entity::EmailAddress("fake@scammer.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: Some("recipient@test.zzz"),
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(
            extract_body_from(email),
            expected_body("fake@scammer.zzz", Some("abuse@providerone.zzz"))
        );
    }

    #[test]
    fn builds_body_for_node_entity() {
        let entity = Entity::Node("http://scam.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: None,
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(
            extract_body_from(email),
            expected_body("http://scam.zzz", None)
        );
    }

    #[test]
    fn includes_actual_recipient_in_body_for_node_entity_if_test_recipient_is_present() {
        let entity = Entity::Node("http://scam.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: Some("recipient@test.zzz"),
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(
            extract_body_from(email),
            expected_body("http://scam.zzz", Some("abuse@providerone.zzz"))
        );
    }

    #[test]
    fn attaches_original_email_as_attachment() {
        let entity = Entity::EmailAddress("fake@scammer.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: None,
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(extract_attachment_body(&email),
            String::from("This\r\nis the raw mail\r\nsource")
        );

        assert_eq!(get_attachment_name(&email), "suspect_email.eml");
    }

    #[test]
    fn sets_the_to_based_on_test_recipient_if_provided() {
        let entity = Entity::EmailAddress("fake@scammer.zzz".into());
        let source = message_source();

        let config = EmailConfiguration {
            author_name: "Jo Bloggs",
            entity: &entity,
            message_source: &source.data,
            recipient_address: "abuse@providerone.zzz",
            sender_address: "sender@phishereagle.com",
            test_recipient: Some("recipient@test.zzz"),
        };

        let generated_email = build_email_to_provider(config).formatted();

        let email = parse_email(&generated_email);

        assert_eq!(get_address(email.to()), "recipient@test.zzz");
    }


    fn message_source() -> MessageSource {
        MessageSource::new("This\nis the raw mail\nsource")
    }

    fn expected_body(entity: &str, original_recipient: Option<&str>) -> String {
        let recipient_header = if let Some(recipient) = original_recipient {
            format!("This mail would be sent to: {}\n\n", recipient)
        } else {
            "".into()
        };

        format!(
            "\
                {}\
                Hello\n\
                \n\
                I recently received a phishing email that suggests that \
                `{entity}` may be supporting phishing activities.\n\
                \n\
                The original email is attached, can you please take the appropriate action?\n\
                \n\
                Thank you,\n\
                Jo Bloggs\n\
            ",
            recipient_header
        )
    }

    fn get_attachment_name(mail: &Message) -> String {
        mail.attachment(0).unwrap().attachment_name().unwrap().into()
    }
}

#[cfg(test)]
mod test_support {
    use crate::message_source::MessageSource;
    use mail_parser::{Addr, HeaderValue, Message, MessagePart, PartType};
    use std::borrow::Borrow;

    pub fn parse_email(email_as_text: &[u8]) -> Message {
        Message::parse(email_as_text).unwrap()
    }

    pub fn get_address<'a>(address: &'a HeaderValue) -> &'a str {
        if let HeaderValue::Address(Addr{address: Some(address_cow), ..})  = address {
            address_cow.borrow()
        } else {
            "notthedroidyouarelookingfor"
        }
    }

    pub fn extract_body_from(mail: Message) -> String {
       mail.body_text(0).unwrap().into_owned()
    }

    pub fn extract_attachment_body(mail: &Message) -> String {
        if let Some(MessagePart{ body: PartType::Text(message_cow), ..}) = mail.attachment(0) {
            message_cow.clone().into_owned()
        } else {
            "nottheattachmentyouarelookingfor".into()
        }
    }

    pub fn message_source_contents(line_break: &str) -> String {
        format!("This{line_break}is the raw mail{line_break}source")
    }

    pub fn message_source() -> MessageSource {
        MessageSource::new(&message_source_contents("\n"))
    }
}
