use crate::data::EmailAddressData;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[cfg(test)]
mod parse_header_tests {
    use super::*;

    #[test]
    fn generates_authentication_results_from_input() {
        let input = authentication_header(dkim_portion(), spf_portion(), dmarc_portion());

        assert_eq!(
            expected_result(),
            AuthenticationResults::parse_header(input)
        );
    }

    #[test]
    fn generates_authentication_results_from_empty_string() {
        assert_eq!(
            expected_result_empty(),
            AuthenticationResults::parse_header(String::from(""))
        );
    }

    fn authentication_header(
        dkim_portion: String,
        spf_portion: String,
        dmarc_portion: String,
    ) -> String {
        let provider = "mx.google.com";

        format!("{provider};\r  {dkim_portion};\r  {spf_portion};\r {dmarc_portion}")
    }

    fn dkim_portion() -> String {
        "dkim=pass header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm".into()
    }

    fn spf_portion() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=pass {parens} smtp.mailfrom={from}")
    }

    fn dmarc_portion() -> String {
        let from = "info@xxx.fr";
        let result = "dmarc=pass";
        let policy = "quarantine";
        let subdomain_policy = "reject";
        let disposition = "none";

        format!("{result} (p={policy} sp={subdomain_policy} dis={disposition}) header.from={from}")
    }

    fn expected_result() -> AuthenticationResults {
        AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Pass),
                selector: Some("ymy".into()),
                signature_snippet: Some("JPh8bpEm".into()),
                user_identifier_snippet: Some("@compromised.zzz".into()),
            }),
            service_identifier: Some("mx.google.com".into()),
            spf: Some(Spf {
                ip_address: Some("10.10.10.10".into()),
                result: Some(SpfResult::Pass),
                smtp_helo: None,
                smtp_mailfrom: Some("info@xxx.fr".into()),
            }),
        }
    }

    fn expected_result_empty() -> AuthenticationResults {
        AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        }
    }
}

#[cfg(test)]
mod authentication_results_valid_tests {
    use super::*;

    #[test]
    fn returns_false_if_no_authentication_results() {
        let address = spf_matching_address();
        let results = empty_authentication_results();

        assert!(!results.valid(&address));
    }

    #[test]
    fn returns_true_if_matching_spf_pass() {
        let address = spf_matching_address();
        let results = authentication_results_valid_spf_entry();

        assert!(results.valid(&address))
    }

    #[test]
    fn returns_true_if_matching_dkim_pass() {
        let address = dkim_matching_address();
        let results = authentication_results_valid_dkim_entry();

        assert!(results.valid(&address))
    }

    #[test]
    fn returns_false_if_matching_neither_dkim_nor_spf() {
        let address = dkim_matching_address();
        let results = authentication_results_no_valid_entries();

        assert!(!results.valid(&address))
    }

    fn spf_matching_address() -> EmailAddressData {
        EmailAddressData {
            address: "from@test.com".into(),
            domain: None,
            registrar: None,
        }
    }

    fn dkim_matching_address() -> EmailAddressData {
        EmailAddressData {
            address: "from@test.xxx".into(),
            domain: None,
            registrar: None,
        }
    }

    fn empty_authentication_results() -> AuthenticationResults {
        AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        }
    }

    fn authentication_results_valid_spf_entry() -> AuthenticationResults {
        AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: Some(Spf {
                ip_address: None,
                result: Some(SpfResult::Pass),
                smtp_helo: None,
                smtp_mailfrom: Some("from@test.com".into()),
            }),
        }
    }

    fn authentication_results_valid_dkim_entry() -> AuthenticationResults {
        AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Pass),
                selector: None,
                signature_snippet: None,
                user_identifier_snippet: Some("@test.xxx".into()),
            }),
            service_identifier: None,
            spf: None,
        }
    }

    fn authentication_results_no_valid_entries() -> AuthenticationResults {
        AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Pass),
                selector: None,
                signature_snippet: None,
                user_identifier_snippet: Some("@not.xxx".into()),
            }),
            service_identifier: None,
            spf: Some(Spf {
                ip_address: None,
                result: Some(SpfResult::Pass),
                smtp_helo: None,
                smtp_mailfrom: Some("recipient@not.com".into()),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AuthenticationResults {
    pub dkim: Option<Dkim>,
    pub service_identifier: Option<String>,
    pub spf: Option<Spf>,
}

impl AuthenticationResults {
    pub fn parse_header(header: String) -> Self {
        let snippets = TextSnippets::new(&header);

        Self {
            dkim: snippets.dkim.as_ref().map(|val| Dkim::new(val)),
            service_identifier: snippets.service_identifier.map(String::from),
            spf: snippets.spf.as_ref().map(|val| Spf::new(val)),
        }
    }

    pub fn valid(&self, email_address_data: &EmailAddressData) -> bool {
        self.valid_spf(email_address_data) || self.valid_dkim(email_address_data)
    }

    fn valid_spf(&self, email_address_data: &EmailAddressData) -> bool {
        match self.spf.as_ref() {
            Some(spf) => spf.valid(&email_address_data.address),
            None => false,
        }
    }

    fn valid_dkim(&self, email_address_data: &EmailAddressData) -> bool {
        match self.dkim.as_ref() {
            Some(dkim) => dkim.valid(&email_address_data.address),
            None => false,
        }
    }
}

#[cfg(test)]
mod text_snippets_tests {
    use super::*;

    #[test]
    fn creates_itself_from_complete_header() {
        let dkim = dkim_portion();
        let dmarc = dmarc_portion();
        let spf = spf_portion();

        let input = full_header_value();
        let expected = TextSnippets {
            dkim: Some(&dkim),
            dmarc: Some(&dmarc),
            service_identifier: Some("mx.google.com"),
            spf: Some(&spf),
        };

        assert_eq!(expected, TextSnippets::new(&input));
    }

    #[test]
    fn creates_itself_from_header_missing_dmarc() {
        let dkim = dkim_portion();
        let spf = spf_portion();

        let input = header_sans_dmarc();
        let expected = TextSnippets {
            dkim: Some(&dkim),
            dmarc: None,
            service_identifier: Some("mx.google.com"),
            spf: Some(&spf),
        };

        assert_eq!(expected, TextSnippets::new(&input));
    }

    #[test]
    fn creates_iself_from_header_missing_spf() {
        let dkim = dkim_portion();
        let dmarc = dmarc_portion();

        let input = header_sans_spf();
        let expected = TextSnippets {
            dkim: Some(&dkim),
            dmarc: Some(&dmarc),
            service_identifier: Some("mx.google.com"),
            spf: None,
        };

        assert_eq!(expected, TextSnippets::new(&input));
    }

    #[test]
    fn creates_iself_from_header_missing_dkim() {
        let dmarc = dmarc_portion();
        let spf = spf_portion();

        let input = header_sans_dkim();
        let expected = TextSnippets {
            dkim: None,
            dmarc: Some(&dmarc),
            service_identifier: Some("mx.google.com"),
            spf: Some(&spf),
        };

        assert_eq!(expected, TextSnippets::new(&input));
    }

    #[test]
    fn creates_itself_from_empty_header() {
        let expected = TextSnippets {
            dkim: None,
            dmarc: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(expected, TextSnippets::new(""));
    }

    fn full_header_value() -> String {
        let provider = "mx.google.com";
        let dkim = dkim_portion();
        let dmarc = dmarc_portion();
        let spf = spf_portion();

        format!("{provider};\r  {dkim};\r  {spf};\r {dmarc}")
    }

    fn header_sans_dmarc() -> String {
        let provider = "mx.google.com";
        let dkim = dkim_portion();
        let spf = spf_portion();

        format!("{provider};\r  {dkim};\r  {spf}")
    }

    fn header_sans_spf() -> String {
        let provider = "mx.google.com";
        let dkim = dkim_portion();
        let dmarc = dmarc_portion();

        format!("{provider};\r  {dkim};\r  {dmarc}")
    }

    fn header_sans_dkim() -> String {
        let provider = "mx.google.com";
        let dmarc = dmarc_portion();
        let spf = spf_portion();

        format!("{provider};\r  {spf};\r  {dmarc}")
    }

    fn dkim_portion() -> String {
        "dkim=pass header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm".into()
    }

    fn spf_portion() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=pass {parens} smtp.mailfrom={from}")
    }

    fn dmarc_portion() -> String {
        let from = "info@xxx.fr";
        let result = "dmarc=pass";
        let policy = "quarantine";
        let subdomain_policy = "reject";
        let disposition = "none";

        format!("{result} (p={policy} sp={subdomain_policy} dis={disposition}) header.from={from}")
    }
}

#[derive(Debug, PartialEq)]
struct TextSnippets<'a> {
    dkim: Option<&'a str>,
    dmarc: Option<&'a str>,
    service_identifier: Option<&'a str>,
    spf: Option<&'a str>,
}

impl<'a> TextSnippets<'a> {
    pub fn new(header: &'a str) -> Self {
        let snippets = Self::extract_snippets(header);

        Self {
            dkim: Self::extract_type_snippet(&snippets, Self::type_pattern("dkim")),
            dmarc: Self::extract_type_snippet(&snippets, Self::type_pattern("dmarc")),
            service_identifier: Self::extract_service_identifier(&snippets),
            spf: Self::extract_type_snippet(&snippets, Self::type_pattern("spf")),
        }
    }

    fn extract_snippets(header: &str) -> Vec<&str> {
        header.split(';').collect()
    }

    fn extract_service_identifier(snippets: &[&'a str]) -> Option<&'a str> {
        //TODO Figure out how to return an empty vec so we can force this unwrap out
        let candidate = snippets.first().copied().unwrap();

        if !candidate.is_empty() {
            Some(candidate)
        } else {
            None
        }
    }

    fn extract_type_snippet(snippets: &[&'a str], auth_pattern: Regex) -> Option<&'a str> {
        snippets
            .iter()
            .filter(|snippet| auth_pattern.is_match(snippet))
            .collect::<Vec<&&str>>()
            .first()
            .map(|snippet| (**snippet).trim())
    }

    fn type_pattern(type_snippet: &str) -> Regex {
        Regex::new(&format!(r"^\s+{type_snippet}=")).unwrap()
    }
}

#[cfg(test)]
mod dkim_new_tests {
    use super::*;

    #[test]
    fn creates_instance_of_dkim() {
        let input = "dkim=pass header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm";
        let expected = Dkim {
            result: Some(DkimResult::Pass),
            selector: Some("ymy".into()),
            signature_snippet: Some("JPh8bpEm".into()),
            user_identifier_snippet: Some("@compromised.zzz".into()),
        };

        assert_eq!(expected, Dkim::new(input))
    }
}

#[cfg(test)]
mod dkim_map_to_result_tests {
    use super::*;

    #[test]
    fn returns_result_option() {
        let input = "dkim=pass header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm";

        assert_eq!(Some(DkimResult::Pass), Dkim::map_to_result(input));
    }

    #[test]
    fn returns_none_if_value_cannot_be_parsed() {
        let input = "dkim=huh header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm";

        assert_eq!(None, Dkim::map_to_result(input));
    }

    #[test]
    fn returns_none_if_pattern_does_not_match_expectations() {
        let input = "xdkim=huh header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm";

        assert_eq!(None, Dkim::map_to_result(input));
    }
}

#[cfg(test)]
mod dkim_extract_value_tests {
    use super::*;

    #[test]
    fn returns_the_value_if_the_key_is_present() {
        let input = "dkim=pass header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm";

        assert_eq!(
            Some(String::from("ymy")),
            Dkim::extract_value(input, "header.s")
        )
    }

    #[test]
    fn returns_the_value_if_the_key_is_not_present() {
        let input = "dkim=pass header.i=@compromised.zzz xheader.s=ymy header.b=JPh8bpEm";

        assert_eq!(None, Dkim::extract_value(input, "header.s"))
    }
}

#[cfg(test)]
mod dkim_valid_tests {
    use super::*;

    #[test]
    fn returns_false_if_no_result() {
        let dkim = no_result_dkim();

        assert!(!dkim.valid("from@test.com"))
    }

    #[test]
    fn returns_true_if_pass_and_identifier_snippet_matches() {
        let dkim = pass_and_match_dkim();

        assert!(dkim.valid("from@test.com"))
    }

    #[test]
    fn returns_false_if_pass_and_no_identifier() {
        let dkim = pass_and_no_identifier_dkim();

        assert!(!dkim.valid("from@test.com"));
    }

    #[test]
    fn returns_false_if_not_pass_and_identifier_snippet_matches() {
        let dkim = not_pass_but_match_dkim();

        assert!(!dkim.valid("from@test.com"));
    }

    #[test]
    fn returns_false_if_pass_but_no_matching_identifier_snippet() {
        let dkim = pass_but_not_matching_dkim();

        assert!(!dkim.valid("from@test.com"))
    }

    fn no_result_dkim() -> Dkim {
        Dkim {
            result: None,
            selector: None,
            signature_snippet: None,
            user_identifier_snippet: Some("@test.com".into()),
        }
    }

    fn pass_and_match_dkim() -> Dkim {
        Dkim {
            result: Some(DkimResult::Pass),
            selector: None,
            signature_snippet: None,
            user_identifier_snippet: Some("@test.com".into()),
        }
    }

    fn pass_and_no_identifier_dkim() -> Dkim {
        Dkim {
            result: Some(DkimResult::Pass),
            selector: None,
            signature_snippet: None,
            user_identifier_snippet: None,
        }
    }

    fn not_pass_but_match_dkim() -> Dkim {
        Dkim {
            result: Some(DkimResult::None),
            selector: None,
            signature_snippet: None,
            user_identifier_snippet: Some("@test.com".into()),
        }
    }

    fn pass_but_not_matching_dkim() -> Dkim {
        Dkim {
            result: Some(DkimResult::Pass),
            selector: None,
            signature_snippet: None,
            user_identifier_snippet: Some("@not-test.com".into()),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Dkim {
    pub result: Option<DkimResult>,
    pub selector: Option<String>,
    pub signature_snippet: Option<String>,
    pub user_identifier_snippet: Option<String>,
}

impl Dkim {
    fn new(header: &str) -> Self {
        Self {
            result: Self::map_to_result(header),
            selector: Self::extract_value(header, "header.s"),
            signature_snippet: Self::extract_value(header, "header.b"),
            user_identifier_snippet: Self::extract_value(header, "header.i"),
        }
    }

    fn extract_value(header: &str, key: &str) -> Option<String> {
        let pattern = Regex::new(&format!(r"\s{key}=(\S+)")).unwrap();

        pattern
            .captures(header)
            .map(|captures| String::from(&captures[1]))
    }

    fn map_to_result(snippet: &str) -> Option<DkimResult> {
        // TODO Resisting the urge to use Self::extract_value() here so that the regex can be
        // tighter (i.e. only at the start of the string
        let pattern = Regex::new(r"\Adkim=(\S+)").unwrap();

        match pattern.captures(snippet) {
            Some(captures) => DkimResult::from_str(&captures[1]).ok(),
            None => None,
        }
    }

    fn valid(&self, email_address: &str) -> bool {
        matches!(self.result, Some(DkimResult::Pass))
            && self.email_matches_identifier(email_address)
    }

    fn email_matches_identifier(&self, email_address: &str) -> bool {
        match self.user_identifier_snippet.as_ref() {
            Some(snippet) => {
                let re = Regex::new(&format!(r"{}\z", &snippet)).unwrap();
                re.is_match(email_address)
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod test_dkim_result_from_str {
    use super::*;

    #[test]
    fn converts_string_to_enum_instance() {
        assert_eq!(DkimResult::Fail, DkimResult::from_str("fail").unwrap());
        assert_eq!(
            DkimResult::Neutral,
            DkimResult::from_str("neutral").unwrap()
        );
        assert_eq!(DkimResult::None, DkimResult::from_str("none").unwrap());
        assert_eq!(DkimResult::Pass, DkimResult::from_str("pass").unwrap());
        assert_eq!(
            DkimResult::PermError,
            DkimResult::from_str("permerror").unwrap()
        );
        assert_eq!(DkimResult::Policy, DkimResult::from_str("policy").unwrap());
        assert_eq!(
            DkimResult::TempError,
            DkimResult::from_str("temperror").unwrap()
        );
    }

    #[test]
    fn returns_an_error_for_unparseable_str() {
        assert!(DkimResult::from_str("unobtainium").is_err())
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum DkimResult {
    Fail,
    Neutral,
    None,
    Pass,
    PermError,
    Policy,
    TempError,
}

impl fmt::Display for DkimResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Fail => "Fail",
                Self::Neutral => "Neutral",
                Self::None => "None",
                Self::Pass => "Pass",
                Self::PermError => "PermError",
                Self::Policy => "Policy",
                Self::TempError => "TempError",
            }
        )
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ParseDkimResultError;

impl FromStr for DkimResult {
    type Err = ParseDkimResultError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fail" => Ok(Self::Fail),
            "neutral" => Ok(Self::Neutral),
            "none" => Ok(Self::None),
            "pass" => Ok(Self::Pass),
            "permerror" => Ok(Self::PermError),
            "policy" => Ok(Self::Policy),
            "temperror" => Ok(Self::TempError),
            _ => Err(ParseDkimResultError),
        }
    }
}

#[cfg(test)]
mod spf_new_tests {
    use super::*;

    #[test]
    fn creates_instance_from_header_with_mailfrom() {
        let input = header_with_mailfrom();
        let expected = Spf {
            ip_address: Some("10.10.10.10".into()),
            result: Some(SpfResult::Pass),
            smtp_helo: None,
            smtp_mailfrom: Some("info@xxx.fr".into()),
        };

        assert_eq!(expected, Spf::new(&input));
    }

    #[test]
    fn creates_instance_from_header_with_helo() {
        let input = header_with_helo();
        let expected = Spf {
            ip_address: Some("10.10.10.10".into()),
            result: Some(SpfResult::Pass),
            smtp_helo: Some("xxx.fr".into()),
            smtp_mailfrom: None,
        };

        assert_eq!(expected, Spf::new(&input));
    }

    fn header_with_mailfrom() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=pass {parens} smtp.mailfrom={from}")
    }

    fn header_with_helo() -> String {
        let helo = "xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!(
            "(google.com: domain of postmaster@{helo} designates {ip} as permitted sender)"
        );

        format!("spf=pass {parens} smtp.helo={helo}")
    }
}

#[cfg(test)]
mod spf_map_to_result_tests {
    use super::*;

    #[test]
    fn returns_result_if_match_succeeds() {
        let input = header();

        assert_eq!(Some(SpfResult::HardFail), Spf::map_to_result(&input));
    }

    #[test]
    fn returns_none_if_result_cannot_be_mapped() {
        let input = header_broken_result();

        assert_eq!(None, Spf::map_to_result(&input));
    }

    #[test]
    fn returns_none_if_header_cannot_be_parsed() {
        let input = broken_header();

        assert_eq!(None, Spf::map_to_result(&input));
    }

    fn header() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=hardfail {parens} smtp.mailfrom={from}")
    }

    fn header_broken_result() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=unobtainium {parens} smtp.mailfrom={from}")
    }

    fn broken_header() -> String {
        "xxxxx".into()
    }
}

#[cfg(test)]
mod spf_extract_mailfrom_tests {
    use super::*;

    #[test]
    fn returns_mailfrom() {
        let input = header();

        assert_eq!(
            Some(String::from("info@xxx.fr")),
            Spf::extract_mailfrom(&input)
        )
    }

    #[test]
    fn returns_none_if_unparseable_header() {
        let input = broken_header();

        assert_eq!(None, Spf::extract_mailfrom(&input))
    }

    fn header() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=hardfail {parens} smtp.mailfrom={from}")
    }

    fn broken_header() -> String {
        "xxxxx".into()
    }
}

#[cfg(test)]
mod spf_extract_helo_tests {
    use super::*;

    #[test]
    fn returns_helo() {
        let input = header();

        assert_eq!(
            Some(String::from("xxx.fr")),
            Spf::extract_helo(&input)
        );
    }

    #[test]
    fn returns_none_if_unparseable_header() {
        let input = broken_header();

        assert_eq!(None, Spf::extract_helo(&input))
    }

    fn header() -> String {
        let helo = "xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!(
            "(google.com: domain of postmaster@{helo} designates {ip} as permitted sender)");

        format!("spf=hardfail {parens} smtp.helo={helo}")
    }

    fn broken_header() -> String {
        "xxxxx".into()
    }
}

#[cfg(test)]
mod spf_extract_ip_address_tests {
    use super::*;

    #[test]
    fn returns_ip_address() {
        let input = header();

        assert_eq!(
            Some(String::from("10.10.10.10")),
            Spf::extract_ip_address(&input)
        );
    }

    #[test]
    fn returns_none_if_header_cannot_be_parsed() {
        let input = broken_header();

        assert_eq!(None, Spf::extract_ip_address(&input));
    }

    fn header() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=hardfail {parens} smtp.mailfrom={from}")
    }

    fn broken_header() -> String {
        "xxxxx".into()
    }
}

#[cfg(test)]
mod spf_valid_tests {
    use super::*;

    #[test]
    fn returns_true_if_spf_pass_and_mailfrom_matches() {
        let spf = spf_pass_matching_mailfrom();

        assert!(spf.valid("recipient@test.com"));
    }

    #[test]
    fn returns_true_if_spf_pass_and_helo_matches() {
        let spf = spf_pass_matching_helo();

        assert!(spf.valid("recipient@test.com"));
    }

    #[test]
    fn returns_false_if_not_pass() {
        let spf = spf_no_result_valid_mailfrom();
        assert!(!spf.valid("recipient@test.com"));

        let spf = spf_no_result_valid_helo();
        assert!(!spf.valid("recipient@test.com"));
    }

    #[test]
    fn returns_false_if_invalid_mailfrom() {
        let spf = spf_no_smtp_mailfrom();

        assert!(!spf.valid("recipient@test.com"));
    }

    #[test]
    fn returns_false_if_invalid_helo() {
        let spf = spf_no_smtp_helo();

        assert!(!spf.valid("recipient@test.com"))
    }

    fn spf_no_result_valid_mailfrom() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: None,
            smtp_mailfrom: Some("recipient@test.com".into()),
        }
    }

    fn spf_no_result_valid_helo() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: Some("test.com".into()),
            smtp_mailfrom: None,
        }
    }

    fn spf_no_smtp_mailfrom() -> Spf {
        Spf {
            ip_address: None,
            result: Some(SpfResult::Pass),
            smtp_helo: None,
            smtp_mailfrom: None,
        }
    }

    fn spf_no_smtp_helo() -> Spf {
        Spf {
            ip_address: None,
            result: Some(SpfResult::Pass),
            smtp_helo: None,
            smtp_mailfrom: None,
        }
    }

    fn spf_pass_matching_mailfrom() -> Spf {
        Spf {
            ip_address: None,
            result: Some(SpfResult::Pass),
            smtp_helo: None,
            smtp_mailfrom: Some("recipient@test.com".into()),
        }
    }

    fn spf_pass_matching_helo() -> Spf {
        Spf {
            ip_address: None,
            result: Some(SpfResult::Pass),
            smtp_helo: Some("test.com".into()),
            smtp_mailfrom: None,
        }
    }
}

#[cfg(test)]
mod spf_valid_result_tests {
    use super::*;

    #[test]
    fn is_true_if_result_is_pass() {
        assert!(spf_pass_result().valid_result())
    }

    #[test]
    fn is_false_if_no_result() {
        assert!(!spf_no_result().valid_result())
    }

    #[test]
    fn is_false_if_result_is_not_pass() {
        assert!(!spf_policy_result().valid_result())
    }

    fn spf_pass_result() -> Spf {
        Spf {
            ip_address: None,
            result: Some(SpfResult::Pass),
            smtp_helo: None,
            smtp_mailfrom: None
        }
    }

    fn spf_no_result() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: None,
            smtp_mailfrom: None
        }
    }

    fn spf_policy_result() -> Spf {
        Spf {
            ip_address: None,
            result: Some(SpfResult::Policy),
            smtp_helo: None,
            smtp_mailfrom: None
        }
    }
}

#[cfg(test)]
mod spf_valid_mailfrom_tests {
    use super::*;

    #[test]
    fn is_false_if_no_mailfrom() {
        let email_address = "foo@test.zzz";

        assert!(!spf_no_mailfrom().valid_mailfrom(email_address))        
    }

    #[test]
    fn is_true_if_mailfrom_matches() {
        let email_address = "foo@test.zzz";

        assert!(spf_mailfrom_matches().valid_mailfrom(email_address))        
    }

    #[test]
    fn is_false_if_mailfrom_does_not_match() {
        let email_address = "foo@test.zzz";

        assert!(!spf_mailfrom_does_not_match().valid_mailfrom(email_address))        
    }

    fn spf_no_mailfrom() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: None,
            smtp_mailfrom: None
        }
    }

    fn spf_mailfrom_matches() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: None,
            smtp_mailfrom: Some("foo@test.zzz".into())
        }
    }

    fn spf_mailfrom_does_not_match() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: None,
            smtp_mailfrom: Some("not-foo@test.zzz".into())
        }
    }
}

#[cfg(test)]
mod spf_valid_helo_tests {
    use super::*;

    #[test]
    fn returns_false_if_none() {
        let spf = spf_no_helo();

        assert!(!spf.valid_helo("recipient@test.com"))
    }

    #[test]
    fn returns_true_if_helo_matches_email_address_domain() {
        let spf = spf_helo_match();

        assert!(spf.valid_helo("recipient@test.com"))
    }

    #[test]
    fn returns_false_if_helo_does_not_match_email_address_domain() {
        let spf = spf_helo_mismatch();

        assert!(!spf.valid_helo("recipient@test.com"))
    }

    #[test]
    fn returns_false_if_email_address_is_broken() {
        let spf = spf_helo_match();

        assert!(!spf.valid_helo("recipienttest.com"))
    }

    fn spf_no_helo() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: None,
            smtp_mailfrom: None
        }
    }

    fn spf_helo_match() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: Some("test.com".into()),
            smtp_mailfrom: None
        }
    }

    fn spf_helo_mismatch() -> Spf {
        Spf {
            ip_address: None,
            result: None,
            smtp_helo: Some("not-test.com".into()),
            smtp_mailfrom: None
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Spf {
    pub ip_address: Option<String>,
    pub result: Option<SpfResult>,
    pub smtp_helo: Option<String>,
    pub smtp_mailfrom: Option<String>,
}

impl Spf {
    fn new(header: &str) -> Self {
        Self {
            ip_address: Self::extract_ip_address(header),
            result: Self::map_to_result(header),
            smtp_helo: Self::extract_helo(header),
            smtp_mailfrom: Self::extract_mailfrom(header),
        }
    }

    fn map_to_result(header: &str) -> Option<SpfResult> {
        let pattern = Regex::new(r"\Aspf=(\S+)").unwrap();

        match pattern.captures(header) {
            Some(captures) => SpfResult::from_str(&captures[1]).ok(),
            _ => None,
        }
    }

    fn extract_mailfrom(header: &str) -> Option<String> {
        let pattern = Regex::new(r"smtp.mailfrom=(.+)\z").unwrap();

        pattern
            .captures(header)
            .map(|captures| String::from(&captures[1]))
    }

    fn extract_ip_address(header: &str) -> Option<String> {
        let pattern = Regex::new(r"designates\s(\S+)\s").unwrap();

        pattern
            .captures(header)
            .map(|captures| String::from(&captures[1]))
    }

    fn extract_helo(header: &str) -> Option<String> {
        let pattern = Regex::new(r"smtp.helo=(.+)\z").unwrap();

        pattern
            .captures(header)
            .map(|captures| String::from(&captures[1]))
    }

    fn valid(&self, email_address: &str) -> bool {
        self.valid_result() &&
            (self.valid_mailfrom(email_address) || self.valid_helo(email_address))
    }

    fn valid_mailfrom(&self, email_address: &str) -> bool {
        if let Some(mailfrom_address) = &self.smtp_mailfrom {
            email_address == mailfrom_address
        } else {
            false
        }
    }

    fn valid_helo(&self, email_address: &str) -> bool {
        if let Some(helo) = &self.smtp_helo {
            if let Some(domain) = email_address.split('@').collect::<Vec<&str>>().pop() {
                domain == helo
            } else {
                false
            }
        } else {
            false
        }
    }

    fn valid_result(&self) -> bool {
        matches!(self.result, Some(SpfResult::Pass))
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum SpfResult {
    HardFail,
    Neutral,
    None,
    Pass,
    PermError,
    Policy,
    SoftFail,
    TempError,
}

impl fmt::Display for SpfResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::HardFail => "HardFail",
                Self::Neutral => "Neutral",
                Self::None => "None",
                Self::Pass => "Pass",
                Self::PermError => "PermError",
                Self::Policy => "Policy",
                Self::SoftFail => "SoftFail",
                Self::TempError => "TempError",
            }
        )
    }
}

#[cfg(test)]
mod spf_result_from_str_tests {
    use super::*;

    #[test]
    fn converts_string_to_enum_instance() {
        assert_eq!(
            SpfResult::HardFail,
            SpfResult::from_str("hardfail").unwrap()
        );
        assert_eq!(SpfResult::Neutral, SpfResult::from_str("neutral").unwrap());
        assert_eq!(SpfResult::None, SpfResult::from_str("none").unwrap());
        assert_eq!(SpfResult::Pass, SpfResult::from_str("pass").unwrap());
        assert_eq!(
            SpfResult::PermError,
            SpfResult::from_str("permerror").unwrap()
        );
        assert_eq!(SpfResult::Policy, SpfResult::from_str("policy").unwrap());
        assert_eq!(
            SpfResult::SoftFail,
            SpfResult::from_str("softfail").unwrap()
        );
        assert_eq!(
            SpfResult::TempError,
            SpfResult::from_str("temperror").unwrap()
        );
    }

    #[test]
    fn returns_an_error_for_an_unmappable_value() {
        assert!(SpfResult::from_str("unobtainum").is_err())
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ParseSpfResultError;

impl FromStr for SpfResult {
    type Err = ParseSpfResultError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "hardfail" => Ok(Self::HardFail),
            "neutral" => Ok(Self::Neutral),
            "none" => Ok(Self::None),
            "pass" => Ok(Self::Pass),
            "permerror" => Ok(Self::PermError),
            "policy" => Ok(Self::Policy),
            "softfail" => Ok(Self::SoftFail),
            "temperror" => Ok(Self::TempError),
            _ => Err(ParseSpfResultError),
        }
    }
}
