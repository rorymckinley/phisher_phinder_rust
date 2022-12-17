use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct InputData {
    parsed_mail: InputParsedMail,
}

#[derive(Deserialize)]
struct InputParsedMail {
    subject: String,
    sender_addresses: InputSenderAddresses
}

#[derive(Deserialize)]
struct InputSenderAddresses {
    from: Option<String>,
    reply_to: Option<String>,
    return_path: Option<String>
}

#[derive(Debug, PartialEq, Serialize)]
pub struct OutputData {
    pub parsed_mail: ParsedMail,
}

impl From<InputData> for OutputData  {
    fn from(data: InputData) -> Self {
        Self {
            parsed_mail: data.parsed_mail.into()
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ParsedMail {
    pub subject: String,
    pub sender_addresses: SenderAddresses
}

impl From<InputParsedMail> for ParsedMail {
    fn from(parsed_mail: InputParsedMail) -> Self {
        Self {
            subject: parsed_mail.subject,
            sender_addresses: parsed_mail.sender_addresses.into() 
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct SenderAddresses {
    pub from: Option<EmailAddressData>,
    pub reply_to: Option<EmailAddressData>,
    pub return_path: Option<EmailAddressData>
}

impl SenderAddresses {
    fn to_email_address_data(address: String) -> EmailAddressData {
        EmailAddressData {address, domain: None}
    }
}

impl From<InputSenderAddresses> for SenderAddresses {
    fn from(addresses: InputSenderAddresses) -> Self {
        Self {
            from: addresses.from.map(Self::to_email_address_data),
            reply_to: addresses.reply_to.map(Self::to_email_address_data),
            return_path: addresses.return_path.map(Self::to_email_address_data),
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct EmailAddressData {
    pub address: String,
    pub domain: Option<Domain>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Domain {
    pub name: String,
    pub registrar: Option<String>,
    pub registration_date: Option<DateTime<Utc>>,
    pub abuse_email_address: Option<String>,
}
