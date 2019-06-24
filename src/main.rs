extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;
extern crate csv;
extern crate envy;
#[macro_use]
extern crate prettytable;
extern crate handlebars;
extern crate indicatif;

extern crate chrono;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate heck;
extern crate secstr;
extern crate curl;
extern crate url;
// extern crate reqwest;
extern crate http;
extern crate rpassword;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate yaml_rust;
extern crate uuid;
extern crate memmap;
extern crate sha2;
extern crate multimap;

#[macro_use]
mod macros;

mod error;
mod client;
mod enums;
mod utils;
mod openstack_connection;
mod structs;
mod objectstore;
mod traits;
mod config;

use std::collections::HashMap;
use std::str::FromStr;
use std::env;

use clap::{Arg, App, SubCommand, Shell};
use std::string::ToString;


use enums::OSOperation;
use structs::{Command, Resource};
use config::{OpenstackInfoMap};
use openstack_connection::{Openstack};
use utils::{get_first_value_from_hashmap_with_vec, print_value, make_args_from_arg_matches};
use error::OpenstackError;


fn main() {
    let mut os_cloud: String = "".to_string();
    if env::var("OS_CLOUD").is_ok(){
        os_cloud = env::var("OS_CLOUD").unwrap()
    }

    let yml = load_yaml!("../data/cli.yaml");
    let app = prepare_app(App::from_yaml(&yml));
    let matches = app.get_matches();
    // println!("{:?}", matches);
    set_log_level(&matches);

    let format = matches.value_of("format").expect("this has a default");
    debug!("set output format to {}", format);

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
        prepare_app(App::from_yaml(&yml)).gen_completions_to("openstack-client", a_shell, &mut std::io::stdout()); // crate_name!()
        return ();
    }

    let matches_options = make_args_from_arg_matches(&matches);
    let command_options = make_args_from_arg_matches(command_sub);
    // let os_command = OSOperation::from(command_input);

    if let Some(x) = get_first_value_from_hashmap_with_vec(&matches_options, "os-cloud"){
        os_cloud = x.to_string();
    };

    let os_config_env = OpenstackInfoMap::from_env(os_cloud.clone());
    debug!("config from env {:?}", os_config_env);
    let mut os_config = match OpenstackInfoMap::from_clouds_yaml(os_cloud.clone()){
        Ok(x) => x,
        Err(_e) => OpenstackInfoMap::default()
    };
    debug!("config from yaml {:?}", os_config);

    os_config.apply(&os_config_env)
      .add_password_if_not_existing().unwrap();

    debug!("config combined {:?}", os_config);

    let mut new_os = match matches_options.get("use-cache").is_some(){
        true => {
            debug!("trying to use cache");
            match Openstack::from_cache_or_new_refreshed(os_config){
                Ok(x) => x,
                Err(e) => return print_error(e, format)
            }
        }
        false => {
            debug!("trying to create new openstack-client");
            match Openstack::new(os_config){
                Ok(x) => x,
                Err(e) => return print_error(e, format)
            }
        }
    };

    if OSOperation::from(command_input) == OSOperation::Call {
        debug!("initialize call operation");

        let method = command_sub.value_of("method").expect("this value is required");
        let os_type = command_sub.value_of("type").expect("this value is required");
        let endpoint = command_sub.value_of("endpoint").expect("this value is required");
        let body = command_sub.values_of("body");
        let headers = command_sub.values_of("header");

        let resource_type = match new_os.resources.get_resource_type(os_type.into()){
            Ok(x) => x,
            Err(e) => {println!("{{\"error\": \"{}\"}}", e); return ()}
        };
        let tmp_resource: Resource = serde_json::from_value(json!({
            "name": "tmp",
            "endpoint_path": endpoint,
            "resource_type": resource_type,
        })).unwrap();
        let _http_method = match http::Method::from_str(method){
            Ok(x) => x,
            Err(e) => {println!("{{\"error\": \"{}\"}}", e); return ()}
        };
        let command: Command = serde_json::from_value(
            json!({
                "aliases": [],
                "has_body": false,
                "help": "",
                "requires_id": false,
                "http_method": method
            })
        ).unwrap();

        if let Some(x) = headers {
            for header in x{
                let header_parts: Vec<&str> = header.split(":").collect();
                if header_parts.len() != 2 {
                    return print_value(&json!({"error": "invalid header"}), format);
                }
                new_os.connection.client.set_header(header_parts[0], header_parts[1])
            }
        };

        new_os.make_url(command, &tmp_resource, endpoint.into(), HashMap::new(), &None, None);

        debug!("{:?}", new_os.connection.client);

        if let Some(x) = body {
            let q: String = x.collect::<Vec<&str>>().join(" ");
            let v: serde_json::Value = match serde_json::from_str(&q){
                Ok(x) => x,
                Err(_e) => {
                    let error = json!({
                        "error": "given body is invalid json"
                    });
                    print_value(&error, format);
                    return ()
                }
            };
            new_os.connection.client.set_json(v);
        };

        if command_options.get("dry-run").is_some(){
            println!("{:?} {:?}\nHeaders: {:?}", new_os.connection.client.method, new_os.connection.client.url, new_os.connection.client.headers);
            print_value(&new_os.connection.client.json, format);
            return ();
        }
        // let mut lbab = req.send().expect("request failed");
        let mut response = new_os.connection.client.perform().expect("request failed");
        let outcome = match Openstack::handle_response(&mut response){
            Ok(x) => x,
            Err(e) => {println!("{}", e); return}
        };

        print_value(&outcome, format);
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
    if command_input == "new" && resource_input == "tokens"{
        if let Some(token) = new_os.connection.token{
            let return_object = json!({
                "token": token
            });
            print_value(&return_object, format);
            return;
        }
        else{
            let return_object = json!({
                "error": "token is not available"
            });
            print_value(&return_object, format);
            return;
        }
    }

    if command_input == "list" && resource_input == "endpoints"{
        if let Some(endpoints) = new_os.connection.endpoints{
            let return_object = json!({
                "endpoints": endpoints
            });
            print_value(&return_object, format);
            return;
        }
        else{
            let return_object = json!({
                "error": "endpoints are not available"
            });
            print_value(&return_object, format);
            return;
        }
    }

    if !new_os.is_resource_available(resource_input.into()){
        print_value(&json!({"error": format!("endpoint for resource '{}' is not available", resource_input)}), format);
        return ()
    };

    let outcome = match new_os.act(command_input.to_string(), resource_input.to_string(), &command_options, &resource_options){
        Ok(x) => x,
        Err(e) => return print_value(&json!({"error": format!("{}", e)}), format)
    };

    // println!("{}", serde_json::to_string_pretty(&outcome).unwrap());
    print_value(&outcome, format);

}


fn prepare_app<'a>(app: App<'a, 'a>) -> App<'a, 'a>{
    app.version(&crate_version!()[..])
        .arg(
            Arg::with_name("format")
                .short("f")
                .help("formats output with this format")
                .takes_value(true)
                .global(true)
                .possible_values(&["json", "csv", "table"])
                .default_value("json")
        )
        .arg(Arg::with_name("verbose")
                .help("increases the verbosity")
                .short("v")
                .long("verbosity")
                .multiple(true)
        )
        .arg(Arg::with_name("use-cache")
                .help("try to use cache, but when that fails use normal api calls")
                .long("use-cache")
        )
        .subcommand(
            SubCommand::with_name("generate-autocomplete")
                .about("generates autocompletion scripts")
                .arg(Arg::with_name("shell")
                    .possible_values(&Shell::variants())
                    .help("which shell to generate script for"))
        )
}

fn set_log_level(matches: &clap::ArgMatches){
    let log_level = match matches.occurrences_of("verbose"){
        0 => log::Level::Error,
        1 => log::Level::Info,
        2 => log::Level::Debug,
        _ => log::Level::Trace
    };
    simple_logger::init_with_level(log_level).expect("unable to set logger");
}


fn print_error(err: OpenstackError, format: &str){
    print_value(&json!({"error": format!("{}", err)}), format)
}