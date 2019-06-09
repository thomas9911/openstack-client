extern crate serde_json;
mod common;
use common::{create_cmd, get_stdout, make_args, Output};

extern crate pest;
#[macro_use]
extern crate pest_derive;

use serde_json::json;

#[test]
fn list_servers() {
    let expected = Output::new(
        "GET",
        "https://example.com/compute/servers?",
        json!({"x-auth-token": "token"}),
        json!(null),
    );

    let mut cmd = create_cmd();
    let raw_output = get_stdout(cmd.args(make_args(vec!["list", "servers"])));
    let output = Output::from_stdout(&raw_output);
    assert_eq!(expected, output);
}

#[test]
fn get_server() {
    let expected = Output::new(
        "GET",
        "https://example.com/compute/servers/123456789?",
        json!({"x-auth-token": "token"}),
        json!(null),
    );

    let mut cmd = create_cmd();
    let raw_output = get_stdout(cmd.args(make_args(vec!["get", "server", "123456789"])));
    let output = Output::from_stdout(&raw_output);
    assert_eq!(expected, output);
}

#[test]
fn list_images() {
    let expected = Output::new(
        "GET",
        "https://example.com/image/v2/images?",
        json!({"x-auth-token": "token"}),
        json!(null),
    );

    let mut cmd = create_cmd();
    let raw_output = get_stdout(cmd.args(make_args(vec!["list", "images"])));
    let output = Output::from_stdout(&raw_output);
    assert_eq!(expected, output);
}

#[test]
fn list_volumes() {
    let expected = Output::new(
        "GET",
        "https://example.com/volumev3/volumes?",
        json!({"x-auth-token": "token"}),
        json!(null),
    );

    let mut cmd = create_cmd();
    let raw_output = get_stdout(cmd.args(make_args(vec!["list", "volumes"])));
    let output = Output::from_stdout(&raw_output);
    assert_eq!(expected, output);
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

    let mut cmd = create_cmd();
    let raw_output = get_stdout(cmd.args(make_args(vec!["new", "keypair", "--name", "testing"])));
    let output = Output::from_stdout(&raw_output);
    assert_eq!(expected, output);
}
