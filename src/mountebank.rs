use chrono::prelude::*;
use reqwest::blocking::Client;
use serde_json::json;
use std::collections::HashMap;

pub fn setup_bootstrap_server() {
    use reqwest::header::{HeaderMap, CONTENT_TYPE};

    let stub_data = Mountebank {
        port: 4545,
        protocol: "http".into(),
        record_requests: false,
        requests: vec![],
        stubs: vec![
            create_asn_bootstrap_stub(),
            create_dns_bootstrap_stub(),
            create_ip_v4_bootstrap_stub(),
            create_ip_v6_bootstrap_stub(),
            create_object_tags_bootstrap_stub()
        ],
    };

    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let client = Client::new();

    client.post("http://localhost:2525/imposters")
        .headers(headers)
        .body(serde_json::to_string(&stub_data).unwrap())
        .send()
        .unwrap();
}

pub fn setup_dns_server(stub_configs: Vec<DnsServerConfig>) {
    let stub_data = Mountebank {
        port: 4546,
        protocol: "http".into(),
        record_requests: false,
        requests: vec![],
        stubs: stub_configs.iter().map(|config| {
            create_dns_service_stub(config)
        }).collect(),
    };

    upload_stub(stub_data);
}

pub fn setup_ip_v4_server(stub_configs: Vec<IpServerConfig>) {
    let stub_data = Mountebank {
        port: 4547,
        protocol: "http".into(),
        record_requests: false,
        requests: vec![],
        stubs: stub_configs.iter().map(|config| {
            create_ip_service_stub(config)
        }).collect(),
    };

    upload_stub(stub_data);
}

pub fn setup_ip_v6_server(stub_configs: Vec<IpServerConfig>) {
    let stub_data = Mountebank {
        port: 4548,
        protocol: "http".into(),
        record_requests: false,
        requests: vec![],
        stubs: stub_configs.iter().map(|config| {
            create_ip_service_stub(config)
        }).collect(),
    };

    upload_stub(stub_data);
}

pub struct DnsServerConfig<'a> {
    pub domain_name: &'a str,
    pub handle: Option<&'a str>,
    pub registrar: Option<&'a str>,
    pub abuse_email: Option<&'a str>,
    pub registration_date: Option<DateTime<Utc>>,
    pub response_code: u16,
}

impl<'a> DnsServerConfig<'a> {
    pub fn response_200(
        domain_name: &'a str,
        handle: Option<&'a str>,
        registrar: &'a str,
        abuse_email: &'a str,
        registration_date: DateTime<Utc>
    ) -> Self {
        Self {
            domain_name,
            handle,
            registrar: Some(registrar),
            abuse_email: Some(abuse_email),
            registration_date: Some(registration_date),
            response_code: 200
        }
    }

    pub fn response_404(domain_name: &'a str) -> Self {
        Self {
            domain_name,
            handle: None,
            registrar: None,
            abuse_email: None,
            registration_date: None,
            response_code: 404
        }
    }
}

pub struct IpServerConfig<'a> {
    ip_address: &'a str,
    handle: Option<&'a str>,
    start_address: Option<&'a str>,
    end_address: Option<&'a str>,
    entity_configs: Option<Vec<EntityConfig<'a>>>,
    response_code: u16,
}

impl<'a> IpServerConfig<'a> {
    pub fn response_200(
        ip_address: &'a str,
        handle: Option<&'a str>,
        address_range: (&'a str, &'a str),
        entities_option: Option<&[(&'a str, &'a str, &'a str)]>
    ) -> Self {
        let (start_address, end_address) = address_range;

        let entity_configs = entities_option
            .map(|entities| {
                entities
                    .iter()
                    .map(|(name, role, abuse_email)| EntityConfig {name ,role, abuse_email})
                    .collect()
            });

        Self {
            ip_address,
            handle,
            start_address: Some(start_address),
            end_address: Some(end_address),
            entity_configs,
            response_code: 200
        }
    }

    pub fn response_404(ip_address: &'a str) -> Self {
        Self {
            ip_address,
            handle: None,
            start_address: None,
            end_address: None,
            entity_configs: Some(vec![]),
            response_code: 404
        }
    }
}

pub struct EntityConfig<'a> {
    name: &'a str,
    role: &'a str,
    abuse_email: &'a str,
}

pub fn setup_head_impostor(port: u16, redirect: bool, location: Option<&str>) {
    let headers = location.map(|loc_str| {
        HashMap::from([
            (String::from("Location"), String::from(loc_str))
        ])
    });
    let response_code = if redirect {
        301
    } else {
        200
    };

    let stub_data = Mountebank {
        port,
        protocol: "http".into(),
        record_requests: false,
        requests: vec![],
        stubs: vec![
            create_stub("/", None, response_code, headers)
        ]
    };

    upload_stub(stub_data);
}

pub fn lookup_impostor(port: u16) -> Mountebank {
    let client = Client::new();

    let response = client.get(format!("http://localhost:2525/imposters/{port}"))
        .send()
        .unwrap();

    serde_json::from_str(&response.text().unwrap()).unwrap()
}

fn upload_stub(stub: Mountebank) {
    use reqwest::header::{HeaderMap, CONTENT_TYPE};

    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let client = Client::new();

    client.post("http://localhost:2525/imposters")
        .headers(headers)
        .body(serde_json::to_string(&stub).unwrap())
        .send()
        .unwrap();
}

fn create_dns_bootstrap_stub() -> MountebankStub {
    let body = json!({
        "publication": "2022-11-01T22:00:01Z",
        "services": [
            [
                [
                    "com"
                ],
                [
                    "http://localhost:4546/"
                ]
            ],
            [
                [
                    "net"
                ],
                [
                    "http://localhost:4546/"
                ]
            ],
        ],
        "version": "1.0"
    });

    create_stub(
        "/dns.json",
       Some(body.to_string()),
       200,
       None
    )
}

fn create_asn_bootstrap_stub() -> MountebankStub {
    let body = json!({
        "publication": "2022-11-01T22:00:01Z",
        "services": [
            [
                [
                    "36864-37887",
                    "327680-328703",
                    "328704-329727"
                ],
                [
                    "https://rdap.afrinic.net/rdap/",
                    "http://rdap.afrinic.net/rdap/"
                ]
            ],
            [
                [
                    "149818-150841",
                    "150842-151865"
                ],
                [
                    "https://rdap.apnic.net/"
                ]
            ]
        ],
        "version": "1.0"
    });

    create_stub(
        "/asn.json",
        Some(body.to_string()),
        200,
        None
    )

}

fn create_ip_v4_bootstrap_stub() -> MountebankStub {
    let body = json!({
        "publication": "2022-11-01T22:00:01Z",
        "services": [
            [
                [
                    "10.0.0.0/8",
                    "192.168.0.0/16"
                ],
                [
                    "http://localhost:4547/"
                ]
            ],
            [
                [
                    "30.0.0.0/8",
                ],
                [
                    // Deliberately no servers to test this use case
                ]
            ],
            [
                [
                    "221.0.0.0/8",
                    "222.0.0.0/8",
                    "223.0.0.0/8"
                ],
                [
                    "https://rdap.apnic.net/"
                ]
            ],
        ],
        "version": "1.0"
    });

    create_stub(
        "/ipv4.json",
        Some(body.to_string()),
        200,
        None
    )
}

fn create_ip_v6_bootstrap_stub() -> MountebankStub {
    let body = json!({
        "publication": "2022-11-01T22:00:01Z",
        "services": [
            [
                [
                    "abcd::/16",
                    "bcde::/16",
                ],
                [
                    "http://localhost:4548/"
                ]
            ],
            [
                [
                    "2001:200::/23",
                    "2001:4400::/23",
                    "2001:8000::/19",
                    "2001:a000::/20",
                    "2001:b000::/20",
                    "2001:c00::/23",
                    "2001:e00::/23",
                    "2400::/12"
                ],
                [
                    "https://rdap.apnic.net/"
                ]
            ],
        ],
        "version": "1.0"
    });

    create_stub(
        "/ipv6.json",
        Some(body.to_string()),
        200,
        None
    )
}

fn create_object_tags_bootstrap_stub() -> MountebankStub {
    let body = json!({
        "publication": "2022-11-01T22:00:01Z",
        "services": [
            [
                [
                    "andy@arin.net"
                ],
                [
                    "ARIN"
                ],
                [
                    "https://rdap.arin.net/registry/",
                    "http://rdap.arin.net/registry/"
                ]
            ],
            [
                [
                    "carlos@lacnic.net"
                ],
                [
                    "LACNIC"
                ],
                [
                    "https://rdap.lacnic.net/rdap/"
                ]
            ],
        ],
        "version": "1.0"
    });

    create_stub(
        "/object-tags.json",
        Some(body.to_string()),
        200,
        None
    )
}

fn create_dns_service_stub(config: &DnsServerConfig) -> MountebankStub {
    let body = if config.response_code == 200 {
        let response = rdap::DomainResponse::new(
            config.domain_name,
            config.handle,
            config.registrar.unwrap(),
            config.abuse_email.unwrap(),
            config.registration_date.unwrap()
        );
        serde_json::to_string(&response).ok()
    } else {
        None
    };

    create_stub(&format!("/domain/{}", config.domain_name), body, config.response_code, None)
}

fn create_ip_service_stub(config: &IpServerConfig) -> MountebankStub {
    let body = if config.response_code == 200 {
        let response = rdap::IpResponse::new(
            config.handle,
            config.start_address.unwrap(),
            config.end_address.unwrap(),
            config.entity_configs.as_ref().map(|configs| configs)
        );
        serde_json::to_string(&response).ok()
    } else {
        None
    };

    create_stub(&format!("/ip/{}", config.ip_address), body, config.response_code, None)
}

fn create_stub(
    path: &str,
    wrapped_body: Option<String>,
    status_code: u16,
    optional_headers: Option<HashMap<String, String>>,
) -> MountebankStub {
    let headers = optional_headers.unwrap_or_else(|| {
        HashMap::from([
            (String::from("Content-Type"), String::from("application/json"))
        ])
    });
    MountebankStub {
        predicates: vec![
            MountebankPredicate {
                equals:  Some(MountebankEquals { path: Some(path.into()) })
            }
        ],
        responses: vec![
            MountebankResponse {
                is: Some(
                    MountebankIs {
                        status_code,
                        headers,
                        body: wrapped_body
                    }
                )
            }
        ],
    }
}

pub fn clear_all_impostors() {
    let client = Client::new();

    client.delete("http://localhost:2525/imposters")
        .send()
        .unwrap();
}

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all(deserialize="snake_case", serialize="camelCase"))]
pub struct Mountebank {
    port: u16,
    protocol: String,
    #[serde(alias="recordRequests")]
    record_requests: bool,
    pub requests: Vec<MountebankRequest>,
    stubs: Vec<MountebankStub>
}

#[derive(Deserialize, Serialize)]
pub struct MountebankRequest {
    from: MountebankEmailAddress,
    pub subject: String,
    pub to: Vec<MountebankEmailAddress>,
}

#[derive(Deserialize, Serialize)]
pub struct MountebankEmailAddress {
    pub address: String,
    name: String
}

#[derive(Debug, Deserialize, Serialize)]
struct MountebankStub {
    predicates: Vec<MountebankPredicate>,
    responses: Vec<MountebankResponse>
}

#[derive(Debug, Deserialize, Serialize)]
struct MountebankPredicate {
    equals: Option<MountebankEquals>
}

#[derive(Debug, Deserialize, Serialize)]
struct MountebankResponse {
    is: Option<MountebankIs>
}

#[derive(Debug, Deserialize, Serialize)]
struct MountebankEquals {
    path: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
struct MountebankIs {
    #[serde(rename = "statusCode")]
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<String>
}

#[derive(Serialize)]
struct MountebankHeaders {
    #[serde(rename = "Content-Type")]
    content_type: String
}

mod rdap {
    use serde::Serialize;
    use chrono::{DateTime, Utc};
    use super::EntityConfig;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DomainResponse {
        object_class_name: String,
        handle: String,
        ldh_name: String,
        links: Vec<Link>,
        status: Vec<String>,
        entities: Vec<Entity>,
        events: Vec<Event>,
        secure_dns: SecureDns,
        nameservers: Vec<Nameserver>,
        rdap_conformance: Vec<String>,
        notices: Vec<Notice>
    }

    impl DomainResponse {
        pub fn new(
            domain_name: &str,
            handle_option: Option<&str>,
            registrar: &str,
            abuse_email: &str,
            registration_date: DateTime<Utc>
        ) -> Self {
            let handle = handle_option.unwrap_or("DOM-XXX").into();

            Self {
                object_class_name: "domain".into(),
                handle,
                ldh_name: String::from(domain_name).to_uppercase(),
                links: vec![],
                status: Self::status(),
                entities: vec![
                    Entity::registrar(registrar, abuse_email)
                ],
                events: vec![
                    Event::registration(registration_date)
                ],
                secure_dns: SecureDns{delegation_signed: false},
                nameservers: vec![],
                rdap_conformance: vec![],
                notices: vec![]
            }
        }

        fn status() -> Vec<String> {
            vec![
                "client transfer prohibited".into(),
                "server delete prohibited".into(),
                "server transfer prohibited".into(),
                "server update prohibited".into()
            ]
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct IpResponse {
        handle: String,
        ip_version: String,
        object_class_name: String,
        start_address: String,
        end_address: String,
        entities: Option<Vec<Entity>>
    }

    impl IpResponse {
        pub fn new(
            handle: Option<&str>,
            start_address: &str,
            end_address: &str,
            entity_configs_option: Option<&Vec<EntityConfig>>,
        ) -> Self {
            let entities = entity_configs_option
                .map(|entity_configs| {
                    entity_configs
                        .iter()
                        .map(|EntityConfig {name, role, abuse_email}| {
                            match *role {
                                "abuse" =>  Entity::abuse(name, abuse_email),
                                "registrant" =>  Entity::registrant(name, abuse_email),
                                _ => panic!("Unexpected entity role")
                            }
                        })
                        .collect()
                });

            Self {
                handle: handle.unwrap_or("NET-XXX").into(),
                ip_version: "v4".into(),
                object_class_name: "ip network".into(),
                start_address: start_address.into(),
                end_address: end_address.into(),
                entities
            }
        }
    }

    #[derive(Serialize)]
    struct Link {
        value: String,
        rel: String,
        href: String,
        #[serde(rename(serialize = "type"))]
        link_type: String,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Entity {
        object_class_name:  String,
        #[serde(skip_serializing_if = "Option::is_none")]
        handle: Option<String>,
        roles: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        public_ids: Option<Vec<PublicId>>,
        vcard_array: (String, Vec<VcardProperty>),
        #[serde(skip_serializing_if = "Option::is_none")]
        entities: Option<Vec<Entity>>,
    }

    impl Entity {
        fn registrant(registrant_name: &str, abuse_email: &str) -> Self {
            Self {
                object_class_name: "entity".into(),
                handle: Some("000".into()),
                roles: vec!["registrant".into()],
                public_ids: Some(vec![PublicId::registrar("000")]),
                vcard_array: (
                    "vcard".into(),
                    vec![
                        VcardProperty(
                            "version".into(),
                            VcardPropertyParameters { property_type: None },
                            "text".into(),
                            "4.0".into()
                        ),
                        VcardProperty(
                            "fn".into(),
                            VcardPropertyParameters { property_type: None },
                            "text".into(),
                            registrant_name.into()
                        ),
                    ]
                ),
                entities: Some(vec![
                    Self::abuse("", abuse_email)
                ])
            }
        }
        // TODO Generalise with registrant
        fn registrar(registrar_name: &str, abuse_email: &str) -> Self {
            Self {
                object_class_name: "entity".into(),
                handle: Some("000".into()),
                roles: vec!["registrar".into()],
                public_ids: Some(vec![PublicId::registrar("000")]),
                vcard_array: (
                    "vcard".into(),
                    vec![
                        VcardProperty(
                            "version".into(),
                            VcardPropertyParameters { property_type: None },
                            "text".into(),
                            "4.0".into()
                        ),
                        VcardProperty(
                            "fn".into(),
                            VcardPropertyParameters { property_type: None },
                            "text".into(),
                            registrar_name.into()
                        ),
                    ]
                ),
                entities: Some(vec![
                    Self::abuse("", abuse_email)
                ])
            }
        }

        fn abuse(name: &str, abuse_email: &str) -> Self {
            Self {
                object_class_name: "entity".into(),
                handle: None,
                roles: vec!["abuse".into()],
                public_ids: None,
                vcard_array: (
                    "vcard".into(),
                    vec![
                        VcardProperty::version(),
                        VcardProperty::full_name(name),
                        VcardProperty::telephone(),
                        VcardProperty::email(abuse_email)
                    ]
                ),
                entities: None
            }
        }
    }

    #[derive(Serialize)]
    struct PublicId {
        #[serde(rename(serialize = "type"))]
        id_type: String,
        identifier: String,
    }

    impl PublicId {
        fn registrar(identifier: &str) -> Self {
            Self {
                id_type: "IANA Registrar ID".into(),
                identifier: identifier.into(),
            }
        }
    }

    #[derive(Serialize)]
    struct VcardProperty(
        String,
        VcardPropertyParameters,
        String,
        String,
    );

    impl VcardProperty {
        fn version() -> Self {
            Self(
                "version".into(),
                VcardPropertyParameters::empty(),
                "text".into(),
                "4.0".into()
            )
        }
        fn full_name(name: &str) -> Self {
            Self(
                "fn".into(),
                VcardPropertyParameters::empty(),
                "text".into(),
                name.into()
            )
        }
        fn telephone() -> Self {
            Self(
                "tel".into(),
                VcardPropertyParameters::voice(),
                "uri".into(),
                "tel:1234567890".into()
            )
        }
        fn email(email_address: &str) -> Self {
            Self(
                "email".into(),
                VcardPropertyParameters::empty(),
                "text".into(),
                email_address.into()
            )
        }
    }

    #[derive(Serialize)]
    struct VcardPropertyParameters {
        #[serde(skip_serializing_if = "Option::is_none", rename(serialize = "type"))]
        property_type: Option<String>
    }

    impl VcardPropertyParameters {
        fn empty() -> Self {
            Self { property_type: None }
        }

        fn voice() -> Self {
            Self { property_type: Some("voice".into()) }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Event {
        event_action: String,
        event_date: DateTime<Utc>,
    }

    impl Event {
        fn registration(event_date: DateTime<Utc>) -> Self {
            Self {
                event_action: "registration".into(),
                event_date
            }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SecureDns {
        delegation_signed: bool,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Nameserver {
        object_class_name: String,
        ldh_name: String,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Notice {
        title: String,
        description: Vec<String>,
        links: Vec<Link>
    }
}

