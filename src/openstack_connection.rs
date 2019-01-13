
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

use chrono::prelude::*;
use chrono::Duration;

use enums::OSOperation;
use structs::{ResourceMap, ResourceTypeEnum, Resource};
use utils::{add_slash, read_yaml, get_first_value_from_hashmap_with_vec, make_hashmaps_from_dot_notation};


#[derive(Debug, Serialize, Deserialize)]
pub struct OpenstackConnection{
    pub config: OpenstackInfoMap,
    #[serde(skip, default = "reqwest::Client::new")]
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

    #[allow(dead_code)]
    pub fn get<T: reqwest::IntoUrl>(&mut self, url: T) -> reqwest::RequestBuilder{
        self.request(reqwest::Method::GET, url)
    }

    #[allow(dead_code)]
    pub fn post<T: reqwest::IntoUrl>(&mut self, url: T) -> reqwest::RequestBuilder{
        self.request(reqwest::Method::POST, url)
    }

    pub fn request<T: reqwest::IntoUrl>(&mut self, method: reqwest::Method, url: T) -> reqwest::RequestBuilder{
        let mut headers = reqwest::header::HeaderMap::new();

        let expire_time: DateTime<Utc> = match &self.token_expiry{
            Some(x) => DateTime::parse_from_rfc3339(&x).unwrap().with_timezone(&Utc),
            _ => DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
        };

        if expire_time - Duration::minutes(5) <= Utc::now(){
            &self.refresh_token().expect("error while refreshing token");
        }

        let token = self.token.clone();
        headers.insert("X-Auth-Token", token.expect("a valid token").parse().unwrap());
        self.client.request(method, url).headers(headers)
    }


    pub fn refresh_token(&mut self) -> Result<(), Error>{
        let password = match String::from_utf8(self.config.password.unsecure().to_vec()){
            Ok(x) => x,
            Err(_e) => return Err(Error::new(ErrorKind::InvalidData, "binary must be in utf-8"))
        };
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
        let auth_url = match reqwest::Url::parse(&format!("{}/", self.config.auth_url)){
            Ok(x) => x.join("auth/tokens").unwrap(),
            Err(_e) => return Err(Error::new(ErrorKind::InvalidData, "Not a valid auth_url"))
        };
        let mut response = match self.client.post(auth_url).json(&body).send(){
            Ok(x) => x,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string()))
        };

        let response_json = match response.json::<serde_json::Value>(){
            Ok(x) => x,
            Err(_e) => return Err(Error::new(ErrorKind::InvalidData, "Response is not valid json"))
        };

        if !response.status().is_success(){
            return Err(Error::new(ErrorKind::Other, serde_json::to_string(&response_json).unwrap()))
        }
        self.parse_token_reponse(&response);
        self.parse_identity_reponse(&response_json);
        self.parse_token_expiry_reponse(&response_json);
        Ok(())
    }

    fn parse_identity_reponse(&mut self, data: &serde_json::Value){
        let mut endpoints: HashMap<String, String> = HashMap::new();

        match data["token"]["catalog"].as_array(){
            Some(catalog) => {
                'outer: for service in catalog.iter(){
                    let the_type: String = match service["type"].as_str(){
                        Some(x) => x.into(),
                        None => continue
                    };
                    match service["endpoints"].as_array(){
                        Some(data_endpoints) => {
                            'inner: for endpoint in data_endpoints{

                                let inferface_end: String = endpoint["inferface"].as_str().unwrap_or("public").into();
                                let region_name_end: String = match endpoint["region_id"].as_str(){
                                    Some(x) => x.into(),
                                    None => continue 'inner
                                };
                                let url_end: String = match endpoint["url"].as_str(){
                                    Some(x) => x.into(),
                                    None => continue 'inner
                                };
                                if self.config.only_use_public_endpoints{
                                    match reqwest::Url::parse(&url_end).unwrap().port(){
                                            Some(_x) => continue 'inner,
                                            _ => ()
                                    };
                                }
                                if (self.config.region_name == region_name_end) & (self.config.interface == inferface_end){
                                    endpoints.insert(the_type.clone(), add_slash(&url_end.clone()));
                                }
                            }
                        },
                        None => ()
                    }
                }
            },
            None => ()
        }
        self.endpoints = Some(endpoints);
    }

    fn parse_token_reponse(&mut self, data: &reqwest::Response){
        let os_token = data.headers().get("x-subject-token").expect("Expected an auth token").to_str().unwrap();
        self.token = Some(os_token.into());
    }

    fn parse_token_expiry_reponse(&mut self, data: &serde_json::Value){
        let expiry: String = match data["token"]["expires_at"].as_str(){
            Some(x) => x.into(),
            _ => return ()
        };
        self.token_expiry = Some(expiry);
    }

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Openstack{
    pub connection: OpenstackConnection,
    pub resources: ResourceMap
}

impl Openstack{
    pub fn new(config: OpenstackInfoMap) -> Result<Self, Error>{
        let mut connection = OpenstackConnection::new(config);
        match connection.refresh_token(){
            Ok(x) => x,
            Err(e) => return Err(e)
        };
        let mut rc = ResourceMap::new();
        if let Some(x) = &connection.endpoints{
            rc.update_from_identity(x)
        };
        Ok(Openstack{connection, resources: rc})
    }

    pub fn refresh_token(&mut self) -> Result<&mut Openstack, Error>{
        match self.connection.refresh_token(){
            Ok(x) => x,
            Err(e) => return Err(e)
        };
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn list(&mut self, res: String) -> Result<serde_json::Value, Error>{
        self.act(OSOperation::List, res.clone(), &HashMap::new(), &HashMap::new())
    }

    #[allow(dead_code)]
    pub fn delete(self, res: String, id: String) {

    }

    #[allow(dead_code)]
    pub fn get(self, res: String, id: String) {

    }

    #[allow(dead_code)]
    pub fn update(self, res: String, id: String) {

    }

    pub fn act(&mut self, op: OSOperation, res: String,  op_args: &HashMap<String, Vec<String>>, res_args: &HashMap<String, Vec<String>>) -> Result<serde_json::Value, Error>{
        if self.connection.endpoints.is_none(){
            self.refresh_token().expect("error while refreshing token");
        }
        let r = match self.resources.get_resource(res){
            Ok(x) => x,
            Err(e) => return Err(e)
        };
        let path = r.endpoint_path.clone();
        let endpoint: String = match r.resource_type.clone(){
            ResourceTypeEnum::ResourceType(x) => x.endpoint,
            ResourceTypeEnum::String(x) => x,
        };

        let post_body = Openstack::handle_post_parameters(&r, res_args);

        let is_dry_run = match op_args.get("dry-run"){
            Some(_x) => true,
            None => false
        };

        if is_dry_run{
            return Ok(post_body)
        }

        let prepared_url = match get_first_value_from_hashmap_with_vec(res_args, "id"){
            Some(id) => self.connection.request(op.match_http_method(), &format!("{}{}/{}", endpoint, path, id)),
            None => self.connection.request(op.match_http_method(), &format!("{}{}", endpoint, path))
        };

        let mut response = match prepared_url.json(&post_body).send(){
            Ok(x) => x,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("{}", e)))
        };
        Openstack::handle_response(&mut response)
    }

    fn handle_response(response: &mut reqwest::Response) -> Result<serde_json::Value, Error>{
        if !response.status().is_success(){
            return Err(Error::new(ErrorKind::Other, format!("'{}' \n{{\"response\": {}}}", response.status(), response.text().unwrap())))
        }
        match response.json::<serde_json::Value>(){
            Ok(x) => return Ok(x),
            Err(_e) => ()
            // Err(_e) => return Err(Error::new(ErrorKind::InvalidData, "Response is not valid json"))
        };
        match response.text(){
            Ok(x) => Ok(x.into()),
            Err(_e) => Err(Error::new(ErrorKind::InvalidData, "Response cannot be parsed"))
        }
    }

    fn handle_post_parameters(res: &Resource, res_args: &HashMap<String, Vec<String>>) -> serde_json::Value{
        if let Some(ref post_param) = res.post_parameters{
            let mut data: Vec<(String, serde_json::Value)> = vec![];
            for item in post_param{
                let path = item.path.clone();
                if item.hidden{
                    data.push((path.clone(), Vec::<serde_json::Value>::new().into()))
                }
                if let Some(x) = res_args.get(&item.name){
                    if item.multiple{
                        data.push((path.clone(), x.clone().into()))
                    } else{
                        data.push((path.clone(), x[0].clone().into()))
                    }
                }
            }
            return make_hashmaps_from_dot_notation(data);
        };
        serde_json::Value::Null
    }

    #[allow(dead_code)]
    pub fn resource_available(&self, res: String) -> Option<Resource>{
        let available = self.is_resource_available(res.clone());

        if available{
            if let Ok(resource) = self.resources.get_resource(res){
                return Some(resource.clone())
            };
        };
        None
    }

    pub fn is_resource_available(&self, res: String) -> bool{
        let res = match self.resources.get_resource(res){
            Ok(x) => x,
            Err(_e) => return false
        };

        match &res.resource_type{
            ResourceTypeEnum::ResourceType(_x) => true,
            _ => false
        }
    }
}


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
    pub interface: String,
    pub only_use_public_endpoints: bool,
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
        OpenstackInfoMap{cloud_name, auth_url, username, password: ps, project_id, project_domain_id, user_domain_id, region_name, interface, only_use_public_endpoints: true}
    }

    pub fn from_clouds_yaml(region: String) -> Result<OpenstackInfoMap, Error>{
        OpenstackInfoMap::from_yaml("clouds.yaml".to_string(), region)
    }

    pub fn from_yaml(location: String, region: String) -> Result<OpenstackInfoMap, Error>{
        let mut region_copy = region.clone();
        let value = match read_yaml(location){
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
            interface: String::from("public"),
            only_use_public_endpoints: true}
    }
}


