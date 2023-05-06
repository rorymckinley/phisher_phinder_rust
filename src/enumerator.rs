use crate::data::{FulfillmentNode, OutputData, ParsedMail};

#[cfg(test)]
mod enumerate_tests {
    use super::*;
    use crate::authentication_results::{AuthenticationResults, Dkim, DkimResult, Spf, SpfResult};
    use crate::data::{EmailAddresses, FulfillmentNode, Node, OutputData};
    use crate::mountebank::*;

    #[test]
    fn enumerates_each_visible_url() {
        clear_all_impostors();
        setup_head_impostor(4545, true, Some("https://re.direct.one"));
        setup_head_impostor(4546, true, Some("https://re.direct.two"));

        let input_data = input();

        let actual = tokio_test::block_on(enumerate(input_data));

        assert_eq!(sorted(output()), sorted(actual));
    }

    fn input() -> OutputData {
        OutputData::new(
            ParsedMail::new(
                authentication_results(),
                vec![],
                email_addresses(),
                vec![
                    FulfillmentNode::new("http://localhost:4545"),
                    FulfillmentNode::new("http://localhost:4546"),
                ],
                None,
            ),
            "raw mail text"
        )
    }

    fn output() -> OutputData {
        let f_nodes = vec![
            FulfillmentNode {
                hidden: Some(Node::new("https://re.direct.one")),
                ..FulfillmentNode::new("http://localhost:4545")
            },
            FulfillmentNode {
                hidden: Some(Node::new("https://re.direct.two")),
                ..FulfillmentNode::new("http://localhost:4546")
            },
        ];

        OutputData::new(
            ParsedMail::new(
                authentication_results(),
                vec![],
                email_addresses(),
                f_nodes,
                None
            ),
            "raw mail text"
        )
    }

    fn email_addresses() -> EmailAddresses {
        EmailAddresses {
            from: vec![],
            links: vec![],
            reply_to: vec![],
            return_path: vec![],
        }
    }

    fn authentication_results() -> Option<AuthenticationResults> {
        Some(
            AuthenticationResults {
                dkim: Some(Dkim {
                    result: Some(DkimResult::Fail),
                    selector: Some("".into()),
                    signature_snippet: Some("".into()),
                    user_identifier_snippet: Some("".into()),
                }),
                service_identifier: Some("does.not.matter".into()),
                spf: Some(Spf {
                    ip_address: Some("".into()),
                    result: Some(SpfResult::SoftFail),
                    smtp_mailfrom: Some("".into())
                })
            }
        )
    }

    fn sorted(mut data: OutputData) -> OutputData {
        data.parsed_mail.fulfillment_nodes.sort_by(|a, b| a.visible_url().cmp(b.visible_url()));

        data
    }
}

pub async fn enumerate(data: OutputData) -> OutputData {
    use tokio::task::JoinSet;

    let mut set: JoinSet<FulfillmentNode> = JoinSet::new();

    for node in data.parsed_mail.fulfillment_nodes.into_iter() {
        set.spawn(async  move{
            enumerate_visible_url(node).await
        });
    }

    let mut fulfillment_nodes = vec![];

    while let Some(res) = set.join_next().await {
        fulfillment_nodes.push(res.unwrap())
    }

    OutputData {
        parsed_mail: ParsedMail {
            fulfillment_nodes,
            ..data.parsed_mail
        },
        ..data
    }
}

#[cfg(test)]
mod enumerate_visible_url_tests {
    use super::*;
    use crate::data::Node;
    use crate::mountebank::*;

    #[test]
    fn sets_hidden_node_if_visible_url_redirects() {
        clear_all_impostors();
        setup_head_impostor(4545, true, Some("https://re.direct.one"));

        let actual = tokio_test::block_on(enumerate_visible_url(input(None)));

        assert_eq!(output(), actual);
    }

    #[test]
    fn sets_hidden_node_to_none_if_visible_url_does_not_redirect() {
        clear_all_impostors();
        setup_head_impostor(4545, false, Some("https://re.direct.one"));

        let actual = tokio_test::block_on(enumerate_visible_url(input(None)));

        assert_eq!(input(None), actual);
    }

    #[test]
    fn sets_hidden_node_to_none_if_request_fails() {
        let actual = tokio_test::block_on(enumerate_visible_url(input(Some("xxxx"))));

        assert_eq!(input(Some("xxxx")), actual);
    }

    #[test]
    fn sets_hidden_node_to_none_if_location_header_absent() {
        clear_all_impostors();
        setup_head_impostor(4545, true, None);

        let actual = tokio_test::block_on(enumerate_visible_url(input(None)));

        assert_eq!(input(None), actual);
    }

    #[test]
    fn sets_hidden_node_to_none_if_location_header_cannot_be_parsed() {
        clear_all_impostors();
        setup_head_impostor(4545, true, Some("»"));

        let actual = tokio_test::block_on(enumerate_visible_url(input(None)));

        assert_eq!(input(None), actual);
    }

    fn input(url: Option<&str>) -> FulfillmentNode {
        FulfillmentNode::new(url.unwrap_or("http://localhost:4545"))
    }

    fn output() -> FulfillmentNode {
        FulfillmentNode {
            hidden: Some(Node::new("https://re.direct.one")),
            ..input(None)
        }
    }
}

async fn enumerate_visible_url(mut node: FulfillmentNode) -> FulfillmentNode {
    // TODO Can this ever produce an error?
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build().unwrap();

    if let Ok(res) = client.head(node.visible_url()).send().await {
        match res.status() {
            reqwest::StatusCode::OK => (),
            _ => {
                if let Some(location_header) = res.headers().get("location") {
                    if let Ok(location) = location_header.to_str() {
                        node.set_hidden(location);
                    }
                }
            }
        }
    }

    node
}
