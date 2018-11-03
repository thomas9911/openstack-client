#[macro_use]
extern crate serde_json;
extern crate openstack;
extern crate argparse;
extern crate waiter;

use serde_json::Value;
use std::collections::HashMap;

use std::env;
use std::fmt::Debug;
use std::io::{stdout, stderr};

use argparse::{ArgumentParser, StoreTrue, Store, List};

mod fmt_methods;

use fmt_methods::*;

use std::fs::File;

// use waiter::{Waiter, WaiterCurrentState};
use waiter::{Waiter, WaiterCurrentState};

fn main() {
    // let user_input = "kaas";

    // let resource: OSResource = match user_input.parse(){
    //     Ok(x) => x,
    //     Err(_e) => OSResource::None
    // };
    // new_os.print_list(resource);

    // println!("{:?}", arguments);

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

    // let os_cloud: &str = "fuga";

    env::set_var("OS_CLOUD", os_cloud);
    let os = match openstack::Cloud::from_env(){
        Ok(x) => x,
        Err(e) =>  {
            println!("Error: {}", e);
            std::process::exit(0);
            },
    };

    let new_os = MyOS::new(os);
    args.insert(0, format!("{} {:?}", "openstack", command));

    // println!("{:?}", args);
    match command{
        OSOperation::List => list_command(new_os, args),
        OSOperation::Show => show_command(new_os, args),
        OSOperation::New => new_command(new_os, args),
        OSOperation::Delete => delete_command(new_os, args),
        OSOperation::None => (),
    }

}   

fn list_command(os: MyOS, args: Vec<String>){
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

fn show_command(os: MyOS, args: Vec<String>){
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

fn new_command(os: MyOS, args: Vec<String>){
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

fn delete_command(os: MyOS, args: Vec<String>){
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

#[derive(Debug, Clone)]
enum OSOperation{
    List,
    Show,
    New,
    Delete,
    None,
}

impl std::str::FromStr for OSOperation{
    type Err = ();

    fn from_str(s: &str) -> Result<OSOperation, ()> {
        match s.to_lowercase().as_str() {
            "show" | "get" => Ok(OSOperation::Show),
            "list" | "ls" => Ok(OSOperation::List),
            "new" | "create" => Ok(OSOperation::New),
            "delete" | "remove" | "rm" => Ok(OSOperation::Delete),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum OSResource{
    Flavors, 
    FloatingIps,
    Images,
    Keypairs,
    Networks,
    Servers,
    Subnets,
    Ports,
    None,
}

impl std::str::FromStr for OSResource{
    type Err = ();

    fn from_str(s: &str) -> Result<OSResource, ()> {
        match convert_to_singular(s).as_str() {
            "flavor" | "size" => Ok(OSResource::Flavors),
            "floating_ip" | "fips" => Ok(OSResource::FloatingIps),
            "image" | "operating_system" => Ok(OSResource::Images),            
            "keypair" | "keys" => Ok(OSResource::Keypairs),
            "network" => Ok(OSResource::Networks),
            "server" => Ok(OSResource::Servers),
            "subnet" => Ok(OSResource::Subnets),
            "port" => Ok(OSResource::Ports),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
struct MyOS{
    client: openstack::Cloud,
}

impl MyOS{
    fn new(os_client: openstack::Cloud) -> MyOS{
        MyOS{client: os_client}
    }
    fn print_list(&self, resource: OSResource){
        match resource{
            OSResource::Flavors => print_flavor_summary_data(self.client.list_flavors()),
            OSResource::FloatingIps => print_floating_ip_data(self.client.list_floating_ips()),
            OSResource::Images => print_image_data(self.client.list_images()),
            OSResource::Keypairs => print_key_pair_data(self.client.list_keypairs()),
            OSResource::Networks => print_network_data(self.client.list_networks()),
            OSResource::Servers => print_server_summary_data(self.client.list_servers()),
            OSResource::Subnets => print_subnet_data(self.client.list_subnets()),
            OSResource::Ports => print_port_data(self.client.list_ports()),
            OSResource::None => println!("[{{\"error\": \"resource cannot be listed\"}}]"),
        }
    }

    fn print_get(&self, resource: OSResource, name: String){
        if name == ""{
            println!("{}", serde_json::to_string_pretty(
                    &fmt_error("'name or id' is a required argument")
                ).unwrap());
            return
        }
        let result = match resource{
            OSResource::Flavors => {
                fmt_flavor(self.client.get_flavor(name))
            },
            // OSResource::FloatingIps => print_floating_ip_data(self.client.list_floating_ips()),
            // OSResource::Images => print_image_data(self.client.list_images()),
            OSResource::Keypairs => {
                 match self.client.get_keypair(name){
                     Ok(x) => fmt_key_pair(x),
                     Err(x) => fmt_error(x)
                 }
            },
                
            // OSResource::Networks => print_network_data(self.client.list_networks()),
            OSResource::Servers => {
                 match self.client.get_server(name){
                     Ok(x) => fmt_server(x),
                     Err(x) => fmt_error(x)
                 }
            },
            // OSResource::Subnets => print_subnet_data(self.client.list_subnets()),
            // OSResource::Ports => print_port_data(self.client.list_ports()),
            // OSResource::None => json!([{"error": "resource cannot be showed"}]),
            _ => fmt_error("resource cannot be showed"),
        };
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }

    fn print_delete(&self, resource: OSResource, name: String){
        if name == ""{
            println!("{}", serde_json::to_string_pretty(
                    &fmt_error("'name or id' is a required argument")
                ).unwrap());
            return
        }

        let result = match resource{
            // OSResource::Flavors => print_flavor_summary_data(self.client.list_flavors()),
            // OSResource::FloatingIps => print_floating_ip_data(self.client.list_floating_ips()),
            // OSResource::Images => print_image_data(self.client.list_images()),
            OSResource::Keypairs => {
                 match self.client.get_keypair(name){
                     Ok(x) => {match x.delete(){
                         Ok(_x) => json!({"info": "keypair deleted"}),
                         Err(x) => fmt_error(x)
                     }},
                     Err(x) => fmt_error(x)
                 }
            },
                
            // OSResource::Networks => print_network_data(self.client.list_networks()),
            // OSResource::Servers => print_server_summary_data(self.client.list_servers()),
            // OSResource::Subnets => print_subnet_data(self.client.list_subnets()),
            // OSResource::Ports => print_port_data(self.client.list_ports()),
            // OSResource::None => json!([{"error": "resource cannot be showed"}]),
            _ => fmt_error("resource cannot be deleted"),
        };
        println!("{}", serde_json::to_string_pretty(&result).unwrap());    
    }

    fn create_keypair(&self, options: HashMap<String, String>) -> Result<Value, String>{
        let name = match options.get("name"){
            Some(x) => x.to_owned(),
            _ => return Err("name is required".to_string())
        };

        let public_key = match options.get("pk"){
            Some(x) => x.to_owned(),
            _ => return Err("public-key is required".to_string())
        };

        let mut file = match File::open(public_key.clone()){
            Ok(x) => x,
            _ => return Err(format!("'{}' file is not available", public_key))
        };
        
        let return_value = match self.client.new_keypair(name)
                    .from_reader(&mut file).expect("file reading goes wrong")
                    .create(){
                Ok(x) => fmt_key_pair(x),
                _ => Value::from("Something went wrong while creating a keypair")
        };
        Ok(return_value)
    }

    fn create_server(&self, options: HashMap<String, String>) -> Result<Value, String>{
        let name = match options.get("name"){
            Some(x) => x.to_owned(),
            _ => return Err("name is required".to_string())
        };

        let flavor_name = match options.get("flavor"){
            Some(x) => x.to_owned(),
            _ => return Err("flavor is required".to_string())
        };

        let image_name = match options.get("image"){
            Some(x) => x.to_owned(),
            _ => return Err("image is required".to_string())
        };

        let keypair_name = match options.get("keypair"){
            Some(x) => x.to_owned(),
            _ => return Err("keypair is required".to_string())
        };

        let network_name = match options.get("network"){
            Some(x) => x.to_owned(),
            _ => return Err("network is required".to_string())
        };


        let flavor = match self.client.get_flavor(flavor_name) {
            Ok(x) => x,
            Err(x) => return Err(x.to_string())
        };

        let image = match find_images(image_name, self.client.list_images()) {
            Ok(x) => x,
            Err(x) => return Err(x.to_string())
        };

        let keypair = match self.client.get_keypair(keypair_name) {
            Ok(x) => x,
            Err(x) => return Err(x.to_string())
        };

        let network = match self.client.get_network(network_name){
            Ok(x) => x,
            Err(x) => return Err(x.to_string())
        };

        let return_value = match self.client.new_server(name, flavor)
                    .with_image(image)
                    .with_keypair(keypair)
                    .with_network(network)
                    .create(){
                Ok(mut x) => {match x.poll(){
                    Ok(x) => fmt_server(x.unwrap()),
                    Err(x) => fmt_error(x.to_string())
                }},
                // Ok(mut x) => {match x.wait(){
                //     Ok(x) => fmt_server(x),
                //     Err(x) => fmt_error(x.to_string())
                // }},
                Err(e) => fmt_error(e.to_string())
        };
        Ok(return_value)
    }
}

fn add_if_exists<S>(hashmap: &mut HashMap<String, String>, name: S, item: String)where S: ToString{
    if item != ""{
        hashmap.insert(name.to_string(), item);
    };
}

fn fmt_error<S>(error: S) -> Value where S: ToString{
    return json!({"error": error.to_string()});
}

fn convert_to_singular(blub: &str) -> String {
    // not 100% bulletproof but good enough for matching
    let tmp = blub.to_lowercase();

    let last_char = tmp[(tmp.len()-1)..].chars().next().unwrap();

    let is_multiple = match last_char{
        's' => true,
        _ => false
    };

    if is_multiple{
        tmp[..(tmp.len()-1)].to_string()
    }
    else{
        tmp
    }
}


fn find_images(name: String, images: openstack::Result<Vec<openstack::image::Image>>)  -> Result<openstack::image::Image, String> {
    let legit_images = match images{
        Ok(x) => x,
        Err(x) => return Err(x.to_string())
    };

    let mut choices = vec![];
    for image in legit_images{
        if image.name().to_lowercase().starts_with(name.to_lowercase().as_str()){
            choices.push(image);
        }
    };

    // Ok(choices.first().unwrap())
    match choices.first(){
        Some(x) => Ok(x.clone()),
        _ => Err("No valid image found".to_string())
    }
}