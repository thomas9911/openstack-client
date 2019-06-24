extern crate pest;
extern crate serde_json;
#[macro_use]
extern crate pest_derive;

mod common;
use common::{exec_command, Output};

use serde_json::json;

#[test]
fn list_servers() {
    let expected = Output::new(
        "GET",
        "https://example.com/compute/servers?",
        json!({"x-auth-token": "token"}),
        json!(null),
    );
    assert_eq!(expected, exec_command(vec!["list", "servers"]));
}

#[test]
fn get_server() {
    let expected = Output::new(
        "GET",
        "https://example.com/compute/servers/123456789?",
        json!({"x-auth-token": "token"}),
        json!(null),
    );
    assert_eq!(expected, exec_command(vec!["get", "server", "123456789"]));
}

#[test]
fn list_images() {
    let expected = Output::new(
        "GET",
        "https://example.com/image/v2/images?",
        json!({"x-auth-token": "token"}),
        json!(null),
    );
    assert_eq!(expected, exec_command(vec!["list", "images"]));
}

#[test]
fn list_volumes() {
    let expected = Output::new(
        "GET",
        "https://example.com/volumev3/volumes?",
        json!({"x-auth-token": "token"}),
        json!(null),
    );
    assert_eq!(expected, exec_command(vec!["list", "volumes"]));
}

#[test]
fn create_keypair() {
    let expected = Output::new(
        "POST",
        "https://example.com/compute/os-keypairs?",
        json!({"x-auth-token": "token"}),
        json!({
          "keypair": {
            "name": "testing"
          }
        }),
    );
    assert_eq!(
        expected,
        exec_command(vec!["new", "keypair", "--name", "testing"])
    );
}

#[test]
fn create_server() {
    let expected = Output::new(
        "POST",
        "https://example.com/compute/servers?",
        json!({"x-auth-token": "token"}),
        json!({
          "server": {
            "flavorRef": "flavor-id-1234",
            "imageRef": "image-id-1234",
            "key_name": "keyname",
            "name": "testing",
            "networks": [
              {
                "uuid": "network-id-1234"
              }
            ]
          }
        }),
    );
    assert_eq!(
        expected,
        exec_command(vec![
            "create",
            "server",
            "--name",
            "testing",
            "--key-name",
            "keyname",
            "--network-id",
            "network-id-1234",
            "--image",
            "image-id-1234",
            "--flavor",
            "flavor-id-1234"
        ])
    );
}

#[test]
fn call_option() {
    let expected = Output::new(
        "POST",
        "https://example.com/compute/os-keypair?",
        json!({"x-auth-token": "token", "Accept": " text/plain"}),
        json!({
            "key1": {
                "key2": "value"
            }
        }),
    );

    assert_eq!(
        expected,
        exec_command(vec![
            "call",
            "POST",
            "compute",
            "os-keypair",
            r#"{
                "key1": {
                    "key2": "value"
                }
            }"#,
            "-H",
            "Accept: text/plain",
        ])
    );
}

#[test]
fn list_network_with_multiple_filters(){
    let expected = Output::new(
        "GET",
        "https://example.com/network/v2.0/networks?name=test&name=test2",
        json!({"x-auth-token": "token"}),
        json!(null),
    );

    assert_eq!(
        expected,
        exec_command(vec![
            "list",
            "network",
            "--name",
            "test",
            "--name",
            "test2",
        ])
    );
}