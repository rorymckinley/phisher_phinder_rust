use assert_cmd::Command;
use assert_json_diff::assert_json_eq;
use predicates::prelude::*;
use serde_json::json;

#[test]
fn test_display_human_parse_results() {
    let mut cmd = Command::cargo_bin("pp-parser").unwrap();

    cmd.args(["--human"])
        .write_stdin(input())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("info@xxx.fr")
                .and(predicates::str::contains("touch base"))
                .and(predicates::str::contains("https://foo.bar/baz"))
                .and(predicates::str::contains("gp13mtaq123"))
                .and(predicates::str::contains("JPh8bpEm"))
                .and(predicates::str::contains("Trusted")),
        );
}

#[test]
fn test_display_json_parse_results() {
    let mut cmd = Command::cargo_bin("pp-parser").unwrap();

    let assert = cmd.write_stdin(input()).assert().success();
    let json_data = &assert.get_output().stdout;

    let json_data: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(json_data).unwrap()).unwrap();

    assert_json_eq!(json_output_as_value(input_data()), json_data);
}

fn input_data() -> String {
    "\
Delivered-To: victim@gmail.com\r
Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp;\r
        Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
X-Google-Smtp-Source: AA6agR7rW0ljZbXj2cQn9NgC8m6wViE4veg3Wroa/sb4ZEQMZAmVYdUGb9EAPvGvoF9UkmUip/o+\r
X-Received: by 2002:a05:6402:35cf:b0:448:84a9:12cf with SMTP id z15-20020a05640235cf00b0044884a912cfmr745701edc.51.1662506240653;\r
        Tue, 06 Sep 2022 16:17:20 -0700 (PDT)\r
ARC-Authentication-Results: i=1; mx.google.com;\r
       spf=pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) smtp.mailfrom=info@xxx.fr\r
Return-Path: <info@xxx.fr>\r
Received: from foo.bar.com (foo.bar.com. [10.10.10.10])\r
        by mx.google.com with ESMTP id jg8-2002\r
        for <victim@gmail.com>;\r
        Tue, 06 Sep 2022 16:17:20 -0700 (PDT)\r
Received-SPF: pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) client-ip=10.10.10.10;\r
Authentication-Results: mx.google.com;\r
        dkim=pass header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm;\r
        spf=pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) smtp.mailfrom=info@xxx.fr\r
Received: from not-real-one.com (not-real-one.com )\r
  (envelope-from <g-123-456-789-012@blah.not-real-two.com (g-123-456-789-012@blah.not-real-two.com)>)\r
  by gp13mtaq123 (mtaq-receiver/2.20190311.1) with ESMTP id yA3jJ-_S5g8Z\r
  for <not.real.three@comcast.net>; Thu, 30 May 2019 19:00:22 +0200\r
Date: Tue, 6 Sep 2022 19:17:19 -0400\r
From: \"Case evaluations\" <PIBIeSRqUtiEw1NCg4@gmail.com>\r
To: victim@gmail.com\r
Message-ID: <Ctht0YgNZJDaAVPvcU36z2Iw9f7Bs7Jge.ecdasmtpin_added_missing@mx.google.com>\r
Subject: We’re sorry that we didn’t touch base with you earlier. f309\r
MIME-Version: 1.0\r
Content-Type: text/html\r\n\r
<div style=\"width:650px;margin:0 auto;font-family:verdana;font-size:16px\">\r
<a href=\"https://foo.bar/baz\">Click Me</a>
</div>\r
".into()
}

fn input() -> String {
    json!(
        {
            "id": 9909,
            "data": input_data()
        }
    )
    .to_string()
}

fn json_output_as_value(source_data: String) -> serde_json::Value {
    json!({
        "parsed_mail": {
            "authentication_results": {
                "dkim": {
                    "result": "Pass",
                    "selector": "ymy",
                    "signature_snippet": "JPh8bpEm",
                    "user_identifier_snippet": "@compromised.zzz",
                },
                "service_identifier": "mx.google.com",
                "spf": {
                    "ip_address": "10.10.10.10",
                    "result": "Pass",
                    "smtp_helo": null,
                    "smtp_mailfrom": "info@xxx.fr"
                }
            },
            "delivery_nodes": [
                {
                    "advertised_sender": null,
                    "observed_sender": null,
                    "position": 0,
                    "recipient": "2002:a05:7300:478f:b0:75:5be4:1dc0",
                    "time": "2022-09-06T23:17:20Z",
                    "trusted": false,
                },
                {
                    "advertised_sender": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "foo.bar.com",
                            "registration_date": null,
                            "resolved_domain": null,
                        },
                        "host": "foo.bar.com",
                        "infrastructure_provider": null,
                        "ip_address": null,
                        "registrar": null,
                    },
                    "observed_sender": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "foo.bar.com",
                            "registration_date": null,
                            "resolved_domain": null,
                        },
                        "host": "foo.bar.com",
                        "infrastructure_provider": null,
                        "ip_address": "10.10.10.10",
                        "registrar": null,
                    },
                    "position": 1,
                    "recipient": "mx.google.com",
                    "time": "2022-09-06T23:17:20Z",
                    "trusted": true
                },
                {
                    "advertised_sender": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "not-real-one.com",
                            "registration_date": null,
                            "resolved_domain": null,
                        },
                        "host": "not-real-one.com",
                        "infrastructure_provider": null,
                        "ip_address": null,
                        "registrar": null,
                    },
                    "observed_sender": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "not-real-one.com",
                            "registration_date": null,
                            "resolved_domain": null,
                        },
                        "host": "not-real-one.com",
                        "infrastructure_provider": null,
                        "ip_address": null,
                        "registrar": null,
                    },
                    "position": 2,
                    "recipient": "gp13mtaq123",
                    "time": "2019-05-30T17:00:22Z",
                    "trusted": false
                }
            ],
            "email_addresses": {
                "from": [
                    {
                        "address": "PIBIeSRqUtiEw1NCg4@gmail.com",
                        "domain": {
                            "abuse_email_address": "abuse@gmail.com",
                            "category": "open_email_provider",
                            "name": "gmail.com",
                            "registration_date": null,
                            "resolved_domain": null,
                        },
                        "registrar": null,
                    }
                ],
                "links": [],
                "reply_to": [],
                "return_path": [
                    {
                        "address": "info@xxx.fr",
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "xxx.fr",
                            "registration_date": null,
                            "resolved_domain": null,
                        },
                        "registrar": null,
                    }
                ]
            },
            "fulfillment_nodes": [
                {
                    "visible": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "foo.bar",
                            "registration_date": null,
                            "resolved_domain": null,
                        },
                        "registrar": null,
                        "url": "https://foo.bar/baz",
                    },
                    "investigable": true,
                    "hidden": null,
                }
            ],
            "subject": "We’re sorry that we didn’t touch base with you earlier. f309",
        },
        "message_source": {
            "id": 9909,
            "data": source_data
        },
        "notifications": [],
        "reportable_entities": null,
        "run_id": null
    })
}

//     let input = r#"Delivered-To: victim@gmail.com\r
// Received: by 2002:a05:7300:478f:b0:75:5be4:1dc0 with SMTP id r15csp4024141dyk;\r
//         Tue, 6 Sep 2022 16:17:20 -0700 (PDT)\r
// X-Google-Smtp-Source: AA6agR7rW0ljZbXj2cQn9NgC8m6wViE4veg3Wroa/sb4ZEQMZAmVYdUGb9EAPvGvoF9UkmUip/o+\r
// X-Received: by 2002:a05:6402:35cf:b0:448:84a9:12cf with SMTP id z15-20020a05640235cf00b0044884a912cfmr745701edc.51.1662506240653;\r
//         Tue, 06 Sep 2022 16:17:20 -0700 (PDT)\r
// ARC-Seal: i=1; a=rsa-sha256; t=1662506240; cv=none;\r
//         d=google.com; s=arc-20160816;\r
//         b=ECdDIcxr4yPESWzVH+8A9q3jeDsbV7qb83aTLS0Sp3rc5+krFWWSywSt8QpI8xzwO1\r
//          RAQrXL0hkHonuSjE9QjLh0AyUknC6ISm4ia0q59IbeuqvSVPS5/fSaKOrBRtxbGBT/Sq\r
//          PeVDLnnOW3Vi7bwTmEvI3lvPfMjT0widjGNJIz92pfXFD/F7bRsQDW8ph9uHJAz/vjL+\r
//          1s56ktJYhtEe/BUQ99XdBQgdhfMvm+qkZ+ze3hIAEWH/a2JV1ESQtPeeTiBwu3E/1Ios\r
//          928oAcEcZcPB+8DHIKlAftrBbDAVQFwea0UeiKOpAfwyXg4wCeTAMEFPZE0xyI5W4ig/\r
//          dWnQ==\r
// ARC-Message-Signature: i=1; a=rsa-sha256; c=relaxed/relaxed; d=google.com; s=arc-20160816;\r
//         h=mime-version:subject:message-id:to:from:date;\r
//         bh=qXvzv08vyQsgvJiswgvEuYGnh1jlBRGhNlceJmZcdrE=;\r
//         b=iAdxPfMXubLj7RjY4zT6ZYXyfnD41LONY4QXHud+MWKGC3LjhmFu2nSGVDNeDVuZjt\r
//          bzziOD/79PX5z0Tg3x2tFDw+PnZnsARVMWU0vOJ09YT3RmRMBYnNEi4NjtQec8lkQIpZ\r
//          pqK2D5j3kpny+IrFYpi66sBCk+Mxq8eF2plCTkzMA33Bav/ueteDdT/f2OPIQVrPrRCk\r
//          swyVROCpMfR0EE8C/yq1iBuSRYbVv1NEpXF6dzTq226tM1DlnQ0FGerB6Al92uWDD42F\r
//          wEowyM5szLWospJYqcW5Siv7vNplu/VLB+z2D/yp5QtrsCNikJJqkhfuG7EKlpz+pLMp\r
//          zXKA==\r
// ARC-Authentication-Results: i=1; mx.google.com;\r
//        spf=pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) smtp.mailfrom=info@xxx.fr\r
// Return-Path: <info@xxx.fr>\r
// Received: from foo.bar.com (foo.bar.com. [10.10.10.10])\r
//         by mx.google.com with ESMTP id jg8-20020a170907970800b0072b83ed8d42si10970498ejc.82.2022.09.06.16.17.19\r
//         for <victim@gmail.com>;\r
//         Tue, 06 Sep 2022 16:17:20 -0700 (PDT)\r
// Received-SPF: pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) client-ip=10.10.10.10;\r
// Authentication-Results: mx.google.com;\r
//        spf=pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) smtp.mailfrom=info@xxx.fr\r
// Date: Tue, 6 Sep 2022 19:17:19 -0400\r
// From: "Case evaluations" <PIBIeSRqUtiEw1NCg4@fake.net>\r
// To: victim@gmail.com\r
// Message-ID: <Ctht0YgNZJDaAVPvcU36z2Iw9f7Bs7Jge.ecdasmtpin_added_missing@mx.google.com>\r
// Subject: We’re sorry that we didn’t touch base with you earlier. f309\r
// MIME-Version: 1.0\r
// Content-Type: text/html\r
// \r
// <center><html xmlns="http://www.w3.org/1999/xhtml"><head></head><body>
// <div style="width:650px;margin:0 auto;font-family:verdana;font-size:16px">
// <hr>
// <p><h3>Settlement Payment Came In?</h3>
// Due to your involvement in a class action or injury lawsuit,<br><br> a <b>settlement payment may have just came in</b> at the following webpage. <br><br>
// <b>Go there now to accept what's reserved in your name. </b><br><br>
// <u><a href="https://storage.googleapis.com/teampass/Ha231120/hrf2zsdf/newb2.html#2395706vW5715755DL628799111Ps1694Ry24Jlr150436OY"  style="text-decoration:none;">>> CHECK FOR POTENTIAL SETTLEMENT <<</a></u>
//
// </p><hr>
// <p style="font-size:12px">
//  <a href="https://storage.googleapis.com/teampass/Ha231120/hrf2zsdf/newb2.html#2395706nJ5715755Ai628799111vp1694Qw24Jnu150436ha">UNSUBSCRIBE</a>
//  <br>
//  <br> This adress is for mail only:
// 1820 Avenue M#534 - Brooklyn, NY 11230</p>
// </div>
//     "#;
//
