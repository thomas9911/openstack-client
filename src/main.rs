extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;
// #[macro_use]
// extern crate structopt_derive;
extern crate argparse;
#[macro_use]
extern crate clap;
extern crate secstr;
extern crate reqwest;
extern crate rpassword;
extern crate chrono;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate heck;
extern crate yaml_rust;

mod enums;
mod utils;
mod openstack_connection;
mod structs;

use std::collections::HashMap;

use std::env;
use std::io::{stdout, stderr, Error, ErrorKind};

use argparse::{ArgumentParser, StoreTrue, Store, List};
use clap::{Arg, App, SubCommand};
use structopt::StructOpt;
use std::string::ToString;

use enums::{OSOperation, OSResource};
use openstack_connection::{OpenstackConnection, OpenstackInfoMap, Openstack};
use utils::{add_if_exists, read_yaml_rust, get_first_value_from_hashmap_with_vec};
use structs::ResourceType;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "snake_case")]
struct Opt{
    #[structopt(parse(from_str))]
    command: OSOperation,
    // #[structopt(subcommand)]
    // resource: OSResource
    // #[structopt(parse(try_from_str))]
    resource: String
}

// #[derive(StructOpt, Debug)]
// #[structopt()]
// enum Sub{
//     OSResource
// }


// #[derive(StructOpt, Debug)]
// struct Opt{
//     #[structopt(parse(from_str))]
//     command: OSOperation,
//     #[structopt(subcommand)]
//     blub: Sub
// }

// #[derive(StructOpt, Debug)]
// enum Sub{
//     #[structopt(parse(from_str), name = "finish")]
//     OSResource(OSResource)
// }

// #[derive(StructOpt, Debug)]
// enum Command{
//     #[structopt(name = "resource")]
//     Oke(Oke)
// }

// #[derive(StructOpt, Debug)]
// struct Oke {
//     #[structopt(short = "t")]
//     time: u32,
//     #[structopt(subcommand)]
//     resource: OSResource,
// }


fn main() {

    // println!("{:?}", OSResource::list());
    let mut os_cloud: String = "".to_string();
    let mut command: OSOperation = OSOperation::None;
    let mut args: Vec<String> = vec!();
    if env::var("OS_CLOUD").is_ok(){
        os_cloud = env::var("OS_CLOUD").unwrap()
    }
    // let matches = App::new("WOW")
    //                     .arg(Arg::with_name("command")
    //                         .help("Sets the input file to use")
    //                         .required(true)
    //                         .index(1))
    //                     .arg(Arg::with_name("v")
    //                         .short("v")
    //                         .multiple(true)
    //                         .help("Sets the level of verbosity"))
    //                     .arg(Arg::with_name("resource")
    //                         .help("Sets the input file to use")
    //                         .required(true)
    //                         .index(2))
    //                     .arg(Arg::with_name("c")
    //                         .short("c")
    //                         .multiple(true)
    //                         .help("Sets the level of verbosity"))
    //                     .get_matches();

    let yml = load_yaml!("../data/cool.yaml");
    let app = App::from_yaml(&yml);
    let matches = app.get_matches();
    // println!("{:?}", matches);
    // let matches = Opt::clap().get_matches();
    // let resource_input: String = match matches.subcommand_name(){
    //     Some(x) => x.to_string(),
    //     _ => {println!("Resource is required"); return}
    // };

    let (command_input, command_sub) = match matches.subcommand(){
        (x, Some(y)) => (x, y),
        (_x, None) => return ()
    };
    let (resource_input, resource_sub) = match command_sub.subcommand(){
        (x, Some(y)) => (x, y),
        (_x, None) => return ()
    };


    // {
    //     let mut ap = ArgumentParser::new();
    //     ap.set_description("Openstack cli in Rust maddafaka");
    //     ap.refer(&mut os_cloud)
    //         .add_option(&["--os-cloud"], Store,
    //         "cloud name from the clouds.yaml");
    //     ap.refer(&mut command)
    //         .add_argument("command", Store,
    //                     "Command to run");
    //      ap.refer(&mut args)
    //         .add_argument("arguments", List,
    //                 r#"Arguments for command"#);

    //     ap.stop_on_first_argument(true);
    //     ap.parse_args_or_exit();
    // }


    // println!("{:?}", command_sub);
    // println!("{:?}", resource_sub);

    // let mut matches_options = HashMap::new();
    // for (k, v) in matches.args.iter(){
    //     matches_options.insert(k.clone(), v.vals.clone());
    // }

    // let mut command_options = HashMap::new();
    // for (k, v) in command_sub.args.iter(){
    //     command_options.insert(k.clone(), v.vals.clone());
    // }

    // let mut resource_options = HashMap::new();
    // for (k, v) in resource_sub.args.iter(){
    //     resource_options.insert(k.clone(), v.vals.clone());
    // }

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
        // Err(e) => {println!("{}", e); return ()}
        Err(_e) => OpenstackInfoMap::default()
    };
    os_config.apply(&os_config_env)
      .add_password_if_not_existing().unwrap();

    // println!("{}", serde_json::to_string_pretty(&os_config).unwrap());

    // let mut new_os = OpenstackConnection::new(os_config);
    // match new_os.refresh_token(){
    //     Ok(x) => x,
    //     Err(e) => {println!("{}", e); return}
    // };

    // println!("{:?}", new_os.token);
    // println!("{:?}", new_os.endpoints);

    // println!("{}", serde_json::to_string_pretty(&new_os).unwrap());
    let mut new_os = match Openstack::new(os_config){
        Ok(x) => x,
        Err(e) => {println!("{}", e); return}
    };

    let os_command = OSOperation::from(command_input);

    // println!("{}", os_command);

    if !new_os.is_resource_available(resource_input.into()){
        println!("error: endpoint for resource '{}' is not available", resource_input);
        return ()
    };

    // new_os.act(os_command, resource_input.into(), HashMap::new());

    let outcome = match new_os.act(os_command, resource_input.to_string(), &command_options, &resource_options){
        Ok(x) => x,
        Err(e) => {println!("{}", e); return}
    };

    println!("{}", serde_json::to_string_pretty(&outcome).unwrap());


    // println!("{}", serde_json::to_string_pretty(&new_os).unwrap());
    // println!("{:?}", new_os.get("https://compute.api.ams.fuga.cloud:443/v2.1/5af86bc2f74c49178f32f6f479e878cc/servers").send().unwrap().json::<serde_json::Value>().unwrap());

    // new_os.connection.request(reqwest::Method::GET, "https://google.com");
    // let cool: OSResource = "keypairs".into();
    // let cool: String = "keypairs".into();

    // println!("{}", cool);
    // let outcome = match new_os.list(resource_input.to_string()){
    //     Ok(x) => x,
    //     Err(e) => {println!("{}", e); return}
    // };

    // println!("{}", serde_json::to_string_pretty(&outcome).unwrap());
    // let mut a: HashMap<String, structs::ResourceTypeEnum> = HashMap::new();
    // a.insert("compute".into(), structs::ResourceTypeEnum::String("compute".to_string()));
    // a.insert("volumev3".into(), structs::ResourceTypeEnum::String("volumev3".to_string()));
    // a.insert("image".into(), structs::ResourceTypeEnum::String("image".to_string()));
    // a.insert("metric".into(), structs::ResourceTypeEnum::String("metric".to_string()));
    // println!("{:?}", structs::ResourceMap::new());




    // args.insert(0, format!("{} {:?}", "openstack", command));

    // match command{
    //     OSOperation::List => list_command(&mut new_os, args),
    //     OSOperation::Show => show_command(new_os, args),
    //     OSOperation::New => new_command(new_os, args),
    //     OSOperation::Delete => delete_command(new_os, args),
    //     OSOperation::None => (),
    //     _ => (),
    // }
}

fn list_command(os: &mut Openstack, args: Vec<String>){
    let mut resource: String = "".into();
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
    // os.print_list(resource);
    let outcome = match os.list(resource){
        Ok(x) => x,
        Err(e) => {println!("{}", e); return}
    };
    println!("{}", serde_json::to_string_pretty(&outcome).unwrap());
}

fn show_command(os: Openstack, args: Vec<String>){
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
    // os.print_get(resource, name_or_id)
}

fn new_command(os: Openstack, args: Vec<String>){
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


    // let txt = match resource{
    //     OSResource::Keypairs => os.create_keypair(hashmap),
    //     OSResource::Servers => os.create_server(hashmap),
    //     _ => Err("Resource cannot be created".to_string())
    // };
    // match txt{
    //     Ok(x) => println!("{}", serde_json::to_string_pretty(&x).unwrap()),
    //     Err(x) => println!("ERROR: {}", x),
    // };
}

fn delete_command(os: Openstack, args: Vec<String>){
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

    // os.print_delete(resource, name_or_id)
}


fn make_args_from_arg_matches(matches: &clap::ArgMatches) -> HashMap<String, Vec<String>>{
    let mut options: HashMap<String, Vec<String>> = HashMap::new();
    for (k, v) in matches.args.iter(){
        options.insert(k.to_string(), v.vals.iter().map(|x| x.clone().into_string().unwrap()).collect());
    };
    options
}