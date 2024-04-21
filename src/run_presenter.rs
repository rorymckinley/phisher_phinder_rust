use crate::result::AppResult;
use crate::run::Run;
use crate::service_configuration::Configuration;
use crate::ui::{
    display_authentication_results,
    display_metadata,
    display_abuse_notifications,
    display_reportable_entities,
    display_sender_addresses_extended
};

pub fn present<T>(run: Run, config: &T) -> AppResult<String>
where T: Configuration {
    // TODO Large overlap with ui::display_run - resolve
    Ok(
        [
            display_metadata(&run)?,
            display_sender_addresses_extended(&run.data.parsed_mail.email_addresses)?,
            display_authentication_results(&run.data)?,
            display_reportable_entities(&run)?,
            display_abuse_notifications(&run, config)?
        ]
        .join("\n")
    )
}

#[cfg(test)]
mod present_tests {
    use crate::cli::{ProcessArgs, SingleCli, SingleCliCommands};
    use crate::data::{
        DeliveryNode,
        Domain,
        DomainCategory,
        EmailAddresses,
        FulfillmentNodesContainer,
        HostNode,
        InfrastructureProvider,
        OutputData,
        ParsedMail,
        Registrar,
        ReportableEntities,
    };
    use crate::message_source::MessageSource;
    use crate::service_configuration::ServiceConfiguration;
    use super::*;

    use chrono::prelude::*;

    #[test]
    fn returns_string_including_sender_addresses() {
        let output = present(build_run(), &build_config()).unwrap();

        assert!(output.contains("Address Source"))
    }

    #[test]
    fn returns_string_containing_authentication_results() {
        let output = present(build_run(), &build_config()).unwrap();

        assert!(output.contains("DKIM"))
    }

    #[test]
    fn returns_string_containing_reportable_entities() {
        let output = present(build_run(), &build_config()).unwrap();

        assert!(output.contains("Delivery Nodes"))
    }

    #[test]
    fn returns_string_containing_run_metadata() {
        let output = present(build_run(), &build_config()).unwrap();

        assert!(output.contains("Run ID"))
    }

    #[test]
    fn returns_string_containing_notification_emails() {
        let output = present(build_run(), &build_config()).unwrap();

        assert!(output.contains("Abuse Notifications"))
    }

    fn build_run() -> Run {
        let reportable_entities = ReportableEntities {
            delivery_nodes: vec![
                build_delivery_node(1),
            ],
            email_addresses: EmailAddresses {
                from: vec![],
                links: vec![],
                reply_to: vec![],
                return_path: vec![],
            },
            fulfillment_nodes_container: FulfillmentNodesContainer {
                duplicates_removed: false,
                nodes: vec![],
            }
        };

        Run {
            id: 99099,
            created_at: Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, 30).unwrap(),
            data: OutputData {
                message_source: MessageSource::new(""),
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
                notifications: vec![],
                reportable_entities: Some(reportable_entities),
                run_id: None,
            },
            message_source: MessageSource {
                data: "".into(),
                id: Some(77177)
            }
        }
    }

    fn build_delivery_node(position: usize) -> DeliveryNode {
        let time = Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, position.try_into().unwrap()).unwrap();

        DeliveryNode {
            advertised_sender: Some(build_host_node("advertised", position)),
            observed_sender: Some(build_host_node("observed", position)),
            position,
            recipient: Some(format!("recipient.{}.test.com", position)),
            time: Some(time),
            trusted: false
        }
    }

    fn build_host_node(sender_type: &str, position: usize) -> HostNode {
        HostNode {
            domain: Some(build_domain(sender_type, position)),
            host: Some(build_host(sender_type, position)),
            infrastructure_provider: Some(build_infrastructure_provider(sender_type, position)),
            ip_address: Some(build_ip_address(sender_type, position)),
            registrar: Some(build_registrar(sender_type, position)),
        }
    }

    fn build_host(sender_type: &str, position: usize) -> String {
        format!("{position}.{sender_type}.host.com")
    }

    fn build_domain(sender_type: &str, position: usize) -> Domain {
        let registration_date = Utc
            .with_ymd_and_hms(2023, 6, 1, 10, 10, position.try_into().unwrap())
            .unwrap();

        Domain {
            abuse_email_address: Some(format!("d.{sender_type}.{position}@test.com")),
            category: DomainCategory::Other,
            name: format!("d.{sender_type}.{position}.com"),
            registration_date: Some(registration_date),
            resolved_domain: None,
        }
    }

    fn build_infrastructure_provider(sender_type: &str, position: usize) -> InfrastructureProvider {
        InfrastructureProvider {
            abuse_email_address: Some(format!("i.{sender_type}.{position}@test.com")),
            name: Some(format!("Provider {sender_type} {position}")),
        }
    }

    fn build_ip_address(sender_type: &str, position: usize) -> String {
        if sender_type == "advertised" {
            format!("10.10.10.{position}")
        } else {
            format!("20.20.20.{position}")
        }
    }

    fn build_registrar(sender_type: &str, position: usize) -> Registrar {
        Registrar {
            abuse_email_address: Some(format!("r.{sender_type}.{position}@test.com")),
            name: Some(format!("Registrar {sender_type} {position}")),
        }
    }

    fn build_config<'a>() -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            Some(""),
            &cli(),
            env_var_iterator()
        ).unwrap()
    }

    fn env_var_iterator() -> Box<dyn Iterator<Item = (String, String)>>
    {
        let v: Vec<(String, String)> = vec![
            ("PP_ABUSE_NOTIFICATIONS_FROM_ADDRESS".into(), "sender@phishereagle.com".into()),
            ("PP_DB_PATH".into(), "does.not.matter.sqlite".into()),
            ("PP_TRUSTED_RECIPIENT".into(), "does.not.matter".into()),
            ("RDAP_BOOTSTRAP_HOST".into(), "does.not.matter".into())
        ];

        Box::new(v.into_iter())
    }

    pub fn cli() -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Process(ProcessArgs {
                reprocess_run: None,
            })
        }
    }
}
