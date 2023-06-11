use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;

#[cfg(test)]
mod parse_header_tests {
    use super::*;

    #[test]
    fn generates_authentication_results_from_input()  {
        let input = authentication_header(dkim_portion(), spf_portion(), dmarc_portion());

        assert_eq!(expected_result(), AuthenticationResults::parse_header(input));
    }

    fn authentication_header(
        dkim_portion: String,
        spf_portion: String,
        dmarc_portion: String
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
            spf: Some(Spf{
                ip_address: Some("10.10.10.10".into()),
                result: Some(SpfResult::Pass),
                smtp_mailfrom: Some("info@xxx.fr".into()),
            })
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
        // println!("PH {}", header);
        let snippets = TextSnippets::new(&header);

        Self {
            dkim: Some(Dkim::new(snippets.dkim.unwrap())),
            service_identifier: snippets.service_identifier.map(String::from),
            spf: Some(Spf::new(snippets.spf.unwrap())),
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
            spf: Some(&spf)
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
            spf: Some(&spf)
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
            spf: None
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

        let service_identifier = snippets.first().copied();

        Self {
            dkim: Self::extract_type_snippet(&snippets, Self::type_pattern("dkim")),
            dmarc: Self::extract_type_snippet(&snippets, Self::type_pattern("dmarc")),
            service_identifier,
            spf: Self::extract_type_snippet(&snippets, Self::type_pattern("spf")),
        }
    }

    fn extract_snippets(header: &str) -> Vec<&str> {
        header
            .split(';')
            .collect()
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
            user_identifier_snippet: Some("@compromised.zzz".into())
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

        assert_eq!(Some(String::from("ymy")), Dkim::extract_value(input, "header.s"))
    }

    #[test]
    fn returns_the_value_if_the_key_is_not_present() {
        let input = "dkim=pass header.i=@compromised.zzz xheader.s=ymy header.b=JPh8bpEm";

        assert_eq!(None, Dkim::extract_value(input, "header.s"))
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

        pattern.captures(header).map(|captures| String::from(&captures[1]))
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
}

#[cfg(test)]
mod test_dkim_result_from_str {
    use super::*;

    #[test]
    fn converts_string_to_enum_instance() {
        assert_eq!(DkimResult::Fail, DkimResult::from_str("fail").unwrap());
        assert_eq!(DkimResult::Neutral, DkimResult::from_str("neutral").unwrap());
        assert_eq!(DkimResult::None, DkimResult::from_str("none").unwrap());
        assert_eq!(DkimResult::Pass, DkimResult::from_str("pass").unwrap());
        assert_eq!(DkimResult::PermError, DkimResult::from_str("permerror").unwrap());
        assert_eq!(DkimResult::Policy, DkimResult::from_str("policy").unwrap());
        assert_eq!(DkimResult::TempError, DkimResult::from_str("temperror").unwrap());
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
                Self::TempError => "TempError"
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
            _ => Err(ParseDkimResultError)
        }
    }
}

#[cfg(test)]
mod spf_new_tests {
    use super::*;

    #[test]
    fn creates_instance_from_header() {
        let input = header();
        let expected = Spf {
            ip_address: Some("10.10.10.10".into()),
            result: Some(SpfResult::Pass),
            smtp_mailfrom: Some("info@xxx.fr".into()),
        };

        assert_eq!(expected, Spf::new(&input));
    }

    fn header() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=pass {parens} smtp.mailfrom={from}")
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

        assert_eq!(Some(String::from("info@xxx.fr")), Spf::extract_mailfrom(&input))
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
mod spf_extract_ip_address_tests {
    use super::*;

    #[test]
    fn returns_ip_address() {
        let input = header();

        assert_eq!(Some(String::from("10.10.10.10")), Spf::extract_ip_address(&input));
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

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Spf {
    pub ip_address: Option<String>,
    pub result: Option<SpfResult>,
    pub smtp_mailfrom: Option<String>
}

impl Spf {
    fn new(header: &str) -> Self {
        Self {
            ip_address: Self::extract_ip_address(header),
            result: Self::map_to_result(header),
            smtp_mailfrom: Self::extract_mailfrom(header)
        }
    }

    fn map_to_result(header: &str) -> Option<SpfResult> {
        let pattern = Regex::new(r"\Aspf=(\S+)").unwrap();

        match pattern.captures(header) {
            Some(captures) => SpfResult::from_str(&captures[1]).ok(),
            _ => None
        }
    }

    fn extract_mailfrom(header: &str) -> Option<String> {
        let pattern = Regex::new(r"smtp.mailfrom=(.+)\z").unwrap();

        pattern.captures(header).map(|captures| String::from(&captures[1]))
    }

    fn extract_ip_address(header: &str) -> Option<String> {
        let pattern = Regex::new(r"designates\s(\S+)\s").unwrap();

        pattern.captures(header).map(|captures| String::from(&captures[1]))
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
                Self::TempError => "TempError"
            }
        )
    }
}

#[cfg(test)]
mod spf_result_from_str_tests {
    use super::*;

    #[test]
    fn converts_string_to_enum_instance() {
        assert_eq!(SpfResult::HardFail, SpfResult::from_str("hardfail").unwrap());
        assert_eq!(SpfResult::Neutral, SpfResult::from_str("neutral").unwrap());
        assert_eq!(SpfResult::None, SpfResult::from_str("none").unwrap());
        assert_eq!(SpfResult::Pass, SpfResult::from_str("pass").unwrap());
        assert_eq!(SpfResult::PermError, SpfResult::from_str("permerror").unwrap());
        assert_eq!(SpfResult::Policy, SpfResult::from_str("policy").unwrap());
        assert_eq!(SpfResult::SoftFail, SpfResult::from_str("softfail").unwrap());
        assert_eq!(SpfResult::TempError, SpfResult::from_str("temperror").unwrap());
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
            _ => Err(ParseSpfResultError)
        }
    }
}
