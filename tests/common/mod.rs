use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::process::{self, Command};

use pest::Parser;
use serde_json::json;

#[derive(Parser)]
#[grammar = "../tests/output.pest"]
struct OutputParser;

#[derive(Debug, PartialEq)]
pub struct Output {
    http_method: String,
    url: String,
    headers: serde_json::Value,
    body: serde_json::Value,
}

impl Output {
    pub fn new<S: Into<String>>(
        http_method: S,
        url: S,
        headers: serde_json::Value,
        body: serde_json::Value,
    ) -> Self {
        Output {
            http_method: http_method.into(),
            url: url.into(),
            headers,
            body,
        }
    }

    pub fn from_stdout(output: &str) -> Self {
        to_output(output)
    }
}

impl From<Vec<&str>> for Output {
    fn from(input: Vec<&str>) -> Self {
        if input.len() != 4 {
            panic!("output parser doesnt work anymore");
        };
        Output {
            http_method: input[0].to_string(),
            url: input[1].to_string(),
            headers: serde_json::from_str(input[2])
                .expect("this was created by to_string function"),
            body: serde_json::from_str(input[3]).expect("this was created by to_string function"),
        }
    }
}

pub fn create_cmd() -> Command {
    let root = env::current_exe()
        .unwrap()
        .parent()
        .expect("test executable's directory")
        .parent()
        .expect("executable's directory")
        .to_path_buf();

    let program;
    if cfg!(windows) {
        program = root.join("openstack-client.exe");
    } else {
        program = root.join("openstack-client");
    }

    let env = vec![
        ("OS_USERNAME", "username"),
        ("OS_PASSWORD", "password"),
        ("OS_AUTH_URL", "https://example.com"),
    ];

    let filtered_env: HashMap<String, String> = env::vars()
        .filter(|&(ref k, _)| !k.starts_with("OS_"))
        .collect();

    let mut dir = env::temp_dir();
    let auth_cache_name = "openstack-client-E78E50ECD12BFBAA";
    dir.push(auth_cache_name);

    let mut file = std::fs::File::create(dir).expect("unable to create file");

    let cache = json!({
        "config": {
            "cloud_name": "fuga",
            "region_name": "ams",
            "interface": "public",
            "auth": {
                "user_id": "",
                "username": "username",
                "user_domain_id": "1234",
                "user_domain_name": "domain",
                "token": "",
                "auth_url": "https://example.com",
                "system_scope": "",
                "domain_id": "",
                "domain_name": "",
                "project_id": "4567",
                "project_name": "project",
                "project_domain_id": "1234",
                "project_domain_name": "domain",
                "trust_id": ""
            },
            "only_use_public_endpoints": true
        },
        "token": "token",
        "token_expiry": "2270-06-02T21:16:34.000000Z",
        "endpoints": {
            "key-manager": "https://example.com/key",
            "object-store": "https://example.com/object-store",
            "volume": "https://example.com/volume",
            "identity": "https://example.com/identity",
            "orchestration": "https://example.com/orchestration",
            "metric": "https://example.com/metric",
            "image": "https://example.com/image",
            "cloudformation": "https://example.com/cloudformation",
            "network": "https://example.com/network",
            "volumev3": "https://example.com/volumev3",
            "metering": "https://example.com/metering",
            "placement": "https://example.com/placement",
            "volumev2": "https://example.com/volumev2",
            "compute": "https://example.com/compute"
        },
        "domain_id": "1234",
        "user_id": "9876"
    });

    file.write(format!("{}", cache).as_bytes()).unwrap();

    let mut cmd = process::Command::new(program);
    cmd.env_clear()
        .envs(filtered_env)
        .envs(env)
        // .arg("-vvv")
        .arg("--use-cache");
    cmd
}

pub fn make_args(mut args: Vec<&'static str>) -> Vec<&str> {
    args.insert(1, "--dry-run");
    args
}

pub fn get_stdout(cmd: &mut Command) -> String {
    let output = cmd.output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

fn to_output(parsable: &str) -> Output {
    let parsed = OutputParser::parse(Rule::output, parsable)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap();

    let mut stuff = vec![];
    for line in parsed.into_inner() {
        match line.as_rule() {
            Rule::some => {
                let mut rule_some = line.into_inner();
                let text_in_some = rule_some.next().unwrap().as_str();
                println!("{}", text_in_some);
                stuff.push(text_in_some);
            }
            Rule::string => {
                let mut inner_rules = line.into_inner();
                let current_section_name = inner_rules.next().unwrap().as_str();
                println!("{}", current_section_name);
                stuff.push(current_section_name);
            }
            Rule::json => {
                let json_str = line.as_str();
                println!("{}", json_str);
                stuff.push(json_str);
            }
            Rule::EOI => (),
            _ => (println!("{:?}", line)),
        }
    }
    Output::from(stuff)
}

#[test]
fn test_output_parser_1() {
    let parsable = "Some(\"POST\") Some(\"https://example.com/compute/os-keypairs?\")
Headers: {\"x-auth-token\": \"token\"}
{
  \"keypair\": {
    \"name\": \"testing\"
  }
}
";
    let http_method = "POST";
    let url = "https://example.com/compute/os-keypairs?";
    let headers = json!({"x-auth-token": "token"});
    let body = json!({"keypair": {"name": "testing"}});

    let output = to_output(parsable);
    assert_eq!(output.http_method, http_method);
    assert_eq!(output.url, url);
    assert_eq!(output.headers, headers);
    assert_eq!(output.body, body);
}

#[test]
fn test_output_parser_2() {
    let parsable = "Some(\"GET\") Some(\"https://example.com/object-store/container/123456/?\")
Headers: {\"x-auth-token\": \"token\"}
null
";
    let http_method = "GET";
    let url = "https://example.com/object-store/container/123456/?";
    let headers = json!({"x-auth-token": "token"});
    let body = serde_json::Value::Null;

    let output = to_output(parsable);
    assert_eq!(output.http_method, http_method);
    assert_eq!(output.url, url);
    assert_eq!(output.headers, headers);
    assert_eq!(output.body, body);
}
