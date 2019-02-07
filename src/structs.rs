use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use utils::{convert_to_multiple, compare_different_cases, get_first_value_from_hashmap_with_vec};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMap {
    pub map: HashMap<String, Resource>,
    pub types: HashMap<String, ResourceTypeEnum>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resource {
    pub name: String,
    pub endpoint_path: String,
    pub resource_type: ResourceTypeEnum,
    pub post_parameters: Option<Vec<PostParameter>>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResourceTypeEnum {
    String(String),
    ResourceType(ResourceType),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceType {
    pub name: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostParameter{
    pub name: String,
    pub path: String,
    pub help: Option<String>,
    #[serde(default = "false_bool")]
    pub multiple: bool,
    #[serde(default = "false_bool")]
    pub required: bool,
    #[serde(default = "false_bool")]
    pub hidden: bool,
    pub default: Option<String>
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionMap {
    pub map: HashMap<String, ActionResource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionResource {
    pub name: String,
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    pub action: String,
    pub resource: String,
    pub url_parameter: String,
    pub requires_id: bool,
    pub body_name: String,
    #[serde(default = "empty_vec_action_parameter")]
    pub params: Vec<ActionParameter>,
    #[serde(default = "post_method")]
    pub http_method: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionParameter {
    pub name: String,
    #[serde(default = "false_bool")]
    pub required: bool,
    pub help: Option<String>,
    pub default: Option<String>,
}


impl ActionMap {
    pub fn new() -> Self {
        let raw_string_yaml = include_str!("../data/actions.yaml");
        let yaml_actions: serde_yaml::Value = serde_yaml::from_str(raw_string_yaml).expect("actions.yaml is not valid yaml");

        let mut new_map: HashMap<String, ActionResource> = HashMap::new();
        if let serde_yaml::Value::Mapping(x) = yaml_actions{
            for (a, b) in  x.iter(){
                if let serde_yaml::Value::Sequence(ref y) = b["resources"] {
                    let actions: Vec<Action> = y.iter().map(|item| {
                        let mut ok = item.as_mapping().unwrap().clone();
                        ok.insert("action".into(), a.clone());
                        serde_yaml::from_value(ok.into()).unwrap()
                        }).collect();
                    let name: String = a.as_str().unwrap().to_string();
                    let ar = ActionResource { name: name.clone(), actions: actions };
                    new_map.insert(name, ar);
                }
            }
        }
        ActionMap { map: new_map }
    }

    pub fn get_action(&self, action: String, resource: String) -> Option<Action>{
        if let Some(x) = self.map.get(&action){
            for item in &x.actions{
                if item.resource == resource{
                    return Some(item.clone());
                }
            }
        }
        None
    }
}

impl Action {
    pub fn make_body(&self, map: &HashMap<String, Vec<String>>) -> serde_json::Value{
        let mut main_body = HashMap::new();
        let body_name = self.body_name.clone();
        let mut sub_body = HashMap::new();

        for param in self.params.iter(){
            let kaas = match get_first_value_from_hashmap_with_vec(map, &param.name){
                Some(x) => Some(x),
                None => param.default.clone()
            };
            sub_body.insert(param.name.clone(), kaas);
        };
        let sub_body_value = match sub_body.is_empty(){
            true => serde_json::Value::Null,
            false => serde_json::to_value(sub_body).expect("you broke it")
        };

        main_body.insert(body_name, sub_body_value);
        serde_json::to_value(main_body).expect("you broke it")
    }
}


impl ResourceMap{
    pub fn new() -> ResourceMap {
        let mut resource_types: Vec<String> = vec![];
        let raw_string_yaml = include_str!("../data/resource_types.yaml");
        let yaml_resource_type: serde_yaml::Value = serde_yaml::from_str(raw_string_yaml).expect("resource_types.yaml is not valid yaml");

        if let serde_yaml::Value::Sequence(ref types) = yaml_resource_type["type"]{
            for item in types.iter(){
                if let serde_yaml::Value::String(s) = item{
                    resource_types.push(s.clone());
                }
            }
        };

        let raw_string_yaml = include_str!("../data/resources.yaml");
        let yaml_resources: serde_yaml::Value = serde_yaml::from_str(raw_string_yaml).expect("resource_types.yaml is not valid yaml");

        let mut new_map: HashMap<String, Resource> = HashMap::new();
        let mut types: HashMap<String, ResourceTypeEnum> = HashMap::new();

        if let serde_yaml::Value::Mapping(x) = yaml_resources{
            for (a, b) in  x.iter(){
                let name: String = a.as_str().unwrap().to_string();
                let resource_type_str: String = serde_yaml::from_value(b["resource_type"].clone()).unwrap();
                // if !(m.contains_key(&resource_type_str) & resource_types.contains(&resource_type_str)){
                //     continue
                // }
                let resource_type = ResourceTypeEnum::String(resource_type_str.clone());
                types.insert(resource_type_str.clone(), resource_type.clone());
                let post_parameters: Option<Vec<PostParameter>> = serde_yaml::from_value(b["post_parameters"].clone()).unwrap();
                let endpoint_path: String = serde_yaml::from_value(b["endpoint_path"].clone()).unwrap();
                let description: Option<String> = serde_yaml::from_value(b["description"].clone()).unwrap();
                let res = Resource{name: name.clone(), resource_type, post_parameters, endpoint_path, description};

                new_map.insert(name, res);
                }
        }

        ResourceMap {
            map: new_map,
            types: types
        }
    }

    pub fn update_from_identity(&mut self, m: &HashMap<String, String>){
        let mut resource_types: HashMap<String, ResourceTypeEnum> = HashMap::new();
        for (k, v) in m.iter(){
            resource_types.insert(k.clone(), ResourceTypeEnum::ResourceType(ResourceType{name: k.clone(), endpoint: v.clone()}));
        }

        for res in self.map.values_mut(){
            if let ResourceTypeEnum::String(x) = res.resource_type.clone(){
                if let Some(y) = resource_types.get(&x){
                    res.resource_type = y.clone();
                }
            }
        }
        self.types = resource_types;
    }

    pub fn get_resource(&self, user_input: String) -> Result<Resource, Error>{
        let mut tmp = String::from("");
        let mut found = false;
        for key in self.map.keys() {
            if compare_different_cases(&convert_to_multiple(user_input.clone()), &convert_to_multiple(key.to_string())){
                tmp = key.to_string();
                found = true;
                break;
            }
        }
        if !found{
            return Err(Error::new(ErrorKind::InvalidData, format!("'{}' is not a valid resource", &user_input)))
        } else{
            Ok(self.map.get(&tmp).expect("comparision went wrong").clone())
        }
    }

    pub fn get_resource_type(&self, user_input: String) -> Result<ResourceTypeEnum, Error>{
        let mut tmp = String::from("");
        let mut found = false;
        for key in self.types.keys() {
            if compare_different_cases(&convert_to_multiple(user_input.clone()), &convert_to_multiple(key.to_string())){
                tmp = key.to_string();
                found = true;
                break;
            }
        }
        if !found{
            return Err(Error::new(ErrorKind::InvalidData, format!("'{}' is not a valid resource type", &user_input)))
        } else{
            Ok(self.types.get(&tmp).expect("comparision went wrong").clone())
        }
    }
}

#[allow(dead_code)]
fn true_bool() -> bool {
    true
}

#[allow(dead_code)]
fn false_bool() -> bool {
    false
}

#[allow(dead_code)]
fn empty_vec_action_parameter() -> Vec<ActionParameter> {
    vec![]
}

#[allow(dead_code)]
fn post_method() -> String{
    reqwest::Method::POST.as_str().to_string()
}