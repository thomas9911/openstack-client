use openstack;

use std::collections::HashMap;
use serde_json::{Value, to_string_pretty};

use std::fs::File;
use waiter::Waiter;

use fmt_methods::*;

use enums::OSResource;

#[derive(Debug)]
pub struct OpenstackConnection{
    client: openstack::Cloud,
}

impl OpenstackConnection{
    pub fn new(os_client: openstack::Cloud) -> OpenstackConnection{
        OpenstackConnection{client: os_client}
    }
    pub fn print_list(&self, resource: OSResource){
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

    pub fn print_get(&self, resource: OSResource, name: String){
        if name == ""{
            println!("{}", to_string_pretty(
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
        println!("{}", to_string_pretty(&result).unwrap());
    }

    pub fn print_delete(&self, resource: OSResource, name: String){
        if name == ""{
            println!("{}", to_string_pretty(
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
        println!("{}", to_string_pretty(&result).unwrap());
    }

    pub fn create_keypair(&self, options: HashMap<String, String>) -> Result<Value, String>{
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

    pub fn create_server(&self, options: HashMap<String, String>) -> Result<Value, String>{
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


pub fn fmt_error<S>(error: S) -> Value where S: ToString{
    return json!({"error": error.to_string()});
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
