
// use std::collections::HashMap;
// use serde_json::{Value, to_string_pretty};

// use std::fs::File;
use std::io::{stdout, stderr, Error, ErrorKind};
use std::collections::HashMap;

use enums::OSResource;

#[derive(Debug)]
pub struct OpenstackConnection{
    pub config: OpenstackInfoMap,
    pub client: reqwest::Client,
    pub token: Option<String>,
    pub token_expiry: Option<String>,
    pub endpoints: Option<HashMap<String, String>>
}

impl OpenstackConnection{
    pub fn new(config: OpenstackInfoMap) -> OpenstackConnection{
        let client = reqwest::Client::new();
        OpenstackConnection{config, client, token: None, token_expiry: None, endpoints: None}
    }

    pub fn refresh_token(&mut self){
        let password = String::from_utf8(self.config.password.unsecure().to_vec()).expect("binary must be in utf-8");
        let body = json!({
            "auth": {
                "identity": {
                    "methods": ["password"],
                    "password": {
                        "user": {
                            "name": format!("{}", self.config.username),
                            "domain": {
                                "id": format!("{}", self.config.user_domain_id)
                            },
                            "password": format!("{}", password)
                        }
                    }
                },
                "scope": {
                    "project": {
                        "id": format!("{}", self.config.project_id),
                        "domain": {
                            "id": format!("{}", self.config.project_domain_id)
                            }
                        }
                    }
                }
            });
        let auth_url = reqwest::Url::parse(&format!("{}/", self.config.auth_url)).expect("Not a valid auth_url").join("auth/tokens").unwrap();
        let mut response = self.client.post(auth_url).json(&body).send().unwrap();
        // println!("{:?}", response.json::<serde_json::Value>().unwrap());

        println!("{}", serde_json::to_string_pretty(&response.json::<serde_json::Value>().unwrap()).unwrap());
        let os_token = response.headers().get("x-subject-token").expect("Expected an auth token").to_str().unwrap();
        println!("{}", os_token);

    }
}

pub struct Openstack{
    connection: OpenstackConnection
}

impl Openstack{
    pub fn delete(self, res: OSResource) {

    }
    pub fn list(self, res: OSResource) {

    }
    pub fn get(self, res: OSResource, id: String) {

    }
    pub fn update(self, res: OSResource, id: String) {

    }
}

// impl OpenstackConnection{
//     pub fn new(os_client: openstack::Cloud) -> OpenstackConnection{
//         OpenstackConnection{client: os_client}
//     }
//     pub fn print_list(&self, resource: OSResource){
//         match resource{
//             OSResource::Flavors => print_flavor_summary_data(self.client.list_flavors()),
//             OSResource::FloatingIps => print_floating_ip_data(self.client.list_floating_ips()),
//             OSResource::Images => print_image_data(self.client.list_images()),
//             OSResource::Keypairs => print_key_pair_data(self.client.list_keypairs()),
//             OSResource::Networks => print_network_data(self.client.list_networks()),
//             OSResource::Servers => print_server_summary_data(self.client.list_servers()),
//             OSResource::Subnets => print_subnet_data(self.client.list_subnets()),
//             OSResource::Ports => print_port_data(self.client.list_ports()),
//             OSResource::None => println!("[{{\"error\": \"resource cannot be listed\"}}]"),
//         }
//     }

//     pub fn print_get(&self, resource: OSResource, name: String){
//         if name == ""{
//             println!("{}", to_string_pretty(
//                     &fmt_error("'name or id' is a required argument")
//                 ).unwrap());
//             return
//         }
//         let result = match resource{
//             OSResource::Flavors => {
//                 fmt_flavor(self.client.get_flavor(name))
//             },
//             // OSResource::FloatingIps => print_floating_ip_data(self.client.list_floating_ips()),
//             // OSResource::Images => print_image_data(self.client.list_images()),
//             OSResource::Keypairs => {
//                  match self.client.get_keypair(name){
//                      Ok(x) => fmt_key_pair(x),
//                      Err(x) => fmt_error(x)
//                  }
//             },

//             // OSResource::Networks => print_network_data(self.client.list_networks()),
//             OSResource::Servers => {
//                  match self.client.get_server(name){
//                      Ok(x) => fmt_server(x),
//                      Err(x) => fmt_error(x)
//                  }
//             },
//             // OSResource::Subnets => print_subnet_data(self.client.list_subnets()),
//             // OSResource::Ports => print_port_data(self.client.list_ports()),
//             // OSResource::None => json!([{"error": "resource cannot be showed"}]),
//             _ => fmt_error("resource cannot be showed"),
//         };
//         println!("{}", to_string_pretty(&result).unwrap());
//     }

//     pub fn print_delete(&self, resource: OSResource, name: String){
//         if name == ""{
//             println!("{}", to_string_pretty(
//                     &fmt_error("'name or id' is a required argument")
//                 ).unwrap());
//             return
//         }

//         let result = match resource{
//             // OSResource::Flavors => print_flavor_summary_data(self.client.list_flavors()),
//             // OSResource::FloatingIps => print_floating_ip_data(self.client.list_floating_ips()),
//             // OSResource::Images => print_image_data(self.client.list_images()),
//             OSResource::Keypairs => {
//                  match self.client.get_keypair(name){
//                      Ok(x) => {match x.delete(){
//                          Ok(_x) => json!({"info": "keypair deleted"}),
//                          Err(x) => fmt_error(x)
//                      }},
//                      Err(x) => fmt_error(x)
//                  }
//             },

//             // OSResource::Networks => print_network_data(self.client.list_networks()),
//             // OSResource::Servers => print_server_summary_data(self.client.list_servers()),
//             // OSResource::Subnets => print_subnet_data(self.client.list_subnets()),
//             // OSResource::Ports => print_port_data(self.client.list_ports()),
//             // OSResource::None => json!([{"error": "resource cannot be showed"}]),
//             _ => fmt_error("resource cannot be deleted"),
//         };
//         println!("{}", to_string_pretty(&result).unwrap());
//     }

//     pub fn create_keypair(&self, options: HashMap<String, String>) -> Result<Value, String>{
//         let name = match options.get("name"){
//             Some(x) => x.to_owned(),
//             _ => return Err("name is required".to_string())
//         };

//         let public_key = match options.get("pk"){
//             Some(x) => x.to_owned(),
//             _ => return Err("public-key is required".to_string())
//         };

//         let mut file = match File::open(public_key.clone()){
//             Ok(x) => x,
//             _ => return Err(format!("'{}' file is not available", public_key))
//         };

//         let return_value = match self.client.new_keypair(name)
//                     .from_reader(&mut file).expect("file reading goes wrong")
//                     .create(){
//                 Ok(x) => fmt_key_pair(x),
//                 _ => Value::from("Something went wrong while creating a keypair")
//         };
//         Ok(return_value)
//     }

//     pub fn create_server(&self, options: HashMap<String, String>) -> Result<Value, String>{
//         let name = match options.get("name"){
//             Some(x) => x.to_owned(),
//             _ => return Err("name is required".to_string())
//         };

//         let flavor_name = match options.get("flavor"){
//             Some(x) => x.to_owned(),
//             _ => return Err("flavor is required".to_string())
//         };

//         let image_name = match options.get("image"){
//             Some(x) => x.to_owned(),
//             _ => return Err("image is required".to_string())
//         };

//         let keypair_name = match options.get("keypair"){
//             Some(x) => x.to_owned(),
//             _ => return Err("keypair is required".to_string())
//         };

//         let network_name = match options.get("network"){
//             Some(x) => x.to_owned(),
//             _ => return Err("network is required".to_string())
//         };


//         let flavor = match self.client.get_flavor(flavor_name) {
//             Ok(x) => x,
//             Err(x) => return Err(x.to_string())
//         };

//         let image = match find_images(image_name, self.client.list_images()) {
//             Ok(x) => x,
//             Err(x) => return Err(x.to_string())
//         };

//         let keypair = match self.client.get_keypair(keypair_name) {
//             Ok(x) => x,
//             Err(x) => return Err(x.to_string())
//         };

//         let network = match self.client.get_network(network_name){
//             Ok(x) => x,
//             Err(x) => return Err(x.to_string())
//         };

//         let return_value = match self.client.new_server(name, flavor)
//                     .with_image(image)
//                     .with_keypair(keypair)
//                     .with_network(network)
//                     .create(){
//                 Ok(mut x) => {match x.poll(){
//                     Ok(x) => fmt_server(x.unwrap()),
//                     Err(x) => fmt_error(x.to_string())
//                 }},
//                 // Ok(mut x) => {match x.wait(){
//                 //     Ok(x) => fmt_server(x),
//                 //     Err(x) => fmt_error(x.to_string())
//                 // }},
//                 Err(e) => fmt_error(e.to_string())
//         };
//         Ok(return_value)
//     }
// }


// pub fn fmt_error<S>(error: S) -> Value where S: ToString{
//     return json!({"error": error.to_string()});
// }

// fn find_images(name: String, images: openstack::Result<Vec<openstack::image::Image>>)  -> Result<openstack::image::Image, String> {
//     let legit_images = match images{
//         Ok(x) => x,
//         Err(x) => return Err(x.to_string())
//     };

//     let mut choices = vec![];
//     for image in legit_images{
//         if image.name().to_lowercase().starts_with(name.to_lowercase().as_str()){
//             choices.push(image);
//         }
//     };

//     // Ok(choices.first().unwrap())
//     match choices.first(){
//         Some(x) => Ok(x.clone()),
//         _ => Err("No valid image found".to_string())
//     }
// }


#[derive(Debug, Serialize, Deserialize)]
pub struct OpenstackInfoMap{
    pub cloud_name: String,
    pub auth_url: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: secstr::SecStr,
    pub project_id: String,
    pub project_domain_id: String,
    pub user_domain_id: String,
    pub region_name: String,
    pub interface: String
}

impl OpenstackInfoMap{
    pub fn new(cloud_name: String,
    auth_url: String,
    username: String,
    password: String,
    project_id: String,
    project_domain_id: String,
    user_domain_id: String,
    region_name: String,
    interface: String) -> OpenstackInfoMap{
        let ps: secstr::SecStr = secstr::SecStr::from(password);
        OpenstackInfoMap{cloud_name, auth_url, username, password: ps, project_id, project_domain_id, user_domain_id, region_name, interface}
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
        if value["clouds"].get(&region_copy).is_none(){
            return Err(Error::new(ErrorKind::InvalidData, "please, choose the cloud you want to use"))
        }
        let auth_map: &serde_yaml::Value = &value["clouds"][&region_copy]["auth"];
        let serde_yaml_string = serde_yaml::Value::String("".to_string());
        let cloud_name: String = region_copy.clone();
        let auth_url: String = auth_map.get("auth_url").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let username: String = auth_map.get("username").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let password: String = auth_map.get("password").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let project_id: String = auth_map.get("project_id").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let project_domain_id: String = auth_map.get("project_domain_id").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let user_domain_id: String = auth_map.get("user_domain_id").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let extra_map: &serde_yaml::Value = &value["clouds"][&region_copy];
        let region_name: String = extra_map.get("region_name").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        let interface: String = extra_map.get("interface").unwrap_or(&serde_yaml_string).as_str().unwrap().to_string();
        Ok(OpenstackInfoMap::new(cloud_name, auth_url, username, password, project_id, project_domain_id, user_domain_id, region_name, interface))
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
        let region_name: String = std::env::var("OS_REGION_NAME").unwrap_or("".to_string());
        let interface: String = std::env::var("OS_INTERFACE").unwrap_or("".to_string());

        OpenstackInfoMap::new(cloud_name, auth_url, username, password, project_id, project_domain_id, user_domain_id, region_name, interface)
    }

    pub fn add_password(&mut self) -> Result<&mut Self, Error>{
        let ps = match rpassword::prompt_password_stdout(&format!("Openstack password for user '{}': ", self.username)){
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

    pub fn apply(&mut self, other: &Self) -> &mut Self{
        if other.password != "".into(){
            self.password = other.password.clone()
        };
        if other.username != ""{
            self.username = other.username.clone()
        };
        if other.cloud_name != ""{
            self.cloud_name = other.cloud_name.clone()
        };
        if other.auth_url != ""{
            self.auth_url = other.auth_url.clone()
        };
        if other.project_id != ""{
            self.project_id = other.project_id.clone()
        };
        if other.project_domain_id != ""{
            self.project_domain_id = other.project_domain_id.clone()
        };
        if other.user_domain_id != ""{
            self.user_domain_id = other.user_domain_id.clone()
        };
        if other.region_name != ""{
            self.region_name = other.region_name.clone()
        };
        if other.interface != ""{
            self.interface = other.interface.clone()
        };
        self
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
            user_domain_id: String::from(""),
            region_name: String::from(""),
            interface: String::from("public")}
    }
}
