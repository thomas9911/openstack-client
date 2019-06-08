extern crate serde_json;
mod common;
use common::{create_cmd, make_args, get_stdout};

#[test]
fn list_servers(){
    let expected = "Some(\"GET\") Some(\"https://example.com/compute/servers?\")
Headers: {\"x-auth-token\": \"token\"}
null
";

    let mut cmd = create_cmd();
    let output = get_stdout(cmd.args(make_args(vec!["list", "servers"])));
    assert_eq!(expected, output);
}


#[test]
fn get_server(){
    let expected = "Some(\"GET\") Some(\"https://example.com/compute/servers/123456789?\")
Headers: {\"x-auth-token\": \"token\"}
null
";

    let mut cmd = create_cmd();
    let output = get_stdout(cmd.args(make_args(vec!["get", "server", "123456789"])));
    assert_eq!(expected, output);
}


#[test]
fn list_images(){
    let expected = "Some(\"GET\") Some(\"https://example.com/image/v2/images?\")
Headers: {\"x-auth-token\": \"token\"}
null
";

    let mut cmd = create_cmd();
    let output = get_stdout(cmd.args(make_args(vec!["list", "images"])));
    assert_eq!(expected, output);
}


#[test]
fn list_volumes(){
    let expected = "Some(\"GET\") Some(\"https://example.com/volumev3/volumes?\")
Headers: {\"x-auth-token\": \"token\"}
null
";

    let mut cmd = create_cmd();
    let output = get_stdout(cmd.args(make_args(vec!["list", "volumes"])));
    assert_eq!(expected, output);
}
