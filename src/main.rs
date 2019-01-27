extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;

extern crate chrono;
#[macro_use]
extern crate clap;
extern crate heck;
extern crate secstr;
extern crate reqwest;
extern crate rpassword;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate yaml_rust;
extern crate uuid;

mod enums;
mod utils;
mod openstack_connection;
mod structs;

use std::collections::HashMap;
use std::str::FromStr;
use std::env;

use clap::{Arg, App, SubCommand, Shell};
use std::string::ToString;

use enums::OSOperation;
use openstack_connection::{OpenstackInfoMap, Openstack};
use utils::{get_first_value_from_hashmap_with_vec};


fn main() {
    let mut os_cloud: String = "".to_string();
    if env::var("OS_CLOUD").is_ok(){
        os_cloud = env::var("OS_CLOUD").unwrap()
    }

    let yml = load_yaml!("../data/cool.yaml");
    let app = App::from_yaml(&yml).subcommand(SubCommand::with_name("generate-autocomplete")
                                    .about("generates autocompletion scripts")
                                    .arg(Arg::with_name("shell")
                                        .possible_values(&Shell::variants())
                                        .help("which shell to generate script for")));
    let matches = app.get_matches();
    // println!("{:?}", matches);

    let (command_input, command_sub) = match matches.subcommand(){
        (x, Some(y)) => (x, y),
        (_x, None) => return ()
    };

    if let Some(sub_m) = matches.subcommand_matches("generate-autocomplete") {
        // assert_eq!(sub_m.value_of("shell"), Some("bash"));
        let a_shell: Shell = match sub_m.value_of("shell"){
            Some(x) => Shell::from_str(x).unwrap(),
            None => return
        };
        App::from_yaml(&yml).gen_completions_to(crate_name!(), a_shell, &mut std::io::stdout());
        return ();
    }

    let matches_options = make_args_from_arg_matches(&matches);
    let command_options = make_args_from_arg_matches(command_sub);
    let os_command = OSOperation::from(command_input);

    if let Some(x) = get_first_value_from_hashmap_with_vec(&matches_options, "os-cloud"){
        os_cloud = x;
    };

    let os_config_env = OpenstackInfoMap::from_env(os_cloud.clone());
    let mut os_config = match OpenstackInfoMap::from_clouds_yaml(os_cloud.clone()){
        Ok(x) => x,
        Err(_e) => OpenstackInfoMap::default()
    };
    os_config.apply(&os_config_env)
      .add_password_if_not_existing().unwrap();

    let mut new_os = match Openstack::new(os_config){
        Ok(x) => x,
        Err(e) => {println!("{}", e); return}
    };

    if os_command == OSOperation::Call {
        println!("{:?}", command_sub);
        let method = command_sub.value_of("method").expect("this value is requered");
        let os_type = command_sub.value_of("type").expect("this value is requered");
        let endpoint = command_sub.value_of("endpoint").expect("this value is requered");
        let body = command_sub.values_of("body");
        println!("{} {} {}", method, os_type, endpoint);
        let resource_type = match new_os.resources.get_resource_type(os_type.into()){
            Ok(x) => x,
            Err(e) => {println!("{{\"error\": \"{}\"}}", e); return ()}
        };
        let http_method = match reqwest::Method::from_str(method){
            Ok(x) => x,
            Err(e) => {println!("{{\"error\": \"{}\"}}", e); return ()}
        };

        let mut req = new_os.make_url(http_method, resource_type, endpoint.into());

        req = match body {
            Some(x) => {
                let q: String = x.collect::<Vec<&str>>().join(" ");
                println!("{}", q);
                let v: serde_json::Value = match serde_json::from_str(&q){
                    Ok(x) => x,
                    Err(e) => {println!("{{\"error\": \"{}\"}}", e); return ()}
                };
                req.json(&v)
            },
            None => req
        };
        let mut lbab = req.send().expect("request failed");
        let outcome = match Openstack::handle_response(&mut lbab){
            Ok(x) => x,
            Err(e) => {println!("{}", e); return}
        };

        println!("{}", serde_json::to_string_pretty(&outcome).unwrap());
        return ();
    }

    let (resource_input, resource_sub) = match command_sub.subcommand(){
        (x, Some(y)) => (x, y),
        (_x, None) => return ()
    };
    let resource_options = make_args_from_arg_matches(resource_sub);


    // println!("{}", os_cloud);

    // println!("{:?}", command_input);
    // println!("{:?}", resource_input);
    // println!("{:?}", matches_options);
    // println!("{:?}", command_options);
    // println!("{:?}", resource_options);



    if !new_os.is_resource_available(resource_input.into()){
        println!("error: endpoint for resource '{}' is not available", resource_input);
        return ()
    };

    let outcome = match new_os.act(os_command, resource_input.to_string(), &command_options, &resource_options){
        Ok(x) => x,
        Err(e) => {println!("{}", e); return}
    };

    println!("{}", serde_json::to_string_pretty(&outcome).unwrap());

}


fn make_args_from_arg_matches(matches: &clap::ArgMatches) -> HashMap<String, Vec<String>>{
    let mut options: HashMap<String, Vec<String>> = HashMap::new();
    for (k, v) in matches.args.iter(){
        options.insert(k.to_string(), v.vals.iter().map(|x| x.clone().into_string().unwrap()).collect());
    };
    options
}
