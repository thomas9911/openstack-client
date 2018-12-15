#[macro_use]
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;

extern crate argparse;
extern crate secstr;
extern crate rpassword;

mod enums;
mod utils;
mod openstack_connection;

use std::collections::HashMap;

use std::env;
use std::io::{stdout, stderr, Error, ErrorKind};

use argparse::{ArgumentParser, StoreTrue, Store, List};

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
    // let os = match openstack::Cloud::from_env(){
    //     Ok(x) => x,
    //     Err(e) =>  {
    //         println!("Error: {}", e);
    //         std::process::exit(0);
    //         },
    // };

    let new_os = OpenstackConnection::new();
    args.insert(0, format!("{} {:?}", "openstack", command));

    match command{
        OSOperation::List => list_command(new_os, args),
        OSOperation::Show => show_command(new_os, args),
        OSOperation::New => new_command(new_os, args),
        OSOperation::Delete => delete_command(new_os, args),
        OSOperation::None => (),
    }

    let mut os = match OpenstackInfoMap::from_clouds_yaml("".to_string()){
        Ok(x) => x,
        // Err(e) => {println!("{}", e); return ()}
        Err(_e) => OpenstackInfoMap::default()
    };
    os.add_password_if_not_existing().unwrap();
    println!("{}", serde_json::to_string_pretty(&os).unwrap());
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
    // os.print_list(resource);
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
    // os.print_get(resource, name_or_id)
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

    // os.print_delete(resource, name_or_id)
}


#[derive(Debug, Serialize, Deserialize)]
struct OpenstackInfoMap{
    pub cloud_name: String,
    pub auth_url: String,
    pub username: String,
    pub password: secstr::SecStr,
    pub project_id: String,
    pub project_domain_id: String,
    pub user_domain_id: String,
}

impl OpenstackInfoMap{
    pub fn new(cloud_name: String,
    auth_url: String,
    username: String,
    password: String,
    project_id: String,
    project_domain_id: String,
    user_domain_id: String) -> OpenstackInfoMap{
        let ps: secstr::SecStr = secstr::SecStr::from(password);
        OpenstackInfoMap{cloud_name, auth_url, username, password: ps, project_id, project_domain_id, user_domain_id}
    }

    pub fn from_clouds_yaml(region: String) -> Result<OpenstackInfoMap, Error>{
        OpenstackInfoMap::from_yaml("clouds.yaml".to_string(), region)
    }

    pub fn from_yaml(location: String, region: String) -> Result<OpenstackInfoMap, Error>{
        let mut region_copy = region.clone();
        let value = match OpenstackInfoMap::read_yaml(location){
            Ok(x) => x,
            Err(e) => return Err(e)
        };
        if &region_copy == ""{
            let lengt = match &value["clouds"]{
                serde_yaml::Value::Mapping(x) => x.len(),
                _ => 0
            };
            if lengt != 1 {
                return Err(Error::new(ErrorKind::InvalidData, "please, choose the cloud you want to use"))
            };
            region_copy = match &value["clouds"]{
                serde_yaml::Value::Mapping(x) => match x.iter().next().unwrap().0.as_str(){
                    Some(x) => x.to_string(),
                    None =>  "".to_string()
                },
                _ => "".to_string()
            };
            if &region_copy == ""{
                return Err(Error::new(ErrorKind::InvalidData, "invalid clouds.yaml"))
            };
        };
        let auth_map: &serde_yaml::Value = &value["clouds"][&region_copy]["auth"];
        let serde_yaml_string = serde_yaml::Value::String("".to_string());
        let cloud_name: String = region_copy;
        let auth_url: String = auth_map.get("auth_url").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let username: String = auth_map.get("username").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let password: String = auth_map.get("password").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let project_id: String = auth_map.get("project_id").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let project_domain_id: String = auth_map.get("project_domain_id").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let user_domain_id: String = auth_map.get("user_domain_id").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();

        Ok(OpenstackInfoMap::new(cloud_name, auth_url, username, password, project_id, project_domain_id, user_domain_id))
    }

    pub fn from_env(region: String) -> OpenstackInfoMap{
        let mut cloud_name = region.clone();
        if region == ""{
            cloud_name = std::env::var("OS_CLOUD").unwrap_or("".to_string());
        }
        let auth_url: String = std::env::var("OS_AUTH_URL").unwrap_or("".to_string());
        let username: String = std::env::var("OS_USERNAME").unwrap_or("".to_string());
        let password: String = std::env::var("OS_PASSWORD").unwrap_or("".to_string());
        let project_id: String = std::env::var("OS_PROJECT_ID").unwrap_or("".to_string());
        let project_domain_id: String = std::env::var("OS_PROJECT_DOMAIN_ID").unwrap_or("".to_string());
        let user_domain_id: String = std::env::var("OS_USER_DOMAIN_ID").unwrap_or("".to_string());

        OpenstackInfoMap::new(cloud_name, auth_url, username, password, project_id, project_domain_id, user_domain_id)
    }

    pub fn add_password(&mut self) -> Result<&mut Self, Error>{
        let ps = match rpassword::prompt_password_stdout("Openstack password: "){
            Ok(x) => x,
            Err(e) => return Err(e)
        };
        self.password = ps.into();
        Ok(self)
    }

    pub fn add_password_if_not_existing(&mut self) -> Result<&mut Self, Error>{
        if self.password == "".to_string().into(){
            return self.add_password()
        }
        Ok(self)
    }

    fn read_yaml(location: String) -> Result<serde_yaml::Value, Error>{
        let file = match std::fs::File::open(&location){
            Ok(x) => x,
            Err(e) => return Err(e)
        };
        let value: serde_yaml::Value = match serde_yaml::from_reader(file){
            Ok(x) => x,
            Err(e) => {return Err(Error::new(ErrorKind::NotFound, e.to_string()))}
        };
        Ok(value)
    }
}

impl Default for OpenstackInfoMap {
    fn default() -> OpenstackInfoMap{
        OpenstackInfoMap{
            cloud_name: String::from(""),
            auth_url: String::from(""),
            username: String::from(""),
            password: String::from("").into(),
            project_id: String::from(""),
            project_domain_id: String::from(""),
            user_domain_id: String::from("")}
    }
}