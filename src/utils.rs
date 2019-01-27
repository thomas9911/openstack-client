use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use yaml_rust::yaml;

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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


pub fn make_hashmaps_from_dot_notation(listing: Vec<(String, serde_json::Value)>) -> serde_json::Value{
/*
let listing = vec![("some.thing", "15"), ("some.stuff", "foo")];
let post_body: serde_json::Value = make_hashmaps_from_dot_notation(listing);
println!("{}", serde_json::to_string_pretty(&post_body).unwrap());
{
    "some": {
        "thing": "15",
        "stuff": "foo"
    }
}
*/
    let mut stuffs = vec![];
    for (k, v) in listing{
        let mut json_string: String = String::from("");
        let mut first = true;
        for item in k.split('.').rev(){
            if first{
                json_string = format!("\"{}\": {}", item, v);
                first = false;
            }else{
                json_string = format!("\"{}\": {{{}}}", item, json_string);
            }
        }
        json_string = format!("{{{}}}", json_string);
        let j: serde_json::Value = serde_json::from_str(&json_string).unwrap();
        stuffs.push(j);
    }

    let mut end_value: serde_json::Value = serde_json::Value::Null;
    let mut first = true;
    for item in stuffs{
        if first{
            end_value = item;
            first = false;
        }
        else{
            end_value = merge_values(&end_value, &item);
        }
    }
    end_value
}


fn merge_values(a: &serde_json::Value, b: &serde_json::Value) -> serde_json::Value{
    let mut c = a.clone();
    merge(&mut c, b);
    c
}


fn merge(a: &mut serde_json::Value, b: &serde_json::Value) {
    match (a, b) {
        (&mut serde_json::Value::Object(ref mut a), serde_json::Value::Object(ref b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(serde_json::Value::Null), v);
            }
        }
        (&mut serde_json::Value::Array(ref mut a), serde_json::Value::Array(ref b)) => {
            let mut tmp = b.clone();
            a.append(&mut tmp);
        }
        (&mut serde_json::Value::Array(ref mut a), b) => {
            a.push(b.clone());
        }
        (a, b) => {
            if let serde_json::Value::Array(x) = b {
                let mut tmp = x.clone();
                if let serde_json::Value::Null = a{
                    *a = b.clone();
                } else{
                    tmp.insert(0, a.clone());
                    *a = serde_json::Value::Array(tmp);
                }
            } else{
                *a = b.clone();
            }
        }
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

#[test]
fn test_merge_values(){
    let a = json!({
        "hallo": 15,
        "oke": "geld"
    });
    let b = json!({
        "oke": "paard"
    });
    let c = json!({
        "hallo": 15,
        "oke": "paard"
    });
    assert_eq!(merge_values(&a, &b), c);
}

#[test]
fn test_merge_values_2(){
    let a = json!({
        "hallo": 15,
        "oke": {
            "wop": 15
        }
    });
    let b = json!({
        "oke": {
            "paard": "wauw"
        }
    });
    let c = json!({
        "hallo": 15,
        "oke": {
            "paard": "wauw",
            "wop": 15
        }
    });
    assert_eq!(merge_values(&a, &b), c);
}

#[test]
fn test_merge_values_3(){
    let a = json!({
        "a": ["b"],
        "c": [{
            "d": ["e", "g"],
            "e": "f"
        }],
        "d": "f"
    });
    let b = json!({
        "a": ["c"],
        "c": [{
            "d": ["e", "f"],
            "e": "f"
        }],
        "d": "e"
    });
    let c = json!({
        "a": ["b", "c"],
        "c": [{
            "d": ["e", "g"],
            "e": "f"
        }, {
            "d": ["e", "f"],
            "e": "f"
        }],
        "d": "e"
    });
    assert_eq!(merge_values(&a, &b), c);
}

#[test]
fn test_hashmaps_from_dot_notation(){
    let listing = vec![
        ("some.thing".into(), "15".into()),
        ("some.stuff".into(), "foo".into())
        ];

    let outcome = json!({
        "some": {
            "thing": "15",
            "stuff": "foo"
            }
        });
    let post_body: serde_json::Value = make_hashmaps_from_dot_notation(listing);
    assert_eq!(post_body, outcome);

    let listing = vec![
        ("some.thing".into(), "15".into()),
        ("some.stuff".into(), vec!["foo", "bar"].into())
        ];

    let outcome = json!({
        "some": {
            "thing": "15",
            "stuff": ["foo", "bar"]
            }
        });
    let post_body: serde_json::Value = make_hashmaps_from_dot_notation(listing);
    assert_eq!(post_body, outcome);

    let listing = vec![
        ("some.thing".into(), "15".into()),
        ("some.stuff".into(), vec!["foo", "bar"].into()),
        ("some.other.more".into(), "cool".into()),
        ("some.other.less".into(), "cool".into()),
        ("other.body".into(), "posts".into())
        ];
    let outcome = json!({
        "some": {
            "thing": "15",
            "stuff": ["foo", "bar"],
            "other": {
                "less": "cool",
                "more": "cool"
            }
        },
        "other": {
            "body": "posts"
        }
        });
    let post_body: serde_json::Value = make_hashmaps_from_dot_notation(listing);
    assert_eq!(post_body, outcome);

    let listing = vec![
        ("some.thing".into(), "15".into()),
        ("some.stuff".into(), vec!["foo", "bar"].into()),
        ("some.other.more".into(), "cool".into()),
        ("some.other.less".into(), "cool".into()),
        ("other.body".into(), "posts".into())
        ];
    let outcome = json!({
        "some": {
            "thing": "15",
            "stuff": ["foo", "bar"],
            "other": {
                "less": "cool",
                "more": "cool"
            }
        },
        "other": {
            "body": "posts"
            }
        });
    let post_body: serde_json::Value = make_hashmaps_from_dot_notation(listing);
    assert_eq!(post_body, outcome);

    let listing = vec![
        ("some.thing".into(), vec!["15"].into()),
        ("some.thing".into(), vec!["20"].into()),
        ("some.stuff".into(), vec!["foo", "bar"].into()),
        ("some.other".into(), Vec::<serde_json::Value>::new().into()),
        ("some.other.more".into(), "cool".into()),
        ("some.other.more".into(), "wow".into()),
        ("some.other.less".into(), "cool".into()),
        ("other.body".into(), "posts".into())
        ];

    let outcome = json!({
        "some": {
            "thing": ["15", "20"],
            "stuff": ["foo", "bar"],
            "other": [{
                "more": "cool"
            }, {
                "more": "wow"
            }, {
                "less": "cool"
            }],
        },
        "other": {
            "body": "posts"
            }
        });

    let post_body: serde_json::Value = make_hashmaps_from_dot_notation(listing);
    assert_eq!(post_body, outcome);

    let listing = vec![
        ("some.thing".into(), vec!["15"].into()),
        ("some.thing".into(), vec!["20"].into()),
        ("some.stuff".into(), vec!["foo", "bar"].into()),
        ("some.other.more".into(), "cool".into()),
        ("some.other".into(), Vec::<serde_json::Value>::new().into()),
        ("some.other.more".into(), "wow".into()),
        ("some.other.less".into(), "cool".into()),
        ("other.body".into(), "posts".into())
        ];

    let outcome = json!({
        "some": {
            "thing": ["15", "20"],
            "stuff": ["foo", "bar"],
            "other": [{
                "more": "cool"
            }, {
                "more": "wow"
            }, {
                "less": "cool"
            }],
        },
        "other": {
            "body": "posts"
            }
        });

    let post_body: serde_json::Value = make_hashmaps_from_dot_notation(listing);

    println!("{}", serde_json::to_string_pretty(&post_body).unwrap());
    println!("{}", serde_json::to_string_pretty(&outcome).unwrap());

    assert_eq!(post_body, outcome);

}

#[test]
fn test_creditidentials(){
    let hex = hash_credidentials("hello world".to_string());
    assert_eq!(hex, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
}