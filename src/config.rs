use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use utils::{
    add_slash, get_first_value_from_hashmap_with_vec, hashmap_with_vec_to_json,
    make_hashmaps_from_dot_notation, read_yaml, remove_slash_start,
};
use client::{Client, Response};

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

    pub fn refresh_token(&mut self) -> Result<(), Error> {
        let password = match String::from_utf8(self.config.password.unsecure().to_vec()) {
            Ok(x) => x,
            Err(_e) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "binary must be in utf-8",
                ));
            }
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
        let auth_url = match url::Url::parse(&add_slash(&self.config.auth_url)) {
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
        let os_token = match headers.get("x-subject-token"){
            Some(x) => x,
            None => headers.get("X-Subject-Token").expect("Expected an auth token")
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

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenstackInfoMap {
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

impl OpenstackInfoMap {
    pub fn new(
        cloud_name: String,
        auth_url: String,
        username: String,
        password: String,
        project_id: String,
        project_domain_id: String,
        user_domain_id: String,
        region_name: String,
        interface: String,
    ) -> OpenstackInfoMap {
        let ps: secstr::SecStr = secstr::SecStr::from(password);
        OpenstackInfoMap {
            cloud_name,
            auth_url,
            username,
            password: ps,
            project_id,
            project_domain_id,
            user_domain_id,
            region_name,
            interface,
            only_use_public_endpoints: true,
        }
    }

    pub fn from_clouds_yaml(region: String) -> Result<OpenstackInfoMap, Error> {
        OpenstackInfoMap::from_yaml("clouds.yaml".to_string(), region)
    }

    pub fn from_yaml(location: String, region: String) -> Result<OpenstackInfoMap, Error> {
        let mut region_copy = region.clone();
        let value = match read_yaml(location) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };
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
        let auth_url: String = auth_map
            .get("auth_url")
            .unwrap_or(&serde_yaml_string)
            .as_str()
            .unwrap()
            .to_string();
        let username: String = auth_map
            .get("username")
            .unwrap_or(&serde_yaml_string)
            .as_str()
            .unwrap()
            .to_string();
        let password: String = auth_map
            .get("password")
            .unwrap_or(&serde_yaml_string)
            .as_str()
            .unwrap()
            .to_string();
        let project_id: String = auth_map
            .get("project_id")
            .unwrap_or(&serde_yaml_string)
            .as_str()
            .unwrap()
            .to_string();
        let project_domain_id: String = auth_map
            .get("project_domain_id")
            .unwrap_or(&serde_yaml_string)
            .as_str()
            .unwrap()
            .to_string();
        let user_domain_id: String = auth_map
            .get("user_domain_id")
            .unwrap_or(&serde_yaml_string)
            .as_str()
            .unwrap()
            .to_string();
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
        Ok(OpenstackInfoMap::new(
            cloud_name,
            auth_url,
            username,
            password,
            project_id,
            project_domain_id,
            user_domain_id,
            region_name,
            interface,
        ))
    }

    pub fn from_env(region: String) -> OpenstackInfoMap {
        let mut cloud_name = region.clone();
        if region == "" {
            cloud_name = std::env::var("OS_CLOUD").unwrap_or("".to_string());
        }
        let auth_url: String = std::env::var("OS_AUTH_URL").unwrap_or("".to_string());
        let username: String = std::env::var("OS_USERNAME").unwrap_or("".to_string());
        let password: String = std::env::var("OS_PASSWORD").unwrap_or("".to_string());
        let project_id: String = std::env::var("OS_PROJECT_ID").unwrap_or("".to_string());
        let project_domain_id: String =
            std::env::var("OS_PROJECT_DOMAIN_ID").unwrap_or("".to_string());
        let user_domain_id: String = std::env::var("OS_USER_DOMAIN_ID").unwrap_or("".to_string());
        let region_name: String = std::env::var("OS_REGION_NAME").unwrap_or("".to_string());
        let interface: String = std::env::var("OS_INTERFACE").unwrap_or("".to_string());

        OpenstackInfoMap::new(
            cloud_name,
            auth_url,
            username,
            password,
            project_id,
            project_domain_id,
            user_domain_id,
            region_name,
            interface,
        )
    }

    pub fn add_password(&mut self) -> Result<&mut Self, Error> {
        let ps = match rpassword::prompt_password_stdout(&format!(
            "Openstack password for user '{}': ",
            self.username
        )) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        self.password = ps.into();
        Ok(self)
    }

    pub fn add_password_if_not_existing(&mut self) -> Result<&mut Self, Error> {
        if self.password == "".to_string().into() {
            return self.add_password();
        }
        Ok(self)
    }

    pub fn apply(&mut self, other: &Self) -> &mut Self {
        if other.password != "".into() {
            self.password = other.password.clone()
        };
        if other.username != "" {
            self.username = other.username.clone()
        };
        if other.cloud_name != "" {
            self.cloud_name = other.cloud_name.clone()
        };
        if other.auth_url != "" {
            self.auth_url = other.auth_url.clone()
        };
        if other.project_id != "" {
            self.project_id = other.project_id.clone()
        };
        if other.project_domain_id != "" {
            self.project_domain_id = other.project_domain_id.clone()
        };
        if other.user_domain_id != "" {
            self.user_domain_id = other.user_domain_id.clone()
        };
        if other.region_name != "" {
            self.region_name = other.region_name.clone()
        };
        if other.interface != "" {
            self.interface = other.interface.clone()
        };
        self
    }
}

impl Default for OpenstackInfoMap {
    fn default() -> OpenstackInfoMap {
        OpenstackInfoMap {
            cloud_name: String::from(""),
            auth_url: String::from(""),
            username: String::from(""),
            password: String::from("").into(),
            project_id: String::from(""),
            project_domain_id: String::from(""),
            user_domain_id: String::from(""),
            region_name: String::from(""),
            interface: String::from("public"),
            only_use_public_endpoints: true,
        }
    }
}
