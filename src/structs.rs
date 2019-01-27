use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use utils::{convert_to_multiple, compare_different_cases};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMap {
    pub map: HashMap<String, Resource>,
    pub types: HashMap<String, ResourceTypeEnum>,
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