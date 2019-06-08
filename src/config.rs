use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::io::{Error, ErrorKind, Read, Write};
use std::hash::{Hash, Hasher};

use client::{Client, Response};
use utils::{
    add_slash, get_first_value_from_hashmap_with_vec, hashmap_with_vec_to_json,
    make_hashmaps_from_dot_notation, read_yaml, remove_slash_start,
};
use traits::SerdeList;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenstackTokenizer {
    pub config: OpenstackInfoMap,
    pub token: Option<String>,
    pub token_expiry: Option<String>,
    pub endpoints: Option<HashMap<String, String>>,
    pub domain_id: Option<String>,
    pub user_id: Option<String>,
}

impl OpenstackTokenizer {
    pub fn new(config: OpenstackInfoMap) -> OpenstackTokenizer {
        OpenstackTokenizer {
            config,
            token: None,
            token_expiry: None,
            endpoints: None,
            domain_id: None,
            user_id: None,
        }
    }

    pub fn from_cache_or_new(config: OpenstackInfoMap) -> Self {
        match Self::from_cache(&config){
            Ok(x) => return x,
            Err(_x) => ()
        };
        Self::new(config)
    }

    // from_cache
    // - from reader

    pub fn from_cache(config: &OpenstackInfoMap) -> Result<Self, Error> {
        let dir = Self::get_tmp_cache_location(config);
        debug!("using cache from {:?}", dir);
        let file = std::fs::File::open(dir)?;
        let reader = std::io::BufReader::new(file);
        let obj = Self::from_reader(reader)?;
        Ok(obj)
    }

    pub fn from_reader<R>(reader: R) -> Result<Self, Error>
        where R: Read
    {
        let obj = serde_json::from_reader(reader)?;
        Ok(obj)
    }

    // to_cache
    // - to writer

    pub fn to_cache(&self) -> Result<(), Error>{
        let dir = Self::get_tmp_cache_location(&self.config);
        let file = std::fs::File::create(dir)?;
        let writer = std::io::BufWriter::new(file);
        self.to_writer(writer)?;
        Ok(())
    }

    pub fn to_writer<W>(&self, writer: W) -> Result<(), Error>
        where W: Write
    {
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    fn get_tmp_cache_location(config: &OpenstackInfoMap) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let filename_hash = format!("openstack-client-{}", config.auth.create_hash());
        dir.push(filename_hash);
        dir
    }

    pub fn refresh_token(&mut self) -> Result<(), Error> {
        // let body = create_token_body(&self.config.auth);
        let body = self.config.auth.pick_token_body();

        let auth_url = match url::Url::parse(&add_slash(&self.config.auth.auth_url)) {
            Ok(x) => x.join("auth/tokens").unwrap(),
            Err(_e) => return Err(Error::new(ErrorKind::InvalidData, "Not a valid auth_url")),
        };
        let mut client = Client::new();
        let mut response = match client.post(auth_url.as_str(), body) {
            Ok(x) => x,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        let response_json = response.response();

        if !response.is_success() {
            return Err(Error::new(
                ErrorKind::Other,
                serde_json::to_string(&response_json).unwrap(),
            ));
        }
        self.parse_token_reponse(&response);
        self.parse_identity_reponse(&response_json);
        self.parse_token_expiry_reponse(&response_json);
        self.parse_token_user_domain_reponse(&response_json);
        Ok(())
    }

    fn parse_identity_reponse(&mut self, data: &serde_json::Value) {
        let mut endpoints: HashMap<String, String> = HashMap::new();
        // println!("{}", serde_json::to_string_pretty(data).unwrap());

        match data["token"]["catalog"].as_array() {
            Some(catalog) => {
                'outer: for service in catalog.iter() {
                    let the_type: String = match service["type"].as_str() {
                        Some(x) => x.into(),
                        None => continue,
                    };
                    match service["endpoints"].as_array() {
                        Some(data_endpoints) => {
                            'inner: for endpoint in data_endpoints {
                                let inferface_end: String =
                                    endpoint["inferface"].as_str().unwrap_or("public").into();
                                let region_name_end: String = match endpoint["region_id"].as_str() {
                                    Some(x) => x.into(),
                                    None => continue 'inner,
                                };
                                let url_end: String = match endpoint["url"].as_str() {
                                    Some(x) => x.into(),
                                    None => continue 'inner,
                                };
                                if self.config.only_use_public_endpoints {
                                    match url::Url::parse(&url_end).unwrap().port() {
                                        Some(_x) => continue 'inner,
                                        _ => (),
                                    };
                                }
                                if (self.config.region_name == region_name_end)
                                    & (self.config.interface == inferface_end)
                                {
                                    endpoints.insert(the_type.clone(), add_slash(&url_end.clone()));
                                }
                            }
                        }
                        None => (),
                    }
                }
            }
            None => (),
        }
        self.endpoints = Some(endpoints);
    }

    fn parse_token_reponse(&mut self, data: &Response) {
        let headers = data.clone().parsed_headers();
        let os_token = match headers.get("x-subject-token") {
            Some(x) => x,
            None => headers
                .get("X-Subject-Token")
                .expect("Expected an auth token"),
        };
        self.token = Some(os_token.to_string());
    }

    fn parse_token_expiry_reponse(&mut self, data: &serde_json::Value) {
        let expiry: String = match data["token"]["expires_at"].as_str() {
            Some(x) => x.into(),
            _ => return (),
        };
        self.token_expiry = Some(expiry);
    }

    fn parse_token_user_domain_reponse(&mut self, data: &serde_json::Value) {
        match data["token"]["user"]["id"].as_str() {
            Some(x) => self.user_id = Some(x.into()),
            _ => self.user_id = None,
        };

        match data["token"]["user"]["domain"]["id"].as_str() {
            Some(x) => self.domain_id = Some(x.into()),
            _ => self.domain_id = None,
        };
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "snake_case")]
pub struct OpenstackInfoMap {
    pub cloud_name: String,
    // pub auth_url: String,
    // pub username: String,
    // #[serde(skip_serializing)]
    // pub password: secstr::SecUtf8,
    // pub project_id: String,
    // pub project_domain_id: String,
    // pub user_domain_id: String,
    pub region_name: String,
    pub interface: String,
    pub auth: Auth,
    pub only_use_public_endpoints: bool,
}

impl OpenstackInfoMap {
    pub fn new(
        cloud_name: String,
        // auth_url: String,
        // username: String,
        // password: String,
        // project_id: String,
        // project_domain_id: String,
        // user_domain_id: String,
        region_name: String,
        interface: String,
        auth: Auth,
    ) -> OpenstackInfoMap {
        // let ps: secstr::SecUtf8 = secstr::SecUtf8::from(password);
        let only_use_public_endpoints;
        if interface == "public" {
            only_use_public_endpoints = true
        } else {
            only_use_public_endpoints = false
        }
        OpenstackInfoMap {
            cloud_name,
            // auth_url,
            // username,
            // password: ps,
            // project_id,
            // project_domain_id,
            // user_domain_id,
            region_name,
            interface,
            auth,
            only_use_public_endpoints,
        }
    }

    // pub fn from_arg_matches(matches: &clap::ArgMatches) -> OpenstackInfoMap {

    // }

    pub fn from_clouds_yaml(region: String) -> Result<OpenstackInfoMap, Error> {
        OpenstackInfoMap::from_yaml("clouds.yaml".to_string(), region)
        // current directory
        // ~/.config/openstack
        // /etc/openstack
    }

    pub fn from_yaml(location: String, region: String) -> Result<OpenstackInfoMap, Error> {
        let value = match read_yaml(location) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        Self::parse_clouds_yaml(value, region)
    }
    fn parse_clouds_yaml(
        value: serde_yaml::Value,
        region: String,
    ) -> Result<OpenstackInfoMap, Error> {
        let mut region_copy = region.clone();
        if &region_copy == "" {
            let lengt = match &value["clouds"] {
                serde_yaml::Value::Mapping(x) => x.len(),
                _ => 0,
            };
            if lengt != 1 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "please, choose the cloud you want to use",
                ));
            };
            region_copy = match &value["clouds"] {
                serde_yaml::Value::Mapping(x) => match x.iter().next().unwrap().0.as_str() {
                    Some(x) => x.to_string(),
                    None => "".to_string(),
                },
                _ => "".to_string(),
            };
            if &region_copy == "" {
                return Err(Error::new(ErrorKind::InvalidData, "invalid clouds.yaml"));
            };
        };
        if value["clouds"].get(&region_copy).is_none() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "please, choose the cloud you want to use",
            ));
        }
        let auth_map: &serde_yaml::Value = &value["clouds"][&region_copy]["auth"];
        let serde_yaml_string = serde_yaml::Value::String("".to_string());
        let cloud_name: String = region_copy.clone();
        // let auth_url: String = auth_map
        //     .get("auth_url")
        //     .unwrap_or(&serde_yaml_string)
        //     .as_str()
        //     .unwrap()
        //     .to_string();
        // let username: String = auth_map
        //     .get("username")
        //     .unwrap_or(&serde_yaml_string)
        //     .as_str()
        //     .unwrap()
        //     .to_string();
        // let password: String = auth_map
        //     .get("password")
        //     .unwrap_or(&serde_yaml_string)
        //     .as_str()
        //     .unwrap()
        //     .to_string();
        // let project_id: String = auth_map
        //     .get("project_id")
        //     .unwrap_or(&serde_yaml_string)
        //     .as_str()
        //     .unwrap()
        //     .to_string();
        // let project_domain_id: String = auth_map
        //     .get("project_domain_id")
        //     .unwrap_or(&serde_yaml_string)
        //     .as_str()
        //     .unwrap()
        //     .to_string();
        // let user_domain_id: String = auth_map
        //     .get("user_domain_id")
        //     .unwrap_or(&serde_yaml_string)
        //     .as_str()
        //     .unwrap()
        //     .to_string();
        let tmp_info_map: Auth = match serde_yaml::from_value(auth_map.clone()) {
            Ok(x) => x,
            Err(_e) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "invalid clouds.yaml format",
                ))
            }
        };
        let extra_map: &serde_yaml::Value = &value["clouds"][&region_copy];
        let region_name: String = extra_map
            .get("region_name")
            .unwrap_or(&serde_yaml_string)
            .as_str()
            .unwrap()
            .to_string();
        let interface: String = extra_map
            .get("interface")
            .unwrap_or(&serde_yaml_string)
            .as_str()
            .unwrap()
            .to_string();
        // Ok(OpenstackInfoMap {
        //     cloud_name,
        //     region_name,
        //     interface,
        //     ..tmp_info_map
        // })
        Ok(OpenstackInfoMap::new(
            cloud_name,
            region_name,
            interface,
            tmp_info_map,
        ))
    }

    pub fn from_env(region: String) -> OpenstackInfoMap {
        // use envy (crate) for deserializing env vars!!

        let mut cloud_name = region.clone();
        if region == "" {
            cloud_name = std::env::var("OS_CLOUD").unwrap_or("".to_string());
        }
        // let auth_url: String = std::env::var("OS_AUTH_URL").unwrap_or("".to_string());
        // let username: String = std::env::var("OS_USERNAME").unwrap_or("".to_string());
        // let password: String = std::env::var("OS_PASSWORD").unwrap_or("".to_string());
        // let project_id: String = std::env::var("OS_PROJECT_ID").unwrap_or("".to_string());
        // let project_domain_id: String =
        //     std::env::var("OS_PROJECT_DOMAIN_ID").unwrap_or("".to_string());
        // let user_domain_id: String = std::env::var("OS_USER_DOMAIN_ID").unwrap_or("".to_string());
        let auth = Auth::from_env();
        let region_name: String = std::env::var("OS_REGION_NAME").unwrap_or("".to_string());
        let interface: String = std::env::var("OS_INTERFACE").unwrap_or("".to_string());

        OpenstackInfoMap::new(
            cloud_name,
            region_name,
            interface,
            auth
        )
    }

    pub fn add_password(&mut self) -> Result<&mut Self, Error> {
        let ps = match rpassword::prompt_password_stdout(&format!(
            "Openstack password for user '{}': ",
            self.auth.username
        )) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        self.auth.password = ps.into();
        Ok(self)
    }

    pub fn add_password_if_not_existing(&mut self) -> Result<&mut Self, Error> {
        if self.auth.password == Auth::default().password {
            return self.add_password();
        }
        Ok(self)
    }

    pub fn apply(&mut self, other: &Self) -> &mut Self {
        // if other.auth.password != "".into() {
        //     self.auth.password = other.auth.password.clone()
        // };
        // if other.auth.username != "" {
        //     self.auth.username = other.auth.username.clone()
        // };
        // if other.cloud_name != "" {
        //     self.cloud_name = other.cloud_name.clone()
        // };
        // if other.auth.auth_url != "" {
        //     self.auth.auth_url = other.auth.auth_url.clone()
        // };
        // if other.auth.project_id != "" {
        //     self.auth.project_id = other.auth.project_id.clone()
        // };
        // if other.auth.project_domain_id != "" {
        //     self.auth.project_domain_id = other.auth.project_domain_id.clone()
        // };
        // if other.auth.user_domain_id != "" {
        //     self.auth.user_domain_id = other.auth.user_domain_id.clone()
        // };

        self.auth = self.auth.apply(&other.auth);

        if other.region_name != "" {
            self.region_name = other.region_name.clone()
        };
        if other.interface != "" {
            self.interface = other.interface.clone();
            if self.interface == "public" {
                self.only_use_public_endpoints = true
            } else {
                self.only_use_public_endpoints = false
            }
        };
        self
    }
}

impl Default for OpenstackInfoMap {
    fn default() -> OpenstackInfoMap {
        OpenstackInfoMap {
            cloud_name: String::from(""),
            // auth_url: String::from(""),
            // username: String::from(""),
            // password: String::from("").into(),
            // project_id: String::from(""),
            // project_domain_id: String::from(""),
            // user_domain_id: String::from(""),
            auth: Auth::default(),
            region_name: String::from(""),
            interface: String::from("public"),
            only_use_public_endpoints: true,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Auth {
    pub user_id: String,
    pub username: String,
    pub user_domain_id: String,
    pub user_domain_name: String,
    #[serde(skip_serializing)]
    pub password: secstr::SecUtf8,
    pub token: String,
    pub auth_url: String,
    pub system_scope: String,
    pub domain_id: String,
    pub domain_name: String,
    pub project_id: String,
    pub project_name: String,
    pub project_domain_id: String,
    pub project_domain_name: String,
    pub trust_id: String,
}

make_list!(Auth);

impl Default for Auth {
    fn default() -> Auth {
        Auth {
            user_id: String::from(""),
            username: String::from(""),
            user_domain_id: String::from(""),
            user_domain_name: String::from(""),
            password: secstr::SecUtf8::from(""),
            token: String::from(""),
            auth_url: String::from(""),
            system_scope: String::from(""),
            domain_id: String::from(""),
            domain_name: String::from(""),
            project_id: String::from(""),
            project_name: String::from(""),
            project_domain_id: String::from(""),
            project_domain_name: String::from(""),
            trust_id: String::from(""),
        }
    }
}

impl From<HashMap<String, String>> for Auth {
    fn from(map: HashMap<String, String>) -> Auth {
        serde_json::from_value(
            serde_json::to_value(map).expect("hashmap should be able to be converted to value"),
        )
        .expect("value is a hashmap")
    }
}

impl From<HashMap<String, serde_json::Value>> for Auth {
    fn from(map: HashMap<String, serde_json::Value>) -> Auth {
        serde_json::from_value(
            serde_json::to_value(map).expect("hashmap should be able to be converted to value"),
        )
        .expect("value is a hashmap")
    }
}


impl From<Auth> for HashMap<String, serde_json::Value> {
    fn from(auth: Auth) -> HashMap<String, serde_json::Value> {
        HashMap::from(&auth)
    }
}

impl From<&Auth> for HashMap<String, serde_json::Value> {
    fn from(auth: &Auth) -> HashMap<String, serde_json::Value> {
        use std::iter::FromIterator;
        let auth_value: serde_json::Value = serde_json::to_value(auth).expect("auth is serilizable");
        HashMap::from_iter(
            auth_value.as_object()
                        .expect("this is an object")
                        .iter()
                        .map(|x| (x.0.clone(), x.1.clone()))
        )
    }
}

impl Hash for Auth{
    fn hash<H: Hasher>(&self, state: &mut H) {
       for x in self.values(){
           if let Some(y) = x.as_bool(){
                y.hash(state);
                continue;
           }
           if let Some(y) = x.as_u64(){
                y.hash(state);
                continue;
           }
           if let Some(y) = x.as_i64(){
                y.hash(state);
                continue;
           }
           if let Some(y) = x.as_str(){
                y.hash(state);
                continue;
           }
       }
    }
}


impl Auth {
    pub fn from_env() -> Self {
        match envy::prefixed("OS_").from_env::<Auth>() {
            Ok(x) => x,
            Err(_e) => Auth::default(),
        }
    }

    pub fn apply(&self, other: &Self) -> Self {
        let default: HashMap<String, serde_json::Value> = Self::default().into();
        let mut self_hm: HashMap<String, serde_json::Value> = self.into();
        let other_hm: HashMap<String, serde_json::Value> = other.into();
        let password;
        if other.password.unsecure() != Self::default().password.unsecure(){
            password = other.password.clone();
        } else {
            password = self.password.clone();
        }
        for (key, value) in default.iter(){
            if other_hm.get(key) != Some(&value){
                self_hm.insert(key.clone(), other_hm.get(key).expect("this exists").clone());
            }
        }
        let mut new_auth = Auth::from(self_hm);
        new_auth.password = password;
        new_auth
    }

    pub fn pick_token_body(&self) -> serde_json::Value{
        create_token_body(&self)
    }

    pub fn create_hash(&self) -> String{
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        format!("{:X}", s.finish())
    }
}


pub fn create_token_body(auth: &Auth) -> serde_json::Value{

// {
//     "auth": {
//         "identity": {
//             "methods": [
//                 "password"
//             ],
//             "password": {
//                 "user": {
//                     "id": "ee4dfb6e5540447cb3741905149d9b6e",
//                     "password": "devstacker"
//                 }
//             }
//         },
//         "scope": {
//             "system": {
//                 "all": true
//             }
//         }
//     }
// }


// {
//     "auth": {
//         "identity": {
//             "methods": [
//                 "password"
//             ],
//             "password": {
//                 "user": {
//                     "id": "ee4dfb6e5540447cb3741905149d9b6e",
//                     "password": "devstacker"
//                 }
//             }
//         },
//         "scope": {
//             "project": {
//                 "domain": {
//                     "id": "default"
//                 },
//                 "name": "admin"
//             }
//         }
//     }
// }

    let default_auth = Auth::default();
    let mut body = json!({});

    // pick token or password method
    if auth.token != default_auth.token {
        body = json!({
            "auth": {
                "identity": {
                    "methods": [
                        "token"
                    ],
                    "token": {
                        "id": auth.token
                    }
                }
            }
        });
    }

    if auth.password != default_auth.password {
        body = json!({
            "auth": {
                "identity": {
                    "methods": ["password"],
                    "password": {
                        "user": {
                            "password": auth.password.unsecure()
                        }
                    }
                }
            }
        });
        if auth.username != default_auth.username{
            body["auth"]["identity"]["password"]["user"]["name"] = json!(auth.username)
        }
        if auth.user_id != default_auth.user_id{
            body["auth"]["identity"]["password"]["user"]["id"] = json!(auth.user_id)
        }

        if auth.user_domain_name != default_auth.user_domain_name {
            body["auth"]["identity"]["password"]["user"]["domain"] = json!({
                "name": auth.user_domain_name
            })
        }

        if auth.user_domain_id != default_auth.user_domain_id {
            body["auth"]["identity"]["password"]["user"]["domain"] = json!({
                "id": auth.user_domain_id
            })
        }

    }

    // set scope or set scope as 'unscoped'

    // if auth.system_scope !=  default_auth.system_scope{
    //     body[""]
    // }

    if auth.domain_id != default_auth.domain_id {
        body["auth"]["scope"] = json!({
            "domain": {
                "id": auth.domain_id
            }
        });
    }

    if auth.domain_name != default_auth.domain_name {
        body["auth"]["scope"] = json!({
            "domain": {
                "name": auth.domain_name
            }
        });
    }
    if auth.project_name != default_auth.project_name {
        body["auth"]["scope"]["project"] = json!({"name": auth.project_name});
    }
    if auth.project_id != default_auth.project_id {
        body["auth"]["scope"]["project"] = json!({"id": auth.project_id});
    }

    if auth.project_domain_id != default_auth.project_domain_id{
        body["auth"]["scope"]["project"]["domain"] = json!({
            "id": auth.project_domain_id
        });
    }

    if body["auth"].get("scope").is_none(){
        body["auth"]["scope"] = json!("unscoped");
    }

    body
}

// #[test]
// fn test_env_config() {
//     use std::env;
//     env::set_var("OS_CLOUD", "cloud");
//     env::set_var("OS_AUTH_URL", "https://identity.example.com");
//     env::set_var("OS_USERNAME", "test_user");
//     env::set_var("OS_PASSWORD", "secret_password");
//     env::set_var("OS_PROJECT_ID", "12345678");
//     env::set_var("OS_PROJECT_DOMAIN_ID", "123412341234");
//     env::set_var("OS_USER_DOMAIN_ID", "123412341234");
//     env::set_var("OS_REGION_NAME", "test");
//     env::set_var("OS_INTERFACE", "admin");

//     let expected_config = OpenstackInfoMap {
//         cloud_name: String::from("cloud"),
//         auth_url: String::from("https://identity.example.com"),
//         username: String::from("test_user"),
//         password: secstr::SecUtf8::from("secret_password"),
//         project_id: String::from("12345678"),
//         project_domain_id: String::from("123412341234"),
//         user_domain_id: String::from("123412341234"),
//         region_name: String::from("test"),
//         interface: String::from("admin"),
//         only_use_public_endpoints: false,
//     };

//     let config = OpenstackInfoMap::from_env(String::from(""));

//     env::remove_var("OS_CLOUD");
//     env::remove_var("OS_AUTH_URL");
//     env::remove_var("OS_USERNAME");
//     env::remove_var("OS_PASSWORD");
//     env::remove_var("OS_PROJECT_ID");
//     env::remove_var("OS_PROJECT_DOMAIN_ID");
//     env::remove_var("OS_USER_DOMAIN_ID");
//     env::remove_var("OS_REGION_NAME");
//     env::remove_var("OS_INTERFACE");

//     assert_eq!(config, expected_config);
// }

// #[test]
// fn test_parse_clouds_yaml() {
//     let raw_clouds_yaml = r#"
//     clouds:
//       cloud:
//         auth:
//           auth_url: "https://identity.example.com"
//           username: "test_user"
//           password: "secret_password"
//           user_domain_id: "123412341234"
//           project_domain_id: "123412341234"
//           project_id: "12345678"
//         region_name: "test"
//         interface: "public"
//     "#;
//     let yaml = serde_yaml::from_str(raw_clouds_yaml).unwrap();

//     let expected_config = OpenstackInfoMap {
//         cloud_name: String::from("cloud"),
//         // auth_url: String::from("https://identity.example.com"),
//         // username: String::from("test_user"),
//         // password: secstr::SecUtf8::from("secret_password"),
//         // project_id: String::from("12345678"),
//         // project_domain_id: String::from("123412341234"),
//         // user_domain_id: String::from("123412341234"),
//         auth: Auth::default(),
//         region_name: String::from("test"),
//         interface: String::from("public"),
//         only_use_public_endpoints: true,
//     };

//     let config = OpenstackInfoMap::parse_clouds_yaml(yaml, String::from("")).unwrap();

//     // assert_eq!(secstr::SecUtf8::from("secret_password"), config.password);
//     // assert_eq!("secret_password", config.password.unsecure());
//     assert_eq!(config, expected_config);
// }

#[test]
fn test_parse_clouds_yaml_errors_when_two_clouds_are_found_without_specific_cloud_chosen() {
    let raw_clouds_yaml = r#"
    clouds:
      cloud:
        auth:
          auth_url: "https://identity.example.com"
          username: "test_user"
          password: "secret_password"
        region_name: "test"
      cloud2:
        auth:
          auth_url: "https://identity2.example.com"
          username: "test_user"
          password: "secret_password"
        region_name: "test"
    "#;
    let yaml = serde_yaml::from_str(raw_clouds_yaml).unwrap();

    let config = OpenstackInfoMap::parse_clouds_yaml(yaml, String::from(""));
    assert!(config.is_err())
}

#[test]
fn test_parse_clouds_yaml_succeeds_when_two_clouds_are_found_with_specific_cloud_chosen() {
    let raw_clouds_yaml = r#"
    clouds:
      cloud:
        auth:
          auth_url: "https://identity.example.com"
          username: "test_user"
          password: "secret_password"
        region_name: "test"
      cloud2:
        auth:
          auth_url: "https://identity2.example.com"
          username: "test_user"
          password: "secret_password"
        region_name: "test"
    "#;
    let yaml = serde_yaml::from_str(raw_clouds_yaml).unwrap();

    let config = OpenstackInfoMap::parse_clouds_yaml(yaml, String::from("cloud2"));
    assert!(config.is_ok())
}

#[test]
fn test_parse_clouds_yaml_fails_when_wrong_type() {
    let raw_clouds_yaml = r#"
    clouds:
      cloud: [1, 2]
    "#;
    let yaml = serde_yaml::from_str(raw_clouds_yaml).unwrap();

    let config = OpenstackInfoMap::parse_clouds_yaml(yaml, String::from(""));
    assert!(config.is_err());

    let raw_clouds_yaml = r#"
    clouds: "testing"
    "#;
    let yaml = serde_yaml::from_str(raw_clouds_yaml).unwrap();

    let config = OpenstackInfoMap::parse_clouds_yaml(yaml, String::from(""));
    assert!(config.is_err())
}

#[test]
fn test_auth_from_hashmap() {
    let mut hm: HashMap<String, String> = HashMap::new();
    hm.insert("project_id".to_string(), "1234".to_string());
    hm.insert("user_id".to_string(), "4321".to_string());
    hm.insert("password".to_string(), "secret_password".to_string());

    let expected = Auth {
        project_id: "1234".to_string(),
        user_id: "4321".to_string(),
        password: secstr::SecUtf8::from("secret_password"),
        ..Default::default()
    };
    assert_eq!(Auth::from(hm), expected);
}

#[test]
fn test_auth_apply_works(){
    let first_auth = Auth {
        project_id: "1234".to_string(),
        user_id: "4321".to_string(),
        ..Default::default()
    };

    let second_auth = Auth {
        project_id: "4567".to_string(),
        username: "testing".to_string(),
        password: secstr::SecUtf8::from("secret_password"),
        ..Default::default()
    };

    let expected = Auth{
        project_id: "4567".to_string(),
        user_id: "4321".to_string(),
        username: "testing".to_string(),
        password: secstr::SecUtf8::from("secret_password"),
        ..Default::default()
    };
    let auth = first_auth.apply(&second_auth);

    println!("{}", second_auth.password.unsecure());
    println!("{}", auth.password.unsecure());
    assert_eq!(auth, expected);
}

#[test]
fn test_auth_apply_works_multiple_times(){
    let first_auth = Auth::default();

    let second_auth = Auth {
        project_id: "1234".to_string(),
        user_id: "4321".to_string(),
        ..Default::default()
    };

    let third_auth = Auth {
        project_id: "4567".to_string(),
        username: "testing".to_string(),
        password: secstr::SecUtf8::from("secret_password"),
        ..Default::default()
    };

    let fourth_auth = Auth {
        auth_url: "https://example.com/auth".to_string(),
        password: secstr::SecUtf8::from("other_password"),
        ..Default::default()
    };

    let fifth_auth = Auth::default();

    let expected = Auth{
        project_id: "4567".to_string(),
        auth_url: "https://example.com/auth".to_string(),
        user_id: "4321".to_string(),
        username: "testing".to_string(),
        password: secstr::SecUtf8::from("other_password"),
        ..Default::default()
    };
    let auth = first_auth.apply(&second_auth)
                         .apply(&third_auth)
                         .apply(&fourth_auth)
                         .apply(&fifth_auth);

    assert_eq!(fourth_auth.password.unsecure(),  auth.password.unsecure());
    assert_eq!(auth, expected);
}


#[test]
fn test_create_token_body_picks_correct_body_password(){
    let auth = Auth {
        username: "test".to_string(),
        project_domain_id: "4321".to_string(),
        user_domain_id: "4321".to_string(),
        project_id: "1234".to_string(),
        password: secstr::SecUtf8::from("password"),
        ..Default::default()
    };

    let expected = json!({
        "auth": {
            "identity": {
                "methods": ["password"],
                "password": {
                    "user": {
                        "name": "test",
                        "domain": {
                            "id": "4321"
                        },
                        "password": "password"
                    }
                }
            },
            "scope": {
                "project": {
                    "id": "1234",
                    "domain": {
                        "id": "4321"
                    }
                }
            }
        }
    });

    assert_eq!(create_token_body(&auth), expected);
}

#[test]
fn test_create_token_body_picks_correct_body_token(){
    let auth = Auth {
        user_id: "123456".to_string(),
        domain_id: "1234".to_string(),
        token: "abcdefghijklmnopqrstuvwxyz".to_string(),
        ..Default::default()
    };

    let expected = json!({
        "auth": {
            "identity": {
                "methods": ["token"],
                "token": {
                    "id": "abcdefghijklmnopqrstuvwxyz"
                }
            },
            "scope": {
                "domain": {
                    "id": "1234",
                }
            }
        }
    });

    assert_eq!(create_token_body(&auth), expected);
}

#[test]
fn test_create_token_body_picks_correct_body_and_set_unscoped(){
    let auth = Auth {
        token: "abcdefghijklmnopqrstuvwxyz".to_string(),
        ..Default::default()
    };

    let expected = json!({
        "auth": {
            "identity": {
                "methods": ["token"],
                "token": {
                    "id": "abcdefghijklmnopqrstuvwxyz"
                }
            },
            "scope": "unscoped"
        }
    });

    assert_eq!(create_token_body(&auth), expected);
}

// example setup teardown function
// fn run_test<T>(test: T) -> ()
//     where T: FnOnce() -> () + std::panic::UnwindSafe
// {
//     let result = std::panic::catch_unwind(|| {
//         test()
//     });
//     assert!(result.is_ok())
// }

// {
//     "cloud": "",
//     "auth-type": "",
//     "auth-url": "",
//     "url": "",
//     "domain-name": "",
//     "domain-id": "",
//     "project-name": "",
//     "project-id": "",
//     "project-domain-name": "",
//     "project-domain-id": "",
//     "username": "",
//     "password": "",
//     "token": "",
//     "user-domain-name": "",
//     "user-domain-id": "",
//     "trust-id": "",
//     "default-domain": "",
//     "region-name": "",
//     "cacert": "",
//     "cert": "",
//     "key": "",
//     "identity-api-version": "",
//     "XXXX-api-version": "",
//     "interface": ""
// }

// {
//     "user-id": "",
//     "username": "",
//     "user-domain-id": "",
//     "user-domain-name": "",
//     "password": "",
//     "token": "",
//     "auth-url": "",
//     "system-scope": "",
//     "domain-id": "",
//     "domain-name": "",
//     "project-id": "",
//     "project-name": "",
//     "project-domain-id": "",
//     "project-domain-name": "",
//     "trust-id": ""
// }


#[test]
fn test_auth_create_hash(){
    let auth: Auth = serde_json::from_value(json!({
        "username": "username",
        "password": "password",
        "auth_url": "https://example.com"
    })).unwrap();
    assert_eq!("53AB15A5FC9889D6", auth.create_hash())
}