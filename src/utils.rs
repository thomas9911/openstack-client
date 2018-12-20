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

pub fn add_if_exists<S>(hashmap: &mut HashMap<String, String>, name: S, item: String)where S: ToString{
    if item != ""{
        hashmap.insert(name.to_string(), item);
    };
}

pub fn to_boolean(a_str: String) -> Option<bool>{
    // [True, ‘True’, ‘TRUE’, ‘true’, ‘1’, ‘ON’, ‘On’, ‘on’, ‘YES’, ‘Yes’, ‘yes’, ‘y’, ‘t’, False, ‘False’, ‘FALSE’, ‘false’, ‘0’, ‘OFF’, ‘Off’, ‘off’, ‘NO’, ‘No’, ‘no’, ‘n’, ‘f’]
    match a_str.to_lowercase().as_str(){
        "yes" | "y" | "1" | "on" | "t" => Some(true),
        "no" | "n" | "0" | "off" | "f" => Some(false),
        _ => None
    }
}