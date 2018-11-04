#[macro_use]
extern crate serde_json;
extern crate openstack;
extern crate argparse;
extern crate waiter;

mod enums;
mod utils;
mod openstack_connection;

use std::collections::HashMap;

use std::env;
use std::io::{stdout, stderr};

use argparse::{ArgumentParser, StoreTrue, Store, List};

mod fmt_methods;

use enums::{OSOperation, OSResource};
use openstack_connection::{OpenstackConnection};
use utils::add_if_exists;


fn main() {
    let mut os_cloud: String = "fuga".to_string();
    let mut command: OSOperation = OSOperation::None;
    let mut args: Vec<String> = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Openstack cli in Rust maddafaka");
        ap.refer(&mut os_cloud)
            .add_option(&["--os-cloud"], Store,
            "cloud name from the clouds.yaml");
        ap.refer(&mut command)
            .add_argument("command", Store,
                        "Command to run");
         ap.refer(&mut args)
            .add_argument("arguments", List,
                    r#"Arguments for command"#);
                    
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    env::set_var("OS_CLOUD", os_cloud);
    let os = match openstack::Cloud::from_env(){
        Ok(x) => x,
        Err(e) =>  {
            println!("Error: {}", e);
            std::process::exit(0);
            },
    };

    let new_os = OpenstackConnection::new(os);
    args.insert(0, format!("{} {:?}", "openstack", command));

    match command{
        OSOperation::List => list_command(new_os, args),
        OSOperation::Show => show_command(new_os, args),
        OSOperation::New => new_command(new_os, args),
        OSOperation::Delete => delete_command(new_os, args),
        OSOperation::None => (),
    }

}   

fn list_command(os: OpenstackConnection, args: Vec<String>){
    let mut resource: OSResource = OSResource::None;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("list a resource");
        ap.refer(&mut resource)
            .add_argument("resource", Store,
                            "resource to list");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) =>  {},
            Err(_e) => {}
        }
    }
    os.print_list(resource);
}

fn show_command(os: OpenstackConnection, args: Vec<String>){
    let mut resource: OSResource = OSResource::None;
    let mut name_or_id = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("list a resource");
        ap.refer(&mut resource)
            .add_argument("resource", Store,
                            "resource to show");
        ap.refer(&mut name_or_id)
            .add_argument("name_or_id", Store,
                            "name or id of object to show");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) =>  {},
            Err(_e) => {}
        }
    }
    os.print_get(resource, name_or_id)
}

fn new_command(os: OpenstackConnection, args: Vec<String>){
    let mut resource: OSResource = OSResource::None;
    let mut name = "".to_string();
    let mut pk = "".to_string();
    let mut flavor = "".to_string();
    let mut image = "".to_string();
    let mut keypair = "".to_string();
    let mut network = "public".to_string();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("create a new object");
        ap.refer(&mut resource)
            .add_argument("resource", Store,
                            "object to create");
        ap.refer(&mut name)
            .add_option(&["--name", "-n"], Store,
                            "how to name object");
        ap.refer(&mut pk)
            .add_option(&["--pk"], Store,
                            "the publickey to create keypair with (only for keypair resource)");
        ap.refer(&mut keypair)
            .add_option(&["--keypair"], Store,
                            "the publickey to create keypair with (only for keypair resource)");
        ap.refer(&mut flavor)
            .add_option(&["--flavor", "--size"], Store,
                            "the flavor/size of the new server (only for server resource)");
        ap.refer(&mut image)
            .add_option(&["--image", "--os"], Store,
                            "the image/os of the new server (only for server resource)");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) =>  {},
            Err(_e) => {}
        }
    }
    let mut hashmap: HashMap<String, String> = HashMap::new();
    add_if_exists(&mut hashmap, "name", name);
    add_if_exists(&mut hashmap, "pk", pk);
    add_if_exists(&mut hashmap, "flavor", flavor);
    add_if_exists(&mut hashmap, "image", image);
    add_if_exists(&mut hashmap, "keypair", keypair);
    add_if_exists(&mut hashmap, "network", network);


    let txt = match resource{
        OSResource::Keypairs => os.create_keypair(hashmap),
        OSResource::Servers => os.create_server(hashmap),
        _ => Err("Resource cannot be created".to_string())
    };
    match txt{
        Ok(x) => println!("{}", serde_json::to_string_pretty(&x).unwrap()),
        Err(x) => println!("ERROR: {}", x),
    };
}

fn delete_command(os: OpenstackConnection, args: Vec<String>){
    let mut resource: OSResource = OSResource::None;
    let mut name_or_id = "".to_string();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("create a new object");
        ap.refer(&mut resource)
            .add_argument("resource", Store,
                            "object to create");
        ap.refer(&mut name_or_id)
            .add_argument("name_or_id", Store,
                            "name for object to delete");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) =>  {},
            Err(_e) => {}
        }
    }

    os.print_delete(resource, name_or_id)
}
