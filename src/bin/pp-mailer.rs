use phisher_phinder_rust::data::OutputData;
use phisher_phinder_rust::mailer::{build_mail_definitions, Mailer, Server};

use std::io;

fn get_env(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| panic!("{var} ENV var is required"))
}

#[tokio::main]
async fn main() {
    let mut input_as_string = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut input_as_string) {
            break;
        }
    }

    let input: OutputData = serde_json::from_str(&input_as_string).unwrap();

    let mail_definitions = build_mail_definitions(input.reportable_entities.as_ref());

    let mail_server = Server::new(
        &get_env("PP_SMTP_HOST_URI"),
        &get_env("PP_SMTP_USERNAME"),
        &get_env("PP_SMTP_PASSWORD"),
    );

    let mailer = Mailer::new(mail_server, &get_env("PP_ABUSE_NOTIFICATIONS_FROM_ADDRESS"));
    mailer
        .send_mails(&mail_definitions, &input.message_source.data)
        .await;
}
