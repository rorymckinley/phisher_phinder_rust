use assert_cmd::Command;
use phisher_phinder_rust::mail_trap::MailTrap;

#[test]
fn send_mail_to_abuse_contacts() {
    let mail_trap = MailTrap::new(mail_trap_api_token());

    mail_trap.clear_mails();

    let mut cmd = Command::cargo_bin("pp-mailer").unwrap();

    cmd
        .write_stdin(json_input())
        .args(["--human"])
        .env("PP_ABUSE_NOTIFICATIONS_FROM_ADDRESS", from_address())
        .env("PP_SMTP_HOST_URI", uri())
        .env("PP_SMTP_PASSWORD", password())
        .env("PP_SMTP_USERNAME", username())
        .assert()
        .success();

    assert_eq!(
        1,
        mail_trap.get_inbox().emails_count
    );

    assert_eq!(from_address(), mail_trap.get_last_email().from.unwrap());
}

fn mail_trap_api_token() -> String {
    std::env::var("MAILTRAP_API_TOKEN").unwrap()
}

fn uri() -> String {
    std::env::var("TEST_SMTP_URI").unwrap()
}

fn password() -> String {
    std::env::var("TEST_SMTP_PASSWORD").unwrap()
}

fn username() -> String {
    std::env::var("TEST_SMTP_USERNAME").unwrap()
}

fn from_address() -> String {
    std::env::var("TEST_NOTIFICATIONS_FROM").unwrap()
}

fn json_input() -> String {
    use serde_json::json;

    json!({
        "parsed_mail": {
            "delivery_nodes": [],
            "fulfillment_nodes": [],
            "subject": "We’re sorry that we didn’t touch base with you earlier. f309",
            "email_addresses": {
                "from": [{
                    "address": "PIBIeSRqUtiEw1NCg4@fake.net",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "fake.net",
                        "registration_date": null,
                    },
                    "registrar": {
                        "abuse_email_address": "abuse@regone.zzz",
                        "name": "Reg One",
                    },
                }],
                "links": [],
                "reply_to": [],
                "return_path": []
            }
        },
        "raw_mail": "",
        "reportable_entities": {
            "delivery_nodes": [],
            "email_addresses": {
                "from": [{
                    "address": "PIBIeSRqUtiEw1NCg4@fake.net",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "fake.net",
                        "registration_date": null,
                    },
                    "registrar": {
                        "abuse_email_address": "abuse@regone.zzz",
                        "name": "Reg One",
                    },
                }],
                "links": [],
                "reply_to": [],
                "return_path": []
            },
            "fulfillment_nodes": [],
        }
    }).to_string()
}
