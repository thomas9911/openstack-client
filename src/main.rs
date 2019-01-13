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

    let (resource_input, resource_sub) = match command_sub.subcommand(){
        (x, Some(y)) => (x, y),
        (_x, None) => return ()
    };


    let matches_options = make_args_from_arg_matches(&matches);
    let command_options = make_args_from_arg_matches(command_sub);
    let resource_options = make_args_from_arg_matches(resource_sub);

    if let Some(x) = get_first_value_from_hashmap_with_vec(&matches_options, "os-cloud"){
        os_cloud = x;
    };

    // println!("{}", os_cloud);

    // println!("{:?}", command_input);
    // println!("{:?}", resource_input);
    // println!("{:?}", matches_options);
    // println!("{:?}", command_options);
    // println!("{:?}", resource_options);

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

    let os_command = OSOperation::from(command_input);

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
