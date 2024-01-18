use crate::authentication_results::AuthenticationResults;
use crate::data::{
    DeliveryNode,
    EmailAddressData,
    EmailAddresses,
    FulfillmentNodesContainer,
    OutputData,
    ParsedMail,
    ReportableEntities,
};

#[cfg(test)]
mod add_reportable_entities_tests {
    use super::*;
    use crate::authentication_results::{AuthenticationResults, Spf, SpfResult};
    use crate::data::{
        Domain,
        DomainCategory,
        EmailAddressData,
        FulfillmentNode,
        HostNode,
        Node,
        ParsedMail,
        ReportableEntities,
        ResolvedDomain,
    };
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
            reportable_delivery_node_1(),
            non_reportable_delivery_node(),
            reportable_delivery_node_2(),
        ]
    }

    fn email_addresses() -> EmailAddresses {
        EmailAddresses {
            from: vec![
                reportable_email_address_authentication()
            ],
            links: vec![],
            reply_to: vec![
                non_reportable_email_address(),
            ],
            return_path: vec![
                reportable_email_address_delivery_node_1(),
                reportable_email_address_delivery_node_2()
            ],
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
        let delivery_nodes = vec![reportable_delivery_node_1(), reportable_delivery_node_2()];
        let fulfillment_node = reportable_fulfillment_node();

        OutputData {
            parsed_mail: parsed_mail(),
            message_source: MessageSource::new(""),
            reportable_entities: Some(ReportableEntities {
                delivery_nodes,
                email_addresses: EmailAddresses {
                    from: vec![
                        reportable_email_address_authentication(),
                    ],
                    links: vec![],
                    reply_to: vec![],
                    return_path: vec![
                        reportable_email_address_delivery_node_1(),
                        reportable_email_address_delivery_node_2(),
                    ],
                },
                fulfillment_nodes_container: FulfillmentNodesContainer {
                    duplicates_removed: false,
                    nodes: vec![fulfillment_node],
                }
            }),
            run_id: None,
        }
    }

    fn reportable_delivery_node_1() -> DeliveryNode {
        build_delivery_node(0, "d-node-1.com", true)
    }

    fn reportable_delivery_node_2() -> DeliveryNode {
        build_delivery_node(1, "d-node-2.com", true)
    }

    fn non_reportable_delivery_node() -> DeliveryNode {
        build_delivery_node(2, "d-node-3.com", false)
    }

    fn reportable_email_address_authentication() -> EmailAddressData {
        EmailAddressData::from_email_address("from@test.com")
    }

    fn reportable_email_address_delivery_node_1() -> EmailAddressData {
        EmailAddressData::from_email_address("return-to@d-node-1.com")
    }

    fn reportable_email_address_delivery_node_2() -> EmailAddressData {
        EmailAddressData::from_email_address("return-to@d-node-2.com")
    }

    fn non_reportable_email_address() -> EmailAddressData {
        EmailAddressData::from_email_address("reply-to@d-node-3.com")
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

    fn build_delivery_node(position: usize, host: &str, trusted: bool) -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                host: Some(host.into()),
                domain: Some(build_domain(host)),
                infrastructure_provider: None,
                ip_address: None,
                registrar: None
            }),
            position,
            recipient: None,
            time: None,
            trusted,
        }
    }

    fn build_domain(name: &str) -> Domain {
        Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: name.into(),
            registration_date: None,
            resolved_domain: Some(ResolvedDomain {
                abuse_email_address: None,
                name: name.into(),
                registration_date: None,
            })
        }
    }
}

pub fn add_reportable_entities(data: OutputData) -> OutputData {
    let delivery_nodes = extract_reportable_delivery_nodes(&data.parsed_mail);
    let email_addresses = extract_reportable_email_addresses(
        &data.parsed_mail.email_addresses,
        data.parsed_mail.authentication_results.as_ref(),
        &delivery_nodes
    );

    OutputData {
        reportable_entities: Some(ReportableEntities {
            delivery_nodes,
            email_addresses,
            fulfillment_nodes_container: extract_reportable_fulfillment_nodes(&data.parsed_mail),
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
    use crate::data::{Domain, DomainCategory, EmailAddressData, HostNode, ResolvedDomain};

    #[test]
    fn includes_link_email_addresses() {
        let parsed_mail = parsed_mail(authentication_results());

        let output = extract_reportable_email_addresses(
            &parsed_mail.email_addresses,
            parsed_mail.authentication_results.as_ref(),
            &delivery_nodes()
        );

        assert_eq!(link_email_addresses(), output.links);
    }

    #[test]
    fn includes_from_addresses_that_are_validated() {
        let parsed_mail = parsed_mail(authentication_results_from_address());

        let output = extract_reportable_email_addresses(
            &parsed_mail.email_addresses,
            parsed_mail.authentication_results.as_ref(),
            &delivery_nodes()
        );

        assert_eq!(
            vec![
                EmailAddressData::from_email_address("from@test-1.com"),
                EmailAddressData::from_email_address("alsofrom@test-1.com"),
                EmailAddressData::from_email_address("from@node-1.com"),
                EmailAddressData::from_email_address("from@node-2.com"),
            ],
            output.from
        );
    }

    #[test]
    fn includes_reply_to_addresses_that_are_validated() {
        let parsed_mail = parsed_mail(authentication_results_from_address());

        let output = extract_reportable_email_addresses(
            &parsed_mail.email_addresses,
            parsed_mail.authentication_results.as_ref(),
            &delivery_nodes()
        );

        assert_eq!(
            vec![
                EmailAddressData::from_email_address("replyto@test-1.com"),
                EmailAddressData::from_email_address("alsoreplyto@test-1.com"),
                EmailAddressData::from_email_address("replyto@node-1.com"),
                EmailAddressData::from_email_address("replyto@node-2.com"),
            ],
            output.reply_to
        );
    }

    #[test]
    fn includes_return_path_that_are_validated() {
        let parsed_mail = parsed_mail(authentication_results_from_address());

        let output = extract_reportable_email_addresses(
            &parsed_mail.email_addresses,
            parsed_mail.authentication_results.as_ref(),
            &delivery_nodes()
        );

        assert_eq!(
            vec![
                EmailAddressData::from_email_address("returnpath@test-1.com"),
                EmailAddressData::from_email_address("alsoreturnpath@test-1.com"),
                EmailAddressData::from_email_address("returnpath@node-1.com"),
                EmailAddressData::from_email_address("returnpath@node-2.com"),
            ],
            output.return_path
        );
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
                EmailAddressData::from_email_address("from@node-1.com"),
                EmailAddressData::from_email_address("from@node-2.com"),
            ],
            links: vec![
                EmailAddressData::from_email_address("link-1@foo.bar"),
                EmailAddressData::from_email_address("link-2@foo.bar"),
            ],
            reply_to: vec![
                EmailAddressData::from_email_address("replyto@test-1.com"),
                EmailAddressData::from_email_address("replyto@test-2.com"),
                EmailAddressData::from_email_address("alsoreplyto@test-1.com"),
                EmailAddressData::from_email_address("replyto@node-1.com"),
                EmailAddressData::from_email_address("replyto@node-2.com"),
            ],
            return_path: vec![
                EmailAddressData::from_email_address("returnpath@test-1.com"),
                EmailAddressData::from_email_address("returnpath@test-2.com"),
                EmailAddressData::from_email_address("alsoreturnpath@test-1.com"),
                EmailAddressData::from_email_address("returnpath@node-1.com"),
                EmailAddressData::from_email_address("returnpath@node-2.com"),
            ],
        }
    }

    fn link_email_addresses() -> Vec<EmailAddressData> {
        vec![
            EmailAddressData::from_email_address("link-1@foo.bar"),
            EmailAddressData::from_email_address("link-2@foo.bar"),
        ]
    }

    fn delivery_nodes() -> Vec<DeliveryNode> {
         vec![
             build_delivery_node("node-1.com"),
             build_delivery_node("node-2.com"),
         ]
    }

    fn build_delivery_node(host: &str) -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                host: Some(host.into()),
                domain: Some(build_domain(host)),
                infrastructure_provider: None,
                ip_address: None,
                registrar: None
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn build_domain(name: &str) -> Domain {
        Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: name.into(),
            registration_date: None,
            resolved_domain: Some(ResolvedDomain {
                abuse_email_address: None,
                name: name.into(),
                registration_date: None,
            })
        }
    }
}

fn extract_reportable_email_addresses(
    email_addresses: &EmailAddresses,
    authentication_results_option: Option<&AuthenticationResults>,
    delivery_nodes: &[DeliveryNode]
) -> EmailAddresses {
    EmailAddresses {
        from: filter_valid_email_addresses(
            email_addresses.from.clone(),
            authentication_results_option,
            delivery_nodes,
        ),
        links: email_addresses.links.clone(),
        reply_to: filter_valid_email_addresses(
            email_addresses.reply_to.clone(),
            authentication_results_option,
            delivery_nodes,
        ),
        return_path: filter_valid_email_addresses(
            email_addresses.return_path.clone(),
            authentication_results_option,
            delivery_nodes,
        ),
    }
}

fn filter_valid_email_addresses(
    email_addresses: Vec<EmailAddressData>,
    authentication_results_option: Option<&AuthenticationResults>,
    delivery_nodes: &[DeliveryNode]
) -> Vec<EmailAddressData> {
    let validator = DeliveryNodeEmailAddressValidator::new(delivery_nodes);
    email_addresses
        .into_iter()
        .filter(|address| {
            let auth_check = if let Some(authentication_results) = authentication_results_option {
                authentication_results.valid(address)
            } else {
                false
            };
            auth_check || validator.valid(address)
        })
        .collect()
}

#[cfg(test)]
mod filter_valid_email_addresses_tests {
    use crate::data::{Domain, DomainCategory, HostNode, ResolvedDomain};
    use crate::authentication_results::{Dkim, DkimResult};
    use super::*;

    #[test]
    fn returns_email_addresses_validated_by_auth_results_or_delivery_nodes() {
        let expected = vec![
            EmailAddressData::from_email_address("from@test-1.com"),
            EmailAddressData::from_email_address("alsofrom@test-1.com"),
            EmailAddressData::from_email_address("from@d-node-1.com"),
            EmailAddressData::from_email_address("from@d-node-2.com"),
        ];

        assert_eq!(
            expected,
            filter_valid_email_addresses(
                email_addresses(),
                Some(&authentication_results()),
                &delivery_nodes()
            )
        )
    }

    #[test]
    fn returns_empty_collection_if_no_authentication_results_or_delivery_nodes() {
        let expected: Vec<EmailAddressData> = vec![];

        assert_eq!(
            expected,
            filter_valid_email_addresses(
                email_addresses(),
                None,
                &[]
            )
        );
    }

    fn email_addresses() -> Vec<EmailAddressData> {
        vec![
            EmailAddressData::from_email_address("from@test-1.com"),
            EmailAddressData::from_email_address("from@test-2.com"),
            EmailAddressData::from_email_address("alsofrom@test-1.com"),
            EmailAddressData::from_email_address("from@d-node-1.com"),
            EmailAddressData::from_email_address("from@d-node-3.com"),
            EmailAddressData::from_email_address("from@d-node-2.com"),
        ]
    }

    fn authentication_results() -> AuthenticationResults {
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

    fn delivery_nodes() -> Vec<DeliveryNode> {
        vec![
            build_delivery_node("d-node-1.com"),
            build_delivery_node("d-node-2.com"),
        ]
    }

    fn build_delivery_node(host: &str) -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                host: Some(host.into()),
                domain: Some(build_domain(host)),
                infrastructure_provider: None,
                ip_address: None,
                registrar: None
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn build_domain(name: &str) -> Domain {
        Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: name.into(),
            registration_date: None,
            resolved_domain: Some(ResolvedDomain {
                abuse_email_address: None,
                name: name.into(),
                registration_date: None,
            })
        }
    }
}

fn extract_reportable_fulfillment_nodes(parsed_mail: &ParsedMail) -> FulfillmentNodesContainer {
    let mut nodes =  parsed_mail.fulfillment_nodes.clone();

    nodes.sort_unstable_by(|a, b| a.functional_cmp(b));

    nodes.dedup_by(|a, b| a.functional_eq(b));

    FulfillmentNodesContainer {
        duplicates_removed: nodes.len() != parsed_mail.fulfillment_nodes.len(),
        nodes,
    }


}

#[cfg(test)]
mod extract_reportable_fulfillment_nodes_tests {
    use crate::data::FulfillmentNode;
    use super::*;

    #[test]
    fn returns_fulfillment_nodes() {
        let input = parsed_mail(fulfillment_nodes());
        let expected = FulfillmentNodesContainer {
            duplicates_removed: false,
            nodes: fulfillment_nodes_sorted(),
        };

        assert_eq!(expected, extract_reportable_fulfillment_nodes(&input));
    }

    #[test]
    fn remove_duplicates_from_the_collection() {
        let input = parsed_mail(fulfillment_nodes_with_duplicates());
        let expected = FulfillmentNodesContainer {
            duplicates_removed: true,
            nodes: fulfillment_nodes_sorted(),
        };

        assert_eq!(expected, extract_reportable_fulfillment_nodes(&input));
    }

    fn parsed_mail(fulfillment_nodes: Vec<FulfillmentNode>) -> ParsedMail {
        ParsedMail::new(
            None,
            vec![],
            EmailAddresses {
                from: vec![],
                links: vec![],
                reply_to: vec![],
                return_path: vec![]
            },
            fulfillment_nodes,
            None,
        )
    }

    fn fulfillment_nodes() -> Vec<FulfillmentNode> {
        vec![
            FulfillmentNode::new("http://foo.biz"),
            FulfillmentNode::new("http://foo.baz"),
            FulfillmentNode::new("http://foo.bar"),
        ]
    }

    fn fulfillment_nodes_with_duplicates() -> Vec<FulfillmentNode> {
        vec![
            FulfillmentNode::new("http://foo.biz"),
            FulfillmentNode::new("http://foo.baz"),
            FulfillmentNode::new("http://foo.biz"),
            FulfillmentNode::new("http://foo.bar"),
            FulfillmentNode::new("http://foo.baz"),
        ]
    }

    fn fulfillment_nodes_sorted() -> Vec<FulfillmentNode> {
        vec![
            FulfillmentNode::new("http://foo.bar"),
            FulfillmentNode::new("http://foo.baz"),
            FulfillmentNode::new("http://foo.biz"),
        ]
    }
}

#[cfg(test)]
mod delivery_node_email_address_validator_tests {
    use crate::data::{Domain, DomainCategory, EmailAddressData, HostNode, ResolvedDomain};
    use super::*;

    #[test]
    fn returns_true_if_delivery_node_domain_matches_email_address_domain() {
        let nodes = delivery_nodes();
        let validator = DeliveryNodeEmailAddressValidator::new(&nodes);
        let email_address = EmailAddressData::from_email_address("a@baz.com");

        assert!(validator.valid(&email_address))
    }

    #[test]
    fn returns_false_if_delivery_node_domain_does_not_match_email_address_domain() {
        let nodes = delivery_nodes();
        let validator = DeliveryNodeEmailAddressValidator::new(&nodes);
        let email_address = EmailAddressData::from_email_address("a@boz.com");

        assert!(!validator.valid(&email_address))
    }

    fn delivery_nodes() -> Vec<DeliveryNode> {
        vec![
            build_delivery_node("bar.com"),
            build_delivery_node("baz.com"),
            build_delivery_node("biz.com"),
        ]
    }

    fn build_delivery_node(host: &str) -> DeliveryNode {

        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                host: None,
                domain: Some(build_domain(host)),
                infrastructure_provider: None,
                ip_address: None,
                registrar: None
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn build_domain(name: &str) -> Domain {
        Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: name.into(),
            registration_date: None,
            resolved_domain: Some(ResolvedDomain {
                abuse_email_address: None,
                name: name.into(),
                registration_date: None,
            })
        }
    }
}

struct DeliveryNodeEmailAddressValidator<'a> {
    delivery_nodes: &'a[DeliveryNode]
}

impl<'a> DeliveryNodeEmailAddressValidator<'a> {
    fn new(delivery_nodes: &'a[DeliveryNode]) -> Self {
        Self { delivery_nodes }
    }

    pub fn valid(&self, email_address: &EmailAddressData) -> bool {
        !self
            .nodes_matching_email_domain(email_address)
            .is_empty()
    }

    fn nodes_matching_email_domain(&self, email_address: &EmailAddressData) -> Vec<&DeliveryNode> {
        self
            .delivery_nodes
            .iter()
            .filter(|node| node.domain_matches(email_address))
            .collect()
    }
}
