use std::process::{self, Command};
use std::env;
use std::io::Write;
use std::collections::HashMap;

use serde_json::json;

pub fn create_cmd() -> Command{
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

    let filtered_env : HashMap<String, String> =
    env::vars().filter(|&(ref k, _)|
        !k.starts_with("OS_")
    ).collect();

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
    cmd.env_clear().envs(filtered_env).envs(env)
        // .arg("-vvv")
        .arg("--use-cache");
    cmd
}


pub fn make_args(mut args: Vec<&'static str>) -> Vec<&str>
{
    args.insert(1, "--dry-run");
    args
}

pub fn get_stdout(cmd: &mut Command) -> String{
    let output = cmd.output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}