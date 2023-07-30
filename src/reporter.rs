use crate::authentication_results::AuthenticationResults;
use crate::data::{
    DeliveryNode, EmailAddressData, EmailAddresses, FulfillmentNode, OutputData, ParsedMail,
    ReportableEntities,
};

#[cfg(test)]
mod add_reportable_entities_tests {
    use super::*;
    use crate::authentication_results::{AuthenticationResults, Spf, SpfResult};
    use crate::data::{EmailAddressData, FulfillmentNode, Node, ParsedMail, ReportableEntities};
    use crate::message_source::MessageSource;

    #[test]
    fn adds_reportable_entities_to_output_data() {
        let input = input();

        assert_eq!(expected_output(), add_reportable_entities(input))
    }

    fn input() -> OutputData {
        OutputData::new(parsed_mail(), MessageSource::new(""))
    }

    fn parsed_mail() -> ParsedMail {
        ParsedMail::new(
            Some(authentication_results()),
            delivery_nodes(),
            email_addresses(),
            fulfillment_nodes(),
            None,
        )
    }

    fn authentication_results() -> AuthenticationResults {
        AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: Some(Spf {
                ip_address: None,
                result: Some(SpfResult::Pass),
                smtp_mailfrom: Some("from@test.com".into()),
            }),
        }
    }

    fn delivery_nodes() -> Vec<DeliveryNode> {
        vec![
            DeliveryNode {
                advertised_sender: None,
                observed_sender: None,
                position: 0,
                recipient: None,
                time: None,
                trusted: true,
            },
            DeliveryNode {
                advertised_sender: None,
                observed_sender: None,
                position: 1,
                recipient: None,
                time: None,
                trusted: false,
            },
        ]
    }

    fn email_addresses() -> EmailAddresses {
        EmailAddresses {
            from: vec![EmailAddressData::from_email_address("from@test.com")],
            links: vec![],
            reply_to: vec![EmailAddressData::from_email_address(
                "reply-to@not-test.com",
            )],
            return_path: vec![],
        }
    }

    fn fulfillment_nodes() -> Vec<FulfillmentNode> {
        vec![FulfillmentNode {
            hidden: None,
            visible: Node {
                domain: None,
                registrar: None,
                url: "https://dodgy-node.com".into(),
            },
        }]
    }

    fn expected_output() -> OutputData {
        let delivery_node = reportable_delivery_node();
        let email_address = reportable_email_address();
        let fulfillment_node = reportable_fulfillment_node();

        OutputData {
            parsed_mail: parsed_mail(),
            message_source: MessageSource::new(""),
            reportable_entities: Some(ReportableEntities {
                delivery_nodes: vec![delivery_node],
                email_addresses: EmailAddresses {
                    from: vec![email_address],
                    links: vec![],
                    reply_to: vec![],
                    return_path: vec![],
                },
                fulfillment_nodes: vec![fulfillment_node],
            }),
            run_id: None,
        }
    }

    fn reportable_delivery_node() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: None,
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn reportable_email_address() -> EmailAddressData {
        EmailAddressData::from_email_address("from@test.com")
    }

    fn reportable_fulfillment_node() -> FulfillmentNode {
        FulfillmentNode {
            hidden: None,
            visible: Node {
                domain: None,
                registrar: None,
                url: "https://dodgy-node.com".into(),
            },
        }
    }
}

pub fn add_reportable_entities(data: OutputData) -> OutputData {
    OutputData {
        reportable_entities: Some(ReportableEntities {
            delivery_nodes: extract_reportable_delivery_nodes(&data.parsed_mail),
            email_addresses: extract_reportable_email_addresses(
                &data.parsed_mail.email_addresses,
                data.parsed_mail.authentication_results.as_ref(),
            ),
            fulfillment_nodes: extract_reportable_fulfillment_nodes(&data.parsed_mail),
        }),
        ..data
    }
}

fn extract_reportable_delivery_nodes(parsed_mail: &ParsedMail) -> Vec<DeliveryNode> {
    parsed_mail
        .delivery_nodes
        .clone()
        .into_iter()
        .filter(|node| node.trusted)
        .collect()
}

#[cfg(test)]
mod extract_reportable_email_addresses_tests {
    use super::*;
    use crate::authentication_results::{AuthenticationResults, Dkim, DkimResult};
    use crate::data::EmailAddressData;

    #[test]
    fn includes_link_email_addresses() {
        let parsed_mail = parsed_mail(authentication_results());

        let output = extract_reportable_email_addresses(
            &parsed_mail.email_addresses,
            parsed_mail.authentication_results.as_ref(),
        );

        assert_eq!(link_email_addresses(), output.links);
    }

    #[test]
    fn includes_from_addresses_that_are_validated_via_authentication_results() {
        let parsed_mail = parsed_mail(authentication_results_from_address());

        let output = extract_reportable_email_addresses(
            &parsed_mail.email_addresses,
            parsed_mail.authentication_results.as_ref(),
        );

        assert_eq!(
            vec![
                EmailAddressData::from_email_address("from@test-1.com"),
                EmailAddressData::from_email_address("alsofrom@test-1.com")
            ],
            output.from
        );
    }

    #[test]
    fn includes_reply_to_addresses_that_are_validated_via_authentication_results() {
        let parsed_mail = parsed_mail(authentication_results_from_address());

        let output = extract_reportable_email_addresses(
            &parsed_mail.email_addresses,
            parsed_mail.authentication_results.as_ref(),
        );

        assert_eq!(
            vec![
                EmailAddressData::from_email_address("replyto@test-1.com"),
                EmailAddressData::from_email_address("alsoreplyto@test-1.com")
            ],
            output.reply_to
        );
    }

    #[test]
    fn includes_return_path_that_are_validated_via_authentication_results() {
        let parsed_mail = parsed_mail(authentication_results_from_address());

        let output = extract_reportable_email_addresses(
            &parsed_mail.email_addresses,
            parsed_mail.authentication_results.as_ref(),
        );

        assert_eq!(
            vec![
                EmailAddressData::from_email_address("returnpath@test-1.com"),
                EmailAddressData::from_email_address("alsoreturnpath@test-1.com")
            ],
            output.return_path
        );
    }

    #[test]
    fn does_not_return_from_reply_to_return_path_if_no_authentication_results() {
        let parsed_mail = parsed_mail(authentication_results_from_address());

        let output = extract_reportable_email_addresses(&parsed_mail.email_addresses, None);

        let expected = EmailAddresses {
            from: vec![],
            links: link_email_addresses(),
            reply_to: vec![],
            return_path: vec![],
        };

        assert_eq!(expected, output);
    }

    fn parsed_mail(authentication_results: AuthenticationResults) -> ParsedMail {
        ParsedMail::new(
            Some(authentication_results),
            vec![],
            email_addresses(),
            vec![],
            None,
        )
    }

    fn authentication_results() -> AuthenticationResults {
        AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        }
    }

    fn authentication_results_from_address() -> AuthenticationResults {
        AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Pass),
                selector: None,
                signature_snippet: None,
                user_identifier_snippet: Some("@test-1.com".into()),
            }),
            service_identifier: None,
            spf: None,
        }
    }

    fn email_addresses() -> EmailAddresses {
        EmailAddresses {
            from: vec![
                EmailAddressData::from_email_address("from@test-1.com"),
                EmailAddressData::from_email_address("from@test-2.com"),
                EmailAddressData::from_email_address("alsofrom@test-1.com"),
            ],
            links: vec![
                EmailAddressData::from_email_address("link-1@foo.bar"),
                EmailAddressData::from_email_address("link-2@foo.bar"),
            ],
            reply_to: vec![
                EmailAddressData::from_email_address("replyto@test-1.com"),
                EmailAddressData::from_email_address("replyto@test-2.com"),
                EmailAddressData::from_email_address("alsoreplyto@test-1.com"),
            ],
            return_path: vec![
                EmailAddressData::from_email_address("returnpath@test-1.com"),
                EmailAddressData::from_email_address("returnpath@test-2.com"),
                EmailAddressData::from_email_address("alsoreturnpath@test-1.com"),
            ],
        }
    }

    fn link_email_addresses() -> Vec<EmailAddressData> {
        vec![
            EmailAddressData::from_email_address("link-1@foo.bar"),
            EmailAddressData::from_email_address("link-2@foo.bar"),
        ]
    }
}

fn extract_reportable_email_addresses(
    email_addresses: &EmailAddresses,
    authentication_results_option: Option<&AuthenticationResults>,
) -> EmailAddresses {
    match authentication_results_option {
        Some(auth_results) => EmailAddresses {
            from: filter_valid_email_addresses(email_addresses.from.clone(), auth_results),
            links: email_addresses.links.clone(),
            reply_to: filter_valid_email_addresses(email_addresses.reply_to.clone(), auth_results),
            return_path: filter_valid_email_addresses(
                email_addresses.return_path.clone(),
                auth_results,
            ),
        },
        None => EmailAddresses {
            from: vec![],
            links: email_addresses.links.clone(),
            reply_to: vec![],
            return_path: vec![],
        },
    }
}

fn filter_valid_email_addresses(
    email_addresses: Vec<EmailAddressData>,
    authentication_results: &AuthenticationResults,
) -> Vec<EmailAddressData> {
    email_addresses
        .into_iter()
        .filter(|address| authentication_results.valid(address))
        .collect()
}

fn extract_reportable_fulfillment_nodes(parsed_mail: &ParsedMail) -> Vec<FulfillmentNode> {
    parsed_mail.fulfillment_nodes.clone()
}
