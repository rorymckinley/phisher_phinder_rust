use chrono::prelude::*;
use reqwest::blocking::Client;
use serde_json::json;

pub fn setup_bootstrap_server() {
    use reqwest::header::{HeaderMap, CONTENT_TYPE};

    let stub_data = Mountebank {
        port: 4545,
        protocol: "http".into(),
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
        stubs: stub_configs.iter().map(|config| {
            create_dns_service_stub(
                config.domain_name, config.registrar, config.abuse_email, config.registration_date
            )
        }).collect(),
    };

    upload_stub(stub_data);
}

pub struct DnsServerConfig<'a> {
    pub domain_name: &'a str,
    pub registrar: &'a str,
    pub abuse_email: &'a str,
    pub registration_date: DateTime<Utc>,
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
        "version": "1.0",
    });

    create_stub("/dns.json", body.to_string())
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
            ],
        ],
        "version": "1.0",
    });

    create_stub("/asn.json", body.to_string())

}

fn create_ip_v4_bootstrap_stub() -> MountebankStub {
    let body = json!({
        "publication": "2022-11-01T22:00:01Z",
        "services": [
            [
                [
                    "41.0.0.0/8",
                    "102.0.0.0/8",
                ],
                [
                    "https://rdap.afrinic.net/rdap/",
                    "http://rdap.afrinic.net/rdap/"
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
        "version": "1.0",
    });

    create_stub("/ipv4.json", body.to_string())
}

fn create_ip_v6_bootstrap_stub() -> MountebankStub {
    let body = json!({
        "publication": "2022-11-01T22:00:01Z",
        "services": [
            [
                [
                    "2001:4200::/23",
                    "2c00::/12"
                ],
                [
                    "https://rdap.afrinic.net/rdap/",
                    "http://rdap.afrinic.net/rdap/"
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
        "version": "1.0",
    });

    create_stub("/ipv6.json", body.to_string())
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
        "version": "1.0",
    });

    create_stub("/object-tags.json", body.to_string())
}

fn create_dns_service_stub(
    domain_name: &str,
    registrar: &str,
    abuse_email: &str,
    registration_date: DateTime<Utc>
) -> MountebankStub {
    let body = rdap::DomainResponse::new(
        domain_name, registrar, abuse_email, registration_date
    );

    create_stub(&format!("/domain/{}", domain_name), serde_json::to_string(&body).unwrap())
}

fn create_stub(path: &str, body: String) -> MountebankStub {
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
                        status_code: 200,
                        headers: MountebankHeaders { content_type: "application/json".into() },
                        body: Some(body)
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

use serde::Serialize;

#[derive(Serialize)]
struct Mountebank {
    port: u16,
    protocol: String,
    stubs: Vec<MountebankStub>
}

#[derive(Serialize)]
struct MountebankStub {
    predicates: Vec<MountebankPredicate>,
    responses: Vec<MountebankResponse>
}

#[derive(Serialize)]
struct MountebankPredicate {
    equals: Option<MountebankEquals>
}

#[derive(Serialize)]
struct MountebankResponse {
    is: Option<MountebankIs>
}

#[derive(Serialize)]
struct MountebankEquals {
    path: Option<String>
}

#[derive(Serialize)]
struct MountebankIs {
    #[serde(rename = "statusCode")]
    status_code: u16,
    headers: MountebankHeaders,
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
            registrar: &str,
            abuse_email: &str,
            registration_date: DateTime<Utc>
        ) -> Self {
            Self {
                object_class_name: "domain".into(),
                handle: "DOM-XXX".into(),
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
                    Self::abuse(abuse_email)
                ])
            }
        }

        fn abuse(abuse_email: &str) -> Self {
            Self {
                object_class_name: "entity".into(),
                handle: None,
                roles: vec!["abuse".into()],
                public_ids: None,
                vcard_array: (
                    "vcard".into(),
                    vec![
                        VcardProperty::version(),
                        VcardProperty::full_name(""),
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
