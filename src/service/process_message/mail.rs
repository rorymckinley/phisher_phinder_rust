use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::transport::smtp::authentication::Credentials;
use super::configuration::EmailNotificationConfiguration;

pub async fn send_mail(mail: Message, config: &EmailNotificationConfiguration) {
    let mailer = build_mailer(config);
    
    mailer.send(mail).await.unwrap();
    // let mailer = Mailer::new(&message_source);
    // for mail in mails {
    //     mailer.send(mail).await;
    // }
}

fn build_credentials(config: &EmailNotificationConfiguration) -> Credentials {
    Credentials::new(config.username.clone(), config.password.clone())
}

fn build_mailer(config: &EmailNotificationConfiguration) -> AsyncSmtpTransport<Tokio1Executor> {
    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host_uri)
        .unwrap()
        .credentials(build_credentials(config))
        .build()
}

#[cfg(test)]
mod send_mail_tests {
    use crate::mail_trap::MailTrap;
    use lettre::Address;
    use lettre::message::Mailbox;
    use super::*;

    #[test]
    #[ignore]
    fn sends_the_mail() {
        let mail_trap = initialise_mail_trap();

        let email = email();
        let config = config();

        tokio_test::block_on(send_mail(email, &config));

        let received_mail = mail_trap.get_last_email();

        assert_eq!("This is subjective", received_mail.subject.unwrap());
    }

    fn email() -> Message {
        let from = Mailbox::new(None, Address::new("sender", "phishereagle.com").unwrap());
        let to = Mailbox::new(None, Address::new("abuse", "regone.zzz").unwrap());

        Message::builder()
            .from(from)
            .to(to)
            .subject("This is subjective")
            .body(String::from("This is the body"))
            .unwrap()
    }

    fn initialise_mail_trap() -> MailTrap {
        let mail_trap = MailTrap::new(mail_trap_api_token());

        mail_trap.clear_mails();

        mail_trap
    }

    fn mail_trap_api_token() -> String {
        std::env::var("MAILTRAP_API_TOKEN").unwrap()
    }

    fn config() -> EmailNotificationConfiguration {
        EmailNotificationConfiguration {
            host_uri: std::env::var("TEST_SMTP_URI").unwrap(),
            password: std::env::var("TEST_SMTP_PASSWORD").unwrap(),
            username: std::env::var("TEST_SMTP_USERNAME").unwrap(),
        }
    }
    
}
// #[cfg(test)]
// mod test_build_mails {
//     use crate::data::{EmailAddresses, ParsedMail};
//     use lettre::Address;
//     use crate::mailer::Entity;
//     use crate::notification::Notification;
//     use super::*;
//
//     #[test]
//     fn generates_message_for_each_notification() {
//         let record = output_data();
//
//         let mails = build_mails(&record);
//
//         assert_eq!(2, mails.len());
//         assert_mail(&mails[0], "scanner@domainone.zzz", "abuse@regone.zzz");
//         assert_mail(&mails[1], "scanner@domaintwo.zzz", "abuse@regtwo.zzz");
//     }
//
//     fn assert_mail(mail: &Message, entity: &str, recipient: &str) {
//         let envelope = mail.envelope();
//         let expected_recipient = recipient.parse::<Address>().unwrap();
//     }
//
//     fn output_data() -> OutputData {
//         OutputData {
//             parsed_mail: parsed_mail(),
//             message_source: message_source(),
//             notifications: vec![
//                 notification("scammer@domainone.zzz", "abuse@regone.zzz"),
//                 notification("scammer@domaintwo.zzz", "abuse@regtwo.zzz"),
//             ],
//             reportable_entities: None,
//             run_id: None,
//         }
//     }
//
//     fn parsed_mail() -> ParsedMail { 
//         ParsedMail {
//             authentication_results: None,
//             delivery_nodes: vec![],
//             email_addresses: email_addresses(),
//             fulfillment_nodes: vec![],
//             subject: None,
//         }
//     }
//
//     fn message_source() -> MessageSource {
//         MessageSource::new("Delivered-To: blah")
//     }
//
//     fn notification(email_address: &str, recipient: &str) -> Notification {
//         Notification::via_email(Entity::EmailAddress(email_address.into()), recipient.into())
//     }
//
//     fn email_addresses() -> EmailAddresses {
//         EmailAddresses {
//             from: vec![],
//             links: vec![],
//             reply_to: vec![],
//             return_path: vec![],
//         }
//     }
// }
