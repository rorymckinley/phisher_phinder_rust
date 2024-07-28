use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::transport::smtp::authentication::Credentials;
use super::configuration::EmailNotificationConfiguration;

pub async fn send_mail(mail: Message, config: &EmailNotificationConfiguration) {
    let mailer = build_mailer(config);
    
    mailer.send(mail).await.unwrap();
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
