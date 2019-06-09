use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use yaml_rust::yaml;

use prettytable::Table;
use std::collections::HashSet;

#[allow(dead_code)]
pub fn convert_to_singular(tmp: &str) -> &str {
    // not 100% bulletproof but good enough for matching
    // let tmp = blub.to_lowercase();

    let last_char = tmp[(tmp.len() - 1)..].chars().next().unwrap();

    let is_multiple = match last_char {
        's' => true,
        'S' => true,
        _ => false,
    };

    if is_multiple {
        &tmp[..(tmp.len() - 1)]
    } else {
        tmp
    }
}

pub fn convert_to_multiple(tmp: String) -> String {
    // not 100% bulletproof but good enough for matching
    // let tmp = blub.to_lowercase();

    let last_char = tmp[(tmp.len() - 1)..].chars().next().unwrap();

    let is_multiple = match last_char {
        's' => true,
        'S' => true,
        _ => false,
    };

    if is_multiple {
        tmp
    } else {
        tmp + "s"
    }
}

pub fn add_slash(tmp: &str) -> String {
    if tmp.len() == 0{
        return String::from("/");
    }
    let last_char = tmp[(tmp.len() - 1)..].chars().next().unwrap();
    let has_slash = match last_char {
        '/' => true,
        _ => false,
    };

    if has_slash {
        tmp.to_string()
    } else {
        format!("{}/", tmp)
    }
}

pub fn remove_slash(tmp: &str) -> String {
    if tmp.len() == 0{
        return String::from("/");
    }
    tmp.trim_end_matches('/').to_string()
}

pub fn remove_slash_start(tmp: &str) -> String{
    if tmp.len() == 0{
        return String::from("/");
    }
    let first_char = tmp[0..1].chars().next().unwrap();
    let has_slash = match first_char {
        '/' => true,
        _ => false,
    };

    if has_slash {
        tmp[1..tmp.len()].to_string()
    } else {
        tmp.to_string()
    }
}

#[allow(dead_code)]
pub fn to_boolean(a_str: String) -> Option<bool> {
    // [True, ‘True’, ‘TRUE’, ‘true’, ‘1’, ‘ON’, ‘On’, ‘on’, ‘YES’, ‘Yes’, ‘yes’, ‘y’, ‘t’, False, ‘False’, ‘FALSE’, ‘false’, ‘0’, ‘OFF’, ‘Off’, ‘off’, ‘NO’, ‘No’, ‘no’, ‘n’, ‘f’]
    match a_str.to_lowercase().as_str() {
        "yes" | "y" | "1" | "on" | "t" | "true" => Some(true),
        "no" | "n" | "0" | "off" | "f" | "false" => Some(false),
        _ => None,
    }
}

pub fn read_yaml(location: String) -> Result<serde_yaml::Value, Error> {
    let file = match std::fs::File::open(&location) {
        Ok(x) => x,
        Err(e) => return Err(e),
    };
    let value: serde_yaml::Value = match serde_yaml::from_reader(file) {
        Ok(x) => x,
        Err(e) => return Err(Error::new(ErrorKind::NotFound, e.to_string())),
    };
    Ok(value)
}

#[allow(dead_code)]
pub fn read_yaml_rust(location: String) -> yaml::Yaml {
    let s = read_yaml(location).unwrap();
    yaml::Yaml::from_str(&serde_yaml::to_string(&s).unwrap())
}

pub fn compare_different_cases(a: &str, b: &str) -> bool {
    use heck::SnakeCase;
    a.to_snake_case() == b.to_snake_case()
}

pub fn get_first_value_from_hashmap_with_vec(
    map: &HashMap<String, Vec<serde_json::Value>>,
    key: &str,
) -> Option<serde_json::Value> {
    match map.get(key) {
        Some(x) => Some(x[0].clone()),
        _ => None,
    }
}

pub fn hashmap_with_vec_to_json(
    map: &HashMap<String, Vec<serde_json::Value>>
) -> serde_json::Value{

    let mut new_map = serde_json::Map::new();
    for (k, v) in map.iter(){
        if v.len() > 1 {
            // println!("{:?}", v);
            new_map.insert(k.to_string(), serde_json::Value::Array(v.clone()));
        } else{
            match v.clone().pop(){
                Some(x) => new_map.insert(k.to_string(), x.clone()),
                None => new_map.insert(k.to_string(), serde_json::Value::Null)
            };
        }
    };
    serde_json::Value::Object(new_map)
}

pub fn make_args_from_arg_matches(matches: &clap::ArgMatches) -> HashMap<String, Vec<serde_json::Value>>{
    // TODO: let the value be one of the serde_json types
    let mut options: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    for (k, v) in matches.args.iter(){
        options.insert(
            k.to_string(),
            // v.vals.iter().map(|x| serde_json::to_value(x).expect("Please don\'t try to break it")).collect()
            // v.vals.iter().map(|x| serde_json::from_str(&x.clone().into_string().expect("this is just a string")).expect("this cannot be parsed")).collect()
            v.vals.iter().map(|x| {
                serde_json::to_value(
                    x.clone().into_string().expect("this is just a string")
                ).expect("this cannot be parsed")
            }).collect()
        );
    };
    options
}

pub fn make_hashmaps_from_dot_notation(
    listing: Vec<(String, serde_json::Value)>,
) -> serde_json::Value {
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
    for (k, v) in listing {
        let mut json_string: String = String::from("");
        let mut first = true;
        for item in k.split('.').rev() {
            if first {
                json_string = format!("\"{}\": {}", item, v);
                first = false;
            } else {
                json_string = format!("\"{}\": {{{}}}", item, json_string);
            }
        }
        json_string = format!("{{{}}}", json_string);
        let j: serde_json::Value = serde_json::from_str(&json_string).unwrap();
        stuffs.push(j);
    }

    let mut end_value: serde_json::Value = serde_json::Value::Null;
    let mut first = true;
    for item in stuffs {
        if first {
            end_value = item;
            first = false;
        } else {
            end_value = merge_values(&end_value, &item);
        }
    }
    end_value
}

fn merge_values(a: &serde_json::Value, b: &serde_json::Value) -> serde_json::Value {
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
                if let serde_json::Value::Null = a {
                    *a = b.clone();
                } else {
                    tmp.insert(0, a.clone());
                    *a = serde_json::Value::Array(tmp);
                }
            } else {
                *a = b.clone();
            }
        }
    }
}

pub fn print_value(v: &serde_json::Value, f: &str) {
    let txt = match f {
        "json" => serde_json::to_string_pretty(v).unwrap(),
        "csv" => convert_to_csv(v),
        "table" => {
            let a_csv = convert_to_csv(v);
            let mut table = Table::from_csv_string(&a_csv).unwrap();
            table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            format!("{}", table)
        }
        _ => String::from(""),
    };

    println!("{}", txt);
}

fn convert_to_csv(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Object(x) => {
            let mut sorted_headers: Vec<String> = vec![];
            let mut block_data = vec![];
            for data in x.values() {
                let tmp = _do_array_thing(&data, sorted_headers, block_data);
                sorted_headers = tmp.0;
                block_data = tmp.1;
            }
            return _write_a_csv(sorted_headers, block_data);
        }
        x => {
            let mut sorted_headers: Vec<String> = vec![];
            let mut block_data = vec![];
            let tmp = _do_array_thing(x, sorted_headers, block_data);
            sorted_headers = tmp.0;
            block_data = tmp.1;
            return _write_a_csv(sorted_headers, block_data);
        }
    }
}

fn _write_a_csv(sorted_headers: Vec<String>, block_data: Vec<Vec<serde_json::Value>>) -> String {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(sorted_headers).unwrap();
    for row in block_data.iter() {
        wtr.serialize(row).unwrap();
    }

    let a_csv = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    return a_csv;
}

fn _do_array_thing(
    data: &serde_json::Value,
    mut sorted_headers: Vec<String>,
    mut block_data: Vec<Vec<serde_json::Value>>,
) -> (Vec<String>, Vec<Vec<serde_json::Value>>) {
    match data {
        serde_json::Value::Array(rows) => {
            let mut headers = HashSet::new();
            for row in rows {
                match row {
                    serde_json::Value::Object(item) => {
                        for (a, b) in item.iter() {
                            match b {
                                serde_json::Value::Object(_x) => (),
                                serde_json::Value::Array(_x) => (),
                                _x => {
                                    headers.insert(a.clone());
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
            sorted_headers = headers.iter().map(|x| x.clone()).collect();
            sorted_headers.sort();

            for row in rows {
                match row {
                    serde_json::Value::Object(item) => {
                        let mut a_row = vec![];
                        for m in sorted_headers.iter() {
                            let p = serde_json::Value::String("".into());
                            let pop = item.get(m).unwrap_or(&p);
                            match pop {
                                serde_json::Value::Object(_x) => a_row.push(p.clone()),
                                serde_json::Value::Array(_x) => a_row.push(p.clone()),
                                x => a_row.push(x.clone()),
                            }
                        }
                        block_data.push(a_row);
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }
    (sorted_headers, block_data)
}

fn remove_empty_fields(val: &mut serde_json::Value){
    let mut delete_keys = vec![];
    if let Some(x) = val.as_object(){
        for (k, v) in x.iter(){
            if v == ""{
                delete_keys.push(k.clone())
            }
        };
    };
    if let Some(x) = val.as_object_mut(){
        for key in delete_keys{
            x.remove(&key);
        };
    };
}


#[test]
fn test_convert_to_singular() {
    assert_eq!(convert_to_singular("TESTS"), "TEST");
    assert_eq!(convert_to_singular("TEST"), "TEST");
    assert_eq!(convert_to_singular("tests"), "test");
    assert_eq!(convert_to_singular("Yes"), "Ye");
}

#[test]
fn test_convert_to_multiple() {
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
fn test_add_slash() {
    assert_eq!(add_slash("test.com"), "test.com/");
    assert_eq!(add_slash("test.com/"), "test.com/");
}

#[test]
fn test_remove_slash() {
    assert_eq!(remove_slash("test.com///"), "test.com");
    assert_eq!(remove_slash("test.com/"), "test.com");
    assert_eq!(remove_slash("test.com/oke/"), "test.com/oke");
}

#[test]
fn test_compare_different_cases() {
    assert_eq!(compare_different_cases("HalloTest", "hallo_test"), true);
    assert_eq!(compare_different_cases("Hallo Test", "hallo-test"), true);
    assert_eq!(compare_different_cases("halloTest", "hallo test"), true);
    assert_eq!(compare_different_cases("hallotest", "Hallo Test"), false);
    assert_eq!(compare_different_cases("Hallo", "test"), false);
}

#[test]
fn test_merge_values() {
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
fn test_merge_values_2() {
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
fn test_merge_values_3() {
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
fn test_hashmaps_from_dot_notation() {
    let listing = vec![
        ("some.thing".into(), "15".into()),
        ("some.stuff".into(), "foo".into()),
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
        ("some.stuff".into(), vec!["foo", "bar"].into()),
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
        ("other.body".into(), "posts".into()),
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
        ("other.body".into(), "posts".into()),
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
        ("other.body".into(), "posts".into()),
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
        ("other.body".into(), "posts".into()),
        ("other.number".into(), 15.into()),
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
        "body": "posts",
        "number": 15
        }
    });

    let post_body: serde_json::Value = make_hashmaps_from_dot_notation(listing);

    println!("{}", serde_json::to_string_pretty(&post_body).unwrap());
    println!("{}", serde_json::to_string_pretty(&outcome).unwrap());

    assert_eq!(post_body, outcome);
}

// #[test]
// fn test_creditidentials(){
//     let hex = hash_credidentials("hello world".to_string());
//     assert_eq!(hex, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
// }

#[test]
fn test_json_convert_to_csv_for_particular_jsons() {
    let a_json = json!({
        "other": [{
            "more": "cool"
        }, {
            "more": "wow",
            "less": "top"
        }, {
            "less": 15,
            "more": [
                "this", "data", "is", "now", "gone"
            ]
        }, {
            "more": true
        }],
    });

    let a_csv = "less,more\n,cool\ntop,wow\n15,\n,true\n";

    assert_eq!(a_csv, convert_to_csv(&a_json));
}

#[test]
fn test_json_convert_to_csv_for_particular_jsons_2() {
    let a_json = json!([{
        "more": "cool"
    }, {
        "more": "wow",
        "less": "top"
    }, {
        "less": 15,
        "more": [
            "this", "data", "is", "now", "gone"
        ]
    }, {
        "more": true
    }]);

    let a_csv = "less,more\n,cool\ntop,wow\n15,\n,true\n";

    assert_eq!(a_csv, convert_to_csv(&a_json));
}


#[test]
fn test_hashmap_with_vec_to_json(){
    let mut h: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    h.insert("test".to_string(), vec!["hallo".into()]);
    h.insert("test2".to_string(), vec!["hallo".into(), "testing".into()]);
    h.insert("test3".to_string(), vec![]);

    let expected = json!({
        "test": "hallo",
        "test2": ["hallo", "testing"],
        "test3": null,
    });

    assert_eq!(expected, hashmap_with_vec_to_json(&h));
}

#[test]
fn test_remove_fields(){
    let mut data = json!({
        "auth-url": "https://example.com/",
        "domain-id": "",
        "domain-name": "",
        "password": "",
        "project-domain-id": "",
        "project-domain-name": "",
        "project-id": "",
        "project-name": "",
        "system-scope": "",
        "token": "",
        "trust-id": "",
        "user-domain-id": "1234-124",
        "user-domain-name": "",
        "user-id": "",
        "username": "Test"
    });
    let expected_output = json!({
        "user-domain-id": "1234-124",
        "auth-url": "https://example.com/",
        "username": "Test"
    });
    remove_empty_fields(&mut data);

    assert_eq!(data, expected_output);
}