use std::collections::HashMap;
use std::io::{stdout, stderr, Error, ErrorKind};
use yaml_rust::{YamlLoader, yaml};

pub fn convert_to_singular(tmp: &str) -> &str {
    // not 100% bulletproof but good enough for matching
    // let tmp = blub.to_lowercase();

    let last_char = tmp[(tmp.len()-1)..].chars().next().unwrap();

    let is_multiple = match last_char{
        's' => true,
        'S' => true,
        _ => false
    };

    if is_multiple{
        &tmp[..(tmp.len()-1)]
    }
    else{
        tmp
    }
}

pub fn convert_to_multiple(tmp: String) -> String {
    // not 100% bulletproof but good enough for matching
    // let tmp = blub.to_lowercase();

    let last_char = tmp[(tmp.len()-1)..].chars().next().unwrap();

    let is_multiple = match last_char{
        's' => true,
        'S' => true,
        _ => false
    };

    if is_multiple{
        tmp
    }
    else{
        tmp + "s"
    }
}


pub fn add_slash(tmp: &str) -> String {
    let last_char = tmp[(tmp.len()-1)..].chars().next().unwrap();
    let has_slash = match last_char{
        '/' => true,
        _ => false
    };

    if has_slash{
        tmp.to_string()
    }
    else{
        format!("{}/", tmp)
    }
}

pub fn add_if_exists<S>(hashmap: &mut HashMap<String, String>, name: S, item: String)where S: ToString{
    if item != ""{
        hashmap.insert(name.to_string(), item);
    };
}

pub fn to_boolean(a_str: String) -> Option<bool>{
    // [True, ‘True’, ‘TRUE’, ‘true’, ‘1’, ‘ON’, ‘On’, ‘on’, ‘YES’, ‘Yes’, ‘yes’, ‘y’, ‘t’, False, ‘False’, ‘FALSE’, ‘false’, ‘0’, ‘OFF’, ‘Off’, ‘off’, ‘NO’, ‘No’, ‘no’, ‘n’, ‘f’]
    match a_str.to_lowercase().as_str(){
        "yes" | "y" | "1" | "on" | "t" | "true" => Some(true),
        "no" | "n" | "0" | "off" | "f" | "false" => Some(false),
        _ => None
    }
}

pub fn read_yaml(location: String) -> Result<serde_yaml::Value, Error>{
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

pub fn read_yaml_rust(location: String) -> yaml::Yaml{
    let s = read_yaml(location).unwrap();
    yaml::Yaml::from_str(&serde_yaml::to_string(&s).unwrap())
}

pub fn compare_different_cases(a: &str, b: &str) -> bool{
    use heck::SnakeCase;
    a.to_snake_case() == b.to_snake_case()
}


pub fn get_first_value_from_hashmap_with_vec(map: &HashMap<String, Vec<String>>, key: &str) -> Option<String>{
    match map.get(key){
        Some(x) => Some(x[0].clone()),
        _ => None
    }
}


#[test]
fn test_convert_to_singular(){
     assert_eq!(convert_to_singular("TESTS"), "TEST");
     assert_eq!(convert_to_singular("TEST"), "TEST");
     assert_eq!(convert_to_singular("tests"), "test");
     assert_eq!(convert_to_singular("Yes"), "Ye");
}

#[test]
fn test_convert_to_multiple(){
     assert_eq!(convert_to_multiple("TESTS".to_string()), "TESTS");
     assert_eq!(convert_to_multiple("TEST".to_string()), "TESTs");
     assert_eq!(convert_to_multiple("tests".to_string()), "tests");
     assert_eq!(convert_to_multiple("Yes".to_string()), "Yes");
}


#[test]
fn test_to_boolean() {
    assert_eq!(to_boolean("Yes".to_string()), Some(true));
    assert_eq!(to_boolean("Y".to_string()), Some(true));
    assert_eq!(to_boolean("ON".to_string()), Some(true));
    assert_eq!(to_boolean("true".to_string()), Some(true));
    assert_eq!(to_boolean("True".to_string()), Some(true));
    assert_eq!(to_boolean("TRUE".to_string()), Some(true));
    assert_eq!(to_boolean("Yes".to_string()), Some(true));
    assert_eq!(to_boolean("1".to_string()), Some(true));

    assert_eq!(to_boolean("N".to_string()), Some(false));
    assert_eq!(to_boolean("OFF".to_string()), Some(false));
    assert_eq!(to_boolean("false".to_string()), Some(false));
    assert_eq!(to_boolean("False".to_string()), Some(false));
    assert_eq!(to_boolean("FALSE".to_string()), Some(false));
    assert_eq!(to_boolean("0".to_string()), Some(false));

    assert_eq!(to_boolean("test".to_string()), None);
    assert_eq!(to_boolean("123".to_string()), None);
}

#[test]
fn test_add_slash(){
    assert_eq!(add_slash("test.com"), "test.com/");
    assert_eq!(add_slash("test.com/"), "test.com/");
}

#[test]
fn test_compare_different_cases(){
    assert_eq!(compare_different_cases("HalloTest", "hallo_test"), true);
    assert_eq!(compare_different_cases("Hallo Test", "hallo-test"), true);
    assert_eq!(compare_different_cases("halloTest", "hallo test"), true);
    assert_eq!(compare_different_cases("hallotest", "Hallo Test"), false);
    assert_eq!(compare_different_cases("Hallo", "test"), false);
}