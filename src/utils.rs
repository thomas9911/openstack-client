use std::collections::HashMap;

pub fn convert_to_singular(blub: &str) -> String {
    // not 100% bulletproof but good enough for matching
    let tmp = blub.to_lowercase();

    let last_char = tmp[(tmp.len()-1)..].chars().next().unwrap();

    let is_multiple = match last_char{
        's' => true,
        _ => false
    };

    if is_multiple{
        tmp[..(tmp.len()-1)].to_string()
    }
    else{
        tmp
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

#[test]
fn test_convert_to_singular(){
     assert_eq!(convert_to_singular("TESTS"), "test".to_string());
     assert_eq!(convert_to_singular("TEST"), "test".to_string());
     assert_eq!(convert_to_singular("Yes"), "ye".to_string());
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