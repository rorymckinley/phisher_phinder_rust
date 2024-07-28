use mail_parser::{Addr, HeaderValue};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use url::Url;

pub struct MailTrap {
    api_token: String,
    url: Url,
}

impl MailTrap {
    pub fn new(api_token: String) -> Self {
        Self {
            api_token,
            url: Url::parse("https://mailtrap.io/api/").unwrap(),
        }
    }

    pub fn clear_mails(&self) {
        let account_id = self.get_account_id();

        let inbox = self.get_inbox();

        let request_url = self
            .url
            .join("accounts/")
            .unwrap()
            .join(&format!("{}/", account_id))
            .unwrap()
            .join("inboxes/")
            .unwrap()
            .join(&format!("{}/", inbox.id))
            .unwrap()
            .join("clean/")
            .unwrap();

        let client = Client::new();

        client
            .patch(request_url)
            .header("Api-Token", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .unwrap();
    }

    fn get_account_id(&self) -> u32 {
        let request_url = self.url.join("accounts").unwrap();

        let client = Client::new();

        let response = client
            .get(request_url)
            .header("Api-Token", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .unwrap();

        let body: Vec<Account> = serde_json::from_str(&response.text().unwrap()).unwrap();

        body[0].id
    }

    pub fn get_inbox(&self) -> Inbox {
        let account_id = self.get_account_id();

        let request_url = self
            .url
            .join("accounts/")
            .unwrap()
            .join(&format!("{}/", account_id))
            .unwrap()
            .join("inboxes")
            .unwrap();

        let client = Client::new();

        let response = client
            .get(request_url)
            .header("Api-Token", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .unwrap();

        let mut body: Vec<Inbox> = serde_json::from_str(&response.text().unwrap()).unwrap();

        body.pop().unwrap()
    }

    pub fn get_last_email(&self) -> Email {
        let last_message = self.get_messages_from_api().pop().unwrap();

        let raw_email = self.download_raw_mail(&last_message.download_path);

        let parsed_mail = mail_parser::Message::parse(raw_email.as_bytes()).unwrap();

        Email::from_parsed_email(parsed_mail)
    }

    pub fn get_all_emails(&self) -> Vec<Email> {
        self.get_messages_from_api()
            .iter()
            .map(|message| {
                let raw_email = self.download_raw_mail(&message.download_path);

                let parsed_mail = mail_parser::Message::parse(raw_email.as_bytes()).unwrap();

                Email::from_parsed_email(parsed_mail)
            })
            .collect()
    }

    fn get_messages_from_api(&self) -> Vec<Message> {
        let account_id = self.get_account_id();

        let inbox = self.get_inbox();

        let client = Client::new();

        let request_url = self
            .url
            .join("accounts/")
            .unwrap()
            .join(&format!("{}/", account_id))
            .unwrap()
            .join("inboxes/")
            .unwrap()
            .join(&format!("{}/", inbox.id))
            .unwrap()
            .join("messages/")
            .unwrap();

        let response = client
            .get(request_url)
            .header("Api-Token", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .unwrap();

        serde_json::from_str(&response.text().unwrap()).unwrap()
    }

    fn download_raw_mail(&self, download_path: &str) -> String {
        let request_url = self.url.join(download_path).unwrap();

        let response = Client::new()
            .get(request_url)
            .header("Api-Token", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .unwrap();

        response.text().unwrap()
    }
}

#[derive(Deserialize, Serialize)]
struct Account {
    id: u32,
    name: String,
    access_levels: Vec<u16>,
}

#[derive(Deserialize, Serialize)]
pub struct Inbox {
    id: u32,
    pub emails_count: u8,
}

#[derive(Deserialize, Serialize)]
struct Message {
    download_path: String,
}

#[derive(Debug, PartialEq)]
pub struct Email {
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub attachment_contents: Option<String>,
}

impl Email {
    pub fn new(from: &str, to: &str, subject: &str, body: &str, attachment_contents: &str) -> Self {
        Self {
            from: Some(from.into()),
            to: Some(to.into()),
            subject: Some(subject.into()),
            body: Some(body.into()),
            attachment_contents: Some(attachment_contents.into()),
        }
    }

    fn from_parsed_email(parsed_mail: mail_parser::Message) -> Self {
        let attachment_contents = if let Some(attachment) = parsed_mail.attachment(0) {
            attachment.text_contents().map(String::from)
        } else {
            None
        };

        Self {
            from: Self::extract_address(parsed_mail.from()),
            to: Self::extract_address(parsed_mail.to()),
            subject: parsed_mail.subject().map(String::from),
            body: parsed_mail.body_text(0).map(String::from),
            attachment_contents,
        }
    }

    fn extract_address(address_header: &HeaderValue) -> Option<String> {
        match address_header {
            HeaderValue::Address(Addr { address, .. }) => {
                address.as_ref().map(|addr| addr.clone().into_owned())
            }
            _ => None,
        }
    }
}
