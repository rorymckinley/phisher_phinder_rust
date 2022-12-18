use chrono::prelude::*;
use crate:: data::{Domain, DomainCategory, EmailAddressData, OutputData};
use rdap_client::bootstrap::Bootstrap;
use rdap_client::Client;

#[cfg(test)]
mod populate_tests {
    use super::*;
    use crate:: data::{Domain, EmailAddressData, ParsedMail, SenderAddresses};
    use crate::mountebank::*;

    #[test]
    fn populates_output_object_with_domain_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let mut input = input_data();

        tokio_test::block_on(populate(&bootstrap, &mut input));

        assert_eq!(
            output_data(),
            input
        )
    }

    #[test]
    fn output_object_with_no_servers_does_not_create_domain_instances() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let mut input = input_data_no_servers();

        tokio_test::block_on(populate(&bootstrap, &mut input));

        assert_eq!(input_data_no_servers(), input);
    }

    #[test]
    fn output_object_with_no_sender_address_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let mut input = input_data_no_sender_addresses();

        tokio_test::block_on(populate(&bootstrap, &mut input));

        assert_eq!(input_data_no_sender_addresses(), input);
    }

    fn input_data() -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                subject: Some("Does not matter".into()),
                sender_addresses: SenderAddresses {
                    from: Some(
                        EmailAddressData {
                            address: "someone@fake.net".into(),
                            domain: None,
                        }
                    ),
                    reply_to: Some(
                        EmailAddressData {
                            address: "anyone@possiblynotfake.com".into(),
                            domain: None,
                        },
                    ),
                    return_path: Some(
                        EmailAddressData {
                            address: "everyone@morethanlikelyfake.net".into(),
                            domain: None,
                        },
                    )

                }
            }
        }
    }

    fn input_data_no_servers() -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                subject: Some("Does not matter".into()),
                sender_addresses: SenderAddresses {
                    from: Some(
                        EmailAddressData {
                            address: "someone@fake.bogus".into(),
                            domain: None,
                        }
                    ),
                    reply_to: Some(
                        EmailAddressData {
                            address: "anyone@possiblynotfake.bogus".into(),
                            domain: None,
                        },
                    ),
                    return_path: Some(
                        EmailAddressData {
                            address: "everyone@morethanlikelyfake.bogus".into(),
                            domain: None,
                        },
                    )

                }
            }
        }
    }

    fn input_data_no_sender_addresses() -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                subject: Some("Does not matter".into()),
                sender_addresses: SenderAddresses {
                    from: None,
                    reply_to: None,
                    return_path: None,
                }
            }
        }
    }

    fn output_data() -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                subject: Some("Does not matter".into()),
                sender_addresses: SenderAddresses {
                    from: Some(
                        EmailAddressData {
                            address: "someone@fake.net".into(),
                            domain: domain_object(
                                "fake.net",
                                Some("Reg One"),
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
                                Some("abuse@regone.zzz")
                            ),
                        }
                    ),
                    reply_to: Some(
                        EmailAddressData {
                            address: "anyone@possiblynotfake.com".into(),
                            domain: domain_object(
                                "possiblynotfake.com",
                                Some("Reg Two"),
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()),
                                Some("abuse@regtwo.zzz"),
                            ),
                        },
                    ),
                    return_path: Some(
                        EmailAddressData {
                            address: "everyone@morethanlikelyfake.net".into(),
                            domain: domain_object(
                                "morethanlikelyfake.net",
                                Some("Reg Three"),
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()),
                                Some("abuse@regthree.zzz"),
                            ),
                        },
                    )
                }
            }
        }
    }

    fn domain_object(
        name: &str,
        registrar: Option<&str>,
        registration_date: Option<DateTime<Utc>>,
        abuse_email_address: Option<&str>
    ) ->  Option<Domain> {
        Some(
            Domain {
                category: DomainCategory::Other,
                name: name.into(),
                registrar: registrar.map(String::from),
                registration_date,
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "fake.net",
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                DnsServerConfig::response_200(
                    "possiblynotfake.com",
                    "Reg Two",
                    "abuse@regtwo.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()
                ),
                DnsServerConfig::response_200(
                    "morethanlikelyfake.net",
                    "Reg Three",
                    "abuse@regthree.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()
                ),
            ]
        );
    }

    async fn get_bootstrap() -> Bootstrap {
        let mut client = Client::new();

        client.set_base_bootstrap_url("http://localhost:4545");

        client.fetch_bootstrap().await.unwrap()
    }
}

pub async fn populate(bootstrap: &Bootstrap, output_data: &mut OutputData) {
    let update_from = lookup_from_rdap(bootstrap, &mut output_data.parsed_mail.sender_addresses.from);
    let update_reply_to = lookup_from_rdap(bootstrap, &mut output_data.parsed_mail.sender_addresses.reply_to);
    let update_return_path = lookup_from_rdap(bootstrap, &mut output_data.parsed_mail.sender_addresses.return_path);

    tokio::join!(update_from, update_reply_to, update_return_path);
}

#[cfg(test)]
mod lookup_from_rdap_tests {
    use super::*;
    use crate::mountebank::*;
    use test_support::*;

    #[test]
    fn populates_email_address_data_with_domain() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let mut input = input();

        tokio_test::block_on(lookup_from_rdap(&bootstrap, &mut input));

        assert_eq!(populated_output(), input);
    }

    #[test]
    fn does_not_populate_if_no_servers_available() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let mut input = input_tld_sans_server();

        tokio_test::block_on(lookup_from_rdap(&bootstrap, &mut input));

        assert_eq!(input_tld_sans_server(), input);

    }

    #[test]
    fn does_not_populate_if_server_returns_404() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_404_impostor();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let mut data = input();

        tokio_test::block_on(lookup_from_rdap(&bootstrap, &mut data));

        assert_eq!(input(), data);
    }

    #[test]
    fn does_not_lookup_with_existing_domain() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let mut input = input_with_domain();

        tokio_test::block_on(lookup_from_rdap(&bootstrap, &mut input));

        assert_eq!(input_with_domain(), input);
    }

    fn input() -> Option<EmailAddressData> {
        Some(
            EmailAddressData {
                address: "someone@fake.net".into(),
                domain: None,
            }
        )
    }

    fn input_tld_sans_server() -> Option<EmailAddressData> {
        Some(
            EmailAddressData {
                address: "someone@fake.unobtainium".into(),
                domain: None,
            }
        )
    }

    fn input_with_domain() -> Option<EmailAddressData> {
        Some(
            EmailAddressData {
                address: "someone@fake.net".into(),
                domain: domain_object(
                    "fake.net",
                    Some("Not Reg One"),
                    Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 59).unwrap()),
                    Some("abuse@notregone.zzz")
                ),
            }
        )
    }

    fn populated_output() -> Option<EmailAddressData> {
        Some(
            EmailAddressData {
                address: "someone@fake.net".into(),
                domain: domain_object(
                    "fake.net",
                    Some("Reg One"),
                    Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
                    Some("abuse@regone.zzz")
                ),
            }
        )
    }
}

async fn lookup_from_rdap(bootstrap: &Bootstrap, data: &mut Option<EmailAddressData>) {
    let client = Client::new();

    if let Some(e_a_d) = data {
        if e_a_d.domain.is_none() {
            let parts = e_a_d.address.split('@').collect::<Vec<&str>>();

            let domain_part: &str = parts.last().unwrap();

            if let Some(servers) = bootstrap.dns.find(domain_part) {
                if let Ok(response) = client.query_domain(&servers[0], domain_part).await {
                    let registrar_name = extract_registrar_name(&response.entities);
                    let abuse_email_address = extract_abuse_email(&response.entities);
                    let registration_date = extract_registration_date(&response.events);

                    e_a_d.domain = Some(
                        Domain {
                            category: DomainCategory::Other,
                            name: domain_part.into(),
                            registrar: registrar_name,
                            registration_date,
                            abuse_email_address,
                        }
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod extract_registrar_name_tests {
    use super::*;

    #[test]
    fn extracts_registrar_name() {
        let entities = entities_including_registrar();

        assert_eq!(
            Some(String::from("Reg One")),
            extract_registrar_name(&entities)
        );
    }

    #[test]
    fn no_registrar_role() {
        let entities = entities_no_registrar();

        assert_eq!(
            None,
            extract_registrar_name(&entities)
        );
    }

    #[test]
    fn registrar_role_no_full_name() {
        let entities = entities_including_registrar_no_fn();

        assert_eq!(
            None,
            extract_registrar_name(&entities)
        );
    }

    fn entities_including_registrar() -> Vec<rdap_types::Entity> {
        vec![
            build_entity(
                Some(vec![rdap_types::Role::Registrant, rdap_types::Role::Sponsor]),
                ("fn", "Not Reg One"),
            ),
            build_entity(
                Some(vec![
                    rdap_types::Role::Noc,
                    rdap_types::Role::Registrar,
                    rdap_types::Role::Sponsor
                ]),
                ("fn", "Reg One"),
            ),
            build_entity(
                Some(vec![rdap_types::Role::Noc, rdap_types::Role::Sponsor]),
                ("fn", "Not Reg Two")
            )
        ]
    }

    fn entities_no_registrar() -> Vec<rdap_types::Entity> {
        vec![
            build_entity(
                Some(vec![rdap_types::Role::Registrant, rdap_types::Role::Sponsor]),
                ("fn", "Not Reg One"),
            ),
            build_entity(
                Some(vec![
                    rdap_types::Role::Noc,
                    rdap_types::Role::Sponsor
                ]),
                ("fn", "Not Reg Two"),
            ),
            build_entity(
                Some(vec![
                    rdap_types::Role::Noc,
                    rdap_types::Role::Sponsor,
                ]),
                ("fn", "Not Reg Three")
            )
        ]
    }

    fn entities_including_registrar_no_fn() -> Vec<rdap_types::Entity> {
        vec![
            build_entity(
                Some(vec![rdap_types::Role::Registrant, rdap_types::Role::Sponsor]),
                ("fn", "Not Reg One"),
            ),
            build_entity(
                Some(vec![
                    rdap_types::Role::Noc,
                    rdap_types::Role::Registrar,
                    rdap_types::Role::Sponsor
                ]),
                ("not-fn", "Reg One"),
            ),
            build_entity(
                Some(vec![rdap_types::Role::Noc, rdap_types::Role::Sponsor]),
                ("fn", "Not Reg Two")
            )
        ]
    }

    fn build_entity(
        roles: Option<Vec<rdap_types::Role>>,
        additional_vcard_item: (&str, &str)
    ) -> rdap_types::Entity {
        let vcard_array = rdap_types::JCard(
            rdap_types::JCardType::Vcard,
            vec![
                build_jcard_item("foo", "bar"),
                build_jcard_item(additional_vcard_item.0, additional_vcard_item.1),
                build_jcard_item("baz", "biz"),
            ]
        );

        let handle: Option<String> = None;

        rdap_types::Entity {
            roles,
            vcard_array: Some(vcard_array),
            handle,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_jcard_item(property_name: &str, value: &str) -> rdap_types::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        rdap_types::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: rdap_types::JCardItemDataType::Text,
            values: vec![json!(value)]

        }
    }
}

fn extract_registrar_name(entities: &[rdap_types::Entity]) -> Option<String> {
    if let Some(entity) = find_registrar_entity(entities) {
        extract_full_name(entity)
    } else {
        None
    }
}

#[cfg(test)]
mod extract_full_name_tests {
    use super::*;

    #[test]
    fn extract_full_name_from_entity() {
        assert_eq!(
            Some(String::from("Reg One")),
            extract_full_name(&entity_with_full_name())
        );
    }

    #[test]
    fn extract_full_name_no_vcard_array() {
        assert_eq!(
            None,
            extract_full_name(&entity_without_vcards())
        );
    }

    #[test]
    fn extract_full_name_no_fn_vcard() {
        assert_eq!(
            None,
            extract_full_name(&entity_without_fn_vcard())
        );
    }

    #[test]
    fn extract_full_name_multiple_fn_vcards() {
        assert_eq!(
            Some("Reg One".into()),
            extract_full_name(&entity_with_multiple_fn_vcards())
        );
    }

    #[test]
    fn extract_full_name_multiple_fn_values() {
        assert_eq!(
            Some("Reg One".into()),
            extract_full_name(
                &build_entity(None, vec![("fn", &["Reg One", "Reg Two"])])
            )
        );
    }

    fn entity_with_full_name() -> rdap_types::Entity {
        build_entity(None, vec![("fn", &["Reg One"])])
    }

    fn build_entity(
        roles: Option<Vec<rdap_types::Role>>,
        additional_items: Vec<(&str, &[&str])>
    ) -> rdap_types::Entity {
        let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
        let mut additional_vcard_items = additional_items.iter().map(|(c_type, c_values)| {
            build_jcard_item(c_type, c_values)
        }).collect();
        let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

        vcard_items.append(&mut additional_vcard_items);
        vcard_items.append(&mut trailing_vcard_items);

        let vcard_array = rdap_types::JCard(rdap_types::JCardType::Vcard, vcard_items);

        let handle: Option<String> = None;

        rdap_types::Entity {
            roles,
            vcard_array: Some(vcard_array),
            handle,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> rdap_types::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        rdap_types::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: rdap_types::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }

    fn entity_without_vcards() -> rdap_types::Entity {
        let handle: Option<String> = None;

        rdap_types::Entity {
            roles: None,
            vcard_array: None,
            handle,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn entity_without_fn_vcard() -> rdap_types::Entity {
        build_entity(None, vec![("not-fn", &["Reg One"])])
    }

    fn entity_with_multiple_fn_vcards() -> rdap_types::Entity {
        build_entity(None, vec![("fn", &["Reg One"]), ("fn", &["Reg Two"])])
    }
}

fn extract_full_name(entity: &rdap_types::Entity) -> Option<String> {
    if let Some(vcards) = &entity.vcard_array {
        let full_name_items = vcards.items_by_name("fn");

        full_name_items
            .first()
            .map(|item| {
                item.values.first().unwrap().as_str().unwrap().into()
            })
    } else {
        None
    }
}

#[cfg(test)]
mod extract_abuse_email_tests {
    use super::*;

    #[test]
    fn returns_abuse_email() {
        let entities = &[
            non_registrar_entity(),
            registrar_entity(),
        ];

        assert_eq!(
            Some(String::from("abuse@test.zzz")),
            extract_abuse_email(entities)
        )
    }

    #[test]
    fn returns_none_if_no_registrar_entity() {
        let entities = &[
            non_registrar_entity(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn returns_none_if_registrar_has_none_entities() {
        let entities = &[
            registrar_entity_none_entities(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn returns_none_if_registrar_has_no_abuse_entity() {
        let entities = &[
            registrar_entity_no_abuse_entities(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn returns_last_abuse_entity_details_if_multiple() {
        let entities = &[
            registrar_entity_multiple_abuse_entities(),
        ];

        assert_eq!(
            Some(String::from("alsoabuse@test.zzz")),
            extract_abuse_email(entities)
        )
    }

    #[test]
    fn registrar_abuse_entity_has_none_vcards() {
        let entities = &[
            registrar_entity_abuse_none_vcards(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn registrar_abuse_entity_no_email_vcard() {
        let entities = &[
            registrar_entity_no_abuse_email(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn registrar_abuse_entity_multiple_email_values_returns_last_value() {
        let entities = &[
            registrar_entity_abuse_multiple_email_values(),
        ];

        assert_eq!(
            Some(String::from("alsoabuse@test.zzz")),
            extract_abuse_email(entities)
        )
    }

    fn registrar_entity() -> rdap_types::Entity {
        build_entity(
            &[rdap_types::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        rdap_types::Role::Administrative,
                        rdap_types::Role::Technical,
                    ],
                    None,
                    Some(&[("email", &["notabuse@test.zzz"])])
                ),
                build_entity(
                    &[
                        rdap_types::Role::Administrative,
                        rdap_types::Role::Abuse,
                        rdap_types::Role::Technical,
                    ],
                    None,
                    Some(&[("email", &["abuse@test.zzz"])])
                ),
                build_entity(
                    &[
                        rdap_types::Role::Administrative,
                        rdap_types::Role::Technical,
                    ],
                    None,
                    Some(&[("email", &["alsonotabuse@test.zzz"])])
                ),
            ]),
           None
        )
    }

    fn non_registrar_entity() -> rdap_types::Entity {
        build_entity(
            &[rdap_types::Role::Sponsor],
            Some(vec![
                build_entity(
                    &[
                        rdap_types::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &["notregabuse@test.zzz"])])
                ),
            ]),
            None
        )
    }

    fn registrar_entity_none_entities() -> rdap_types::Entity {
        build_entity(
            &[rdap_types::Role::Registrar],
            None,
            None
        )
    }

    fn registrar_entity_no_abuse_entities() -> rdap_types::Entity {
        build_entity(
            &[rdap_types::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        rdap_types::Role::Administrative,
                    ],
                    None,
                    Some(&[("email", &["notabuse@test.zzz"])])
                ),
            ]),
            None
        )
    }

    fn registrar_entity_multiple_abuse_entities() -> rdap_types::Entity {
        build_entity(
            &[rdap_types::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        rdap_types::Role::Administrative,
                        rdap_types::Role::Technical,
                    ],
                    None,
                    Some(&[("email", &["notabuse@test.zzz"])])
                ),
                build_entity(
                    &[
                        rdap_types::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &["abuse@test.zzz"])])
                ),
                build_entity(
                    &[
                        rdap_types::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &["alsoabuse@test.zzz"])]),
                ),
            ]),
            None
        )
    }

    fn registrar_entity_abuse_none_vcards() -> rdap_types::Entity {
        build_entity(
            &[rdap_types::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        rdap_types::Role::Abuse,
                    ],
                    None,
                    None
                ),
            ]),
            None
        )
    }

    fn registrar_entity_no_abuse_email() -> rdap_types::Entity {
        build_entity(
            &[rdap_types::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        rdap_types::Role::Abuse,
                    ],
                    None,
                    Some(&[])
                ),
            ]),
            None
        )
    }

    fn registrar_entity_abuse_multiple_email_values() -> rdap_types::Entity {
        build_entity(
            &[rdap_types::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        rdap_types::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &["abuse@test.zzz", "alsoabuse@test.zzz"])])
                ),
            ]),
            None
        )
    }
    fn build_entity(
        roles: &[rdap_types::Role],
        entities: Option<Vec<rdap_types::Entity>>,
        additional_items: Option<&[(&str, &[&str])]>
    ) -> rdap_types::Entity {
        let vcard_array = if let Some(additional) = additional_items {
            let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
            let mut additional_vcard_items = additional.iter().map(|(c_type, c_values)| {
                build_jcard_item(c_type, c_values)
            }).collect();
            let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

            vcard_items.append(&mut additional_vcard_items);
            vcard_items.append(&mut trailing_vcard_items);

            Some(rdap_types::JCard(rdap_types::JCardType::Vcard, vcard_items))
        } else {
            None
        };

        rdap_types::Entity {
            roles: Some(roles.to_vec()),
            vcard_array,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> rdap_types::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        rdap_types::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: rdap_types::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }
}

fn extract_abuse_email(entities: &[rdap_types::Entity]) -> Option<String> {
    if let Some(registrar_entity) = find_registrar_entity(entities) {
        if let Some(r_entities) = &registrar_entity.entities {
            let abuse_entities: Vec<&rdap_types::Entity> = r_entities
                .iter()
                .filter(|e| {
                    if let Some(roles) = &e.roles {
                        roles.contains(&rdap_types::Role::Abuse)
                    } else {
                        false
                    }
                })
            .collect();

            if let Some(abuse_entity) = abuse_entities.last() {
                if let Some(vcards) = &abuse_entity.vcard_array {
                    vcards
                        .items_by_name("email")
                        .last()
                        .map(stringify_last_value)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn stringify_last_value(item: &&rdap_types::JCardItem) -> String {
    item
        .values
        .last()
        .unwrap()
        .as_str()
        .unwrap()
        .into()
}

#[cfg(test)]
mod extract_registration_date_tests {
    use super::*;

    #[test]
    fn returns_registration_date() {
        let events = rdap_types::Events(vec![
            non_registration_event(), registration_event(), non_registration_event()
        ]);
        let expected_registration_date = chrono::Utc
            .with_ymd_and_hms(2022, 12, 11, 12, 5, 30)
            .single()
            .unwrap();

        assert_eq!(
            Some(expected_registration_date),
            extract_registration_date(&events)
        );
    }

    #[test]
    fn returns_none_if_no_registration_event() {
        let events = rdap_types::Events(vec![
            non_registration_event(), non_registration_event()
        ]);

        assert!(extract_registration_date(&events).is_none());
    }

    fn registration_event() -> rdap_types::Event {
        let event_date = time_zone()
            .with_ymd_and_hms(2022, 12, 11, 14, 5, 30)
            .single()
            .unwrap();

        rdap_types::Event {
            event_actor: None,
            event_action: rdap_types::EventAction::Registration,
            event_date,
            links: None,
        }
    }

    fn non_registration_event() -> rdap_types::Event {

        let event_date = time_zone()
            .with_ymd_and_hms(2022, 12, 25, 16, 5, 30)
            .single()
            .unwrap();

        rdap_types::Event {
            event_actor: None,
            event_action: rdap_types::EventAction::Locked,
            event_date,
            links: None,
        }
    }

    fn time_zone() -> chrono::FixedOffset {
        chrono::FixedOffset::east_opt(2 * 3600).unwrap()
    }
}

fn extract_registration_date(events: &rdap_types::Events) -> Option<DateTime<Utc>> {
    events
        .action_date(rdap_types::EventAction::Registration)
        .map(|date| date.into())
}

#[cfg(test)]
mod find_registrar_entity_tests {
    use super::*;

    #[test]
    fn extracts_registrar_name() {
        let entities = vec![
            non_registrar_entity(),
            registrar_entity_1(),
            non_registrar_entity(),
        ];

        compare(&registrar_entity_1(), find_registrar_entity(&entities).unwrap());
    }

    #[test]
    fn multiple_registrar_entries_returns_last_registrar() {
        let entities = vec![
            non_registrar_entity(),
            registrar_entity_1(),
            registrar_entity_2(),
            non_registrar_entity(),
        ];

        compare(&registrar_entity_2(), find_registrar_entity(&entities).unwrap());
    }

    #[test]
    fn no_registrar_role() {
        let entities = vec![
            non_registrar_entity(),
            non_registrar_entity(),
        ];

        assert!(find_registrar_entity(&entities).is_none());
    }

    fn non_registrar_entity() -> rdap_types::Entity {
        build_entity(&[
            rdap_types::Role::Noc,
            rdap_types::Role::Sponsor
        ])
    }

    fn registrar_entity_1() -> rdap_types::Entity {
        build_entity(&[
            rdap_types::Role::Noc,
            rdap_types::Role::Registrar,
            rdap_types::Role::Sponsor
        ])
    }

    fn registrar_entity_2() -> rdap_types::Entity {
        build_entity(&[
            rdap_types::Role::Noc,
            rdap_types::Role::Registrar,
        ])
    }

    fn build_entity(
        roles: &[rdap_types::Role],
    ) -> rdap_types::Entity {
        rdap_types::Entity {
            roles: Some(roles.to_vec()),
            vcard_array: None,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn compare(expected: &rdap_types::Entity, actual: &rdap_types::Entity) {
        // Use the assigned roles as a unique 'id'
        assert_eq!(expected.roles, actual.roles);
    }
}

fn find_registrar_entity(entities: &[rdap_types::Entity]) -> Option<&rdap_types::Entity> {
    let mut registrar_entities: Vec<&rdap_types::Entity> = entities
        .iter()
        .filter(|e| {
            if let Some(roles) = &e.roles {
                roles.contains(&rdap_types::Role::Registrar)
            } else {
                false
            }
        })
    .collect();

    registrar_entities.pop()
}

#[cfg(test)]
mod test_support {
    use super::*;

    use crate::mountebank::{
        setup_dns_server,
        DnsServerConfig,
    };

    pub fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "fake.net",
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                DnsServerConfig::response_200(
                    "possiblynotfake.com",
                    "Reg Two",
                    "abuse@regtwo.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()
                ),
                DnsServerConfig::response_200(
                    "morethanlikelyfake.net",
                    "Reg Three",
                    "abuse@regthree.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()
                ),
            ]
        );
    }

    pub fn setup_404_impostor() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_404("fake.net"),
            ]
        );
    }

    pub async fn get_bootstrap() -> Bootstrap {
        let mut client = Client::new();

        client.set_base_bootstrap_url("http://localhost:4545");

        client.fetch_bootstrap().await.unwrap()
    }

   pub  fn domain_object(
        name: &str,
        registrar: Option<&str>,
        registration_date: Option<DateTime<Utc>>,
        abuse_email_address: Option<&str>
    ) ->  Option<Domain> {
        Some(
            Domain {
                category: DomainCategory::Other,
                name: name.into(),
                registrar: registrar.map(String::from),
                registration_date,
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }
}
