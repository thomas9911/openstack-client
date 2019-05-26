#[macro_export]
macro_rules! make_list {
    ($t:ty) => {
        impl $crate::traits::SerdeList for $t {
            fn items(&self) -> Vec<(String, serde_json::Value)> {
                let value: serde_json::Value = serde_json::to_value(self).unwrap();
                match value.as_object() {
                    Some(x) => x.iter().map(|x| (x.0.clone(), x.1.clone())).collect(),
                    None => return vec![],
                }
            }

            fn keys(&self) -> Vec<String> {
                let value: serde_json::Value = serde_json::to_value(self).unwrap();
                match value.as_object() {
                    Some(x) => x.iter().map(|x| x.0.clone()).collect(),
                    None => return vec![],
                }
            }

            fn values(&self) -> Vec<serde_json::Value> {
                let value: serde_json::Value = serde_json::to_value(self).unwrap();
                match value.as_object() {
                    Some(x) => x.iter().map(|x| x.1.clone()).collect(),
                    None => return vec![],
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use traits::SerdeList;

    #[derive(Debug, Serialize, Deserialize)]
    struct Testing {
        x: String,
        y: i32,
        z: Vec<String>,
    }
    make_list!(Testing);

    #[test]
    fn make_list_macro_items() {
        use serde_json::Value;

        let test = Testing {
            x: "testing".to_string(),
            y: 13,
            z: vec!["testing".to_string(), "test".to_string()],
        };

        assert_eq!(
            test.items(),
            vec![
                ("x".to_string(), Value::from("testing")),
                ("y".to_string(), Value::from(13)),
                ("z".to_string(), Value::from(vec!["testing".to_string(), "test".to_string()]))
            ]
        )
    }

    #[test]
    fn make_list_macro_values() {
        use serde_json::Value;

        let test = Testing {
            x: "testing".to_string(),
            y: 13,
            z: vec!["testing".to_string(), "test".to_string()],
        };

        assert_eq!(
            test.values(),
            vec![
                Value::from("testing"),
                Value::from(13),
                Value::from(vec!["testing".to_string(), "test".to_string()])
            ]
        )
    }

    #[test]
    fn make_list_macro_keys() {

        let test = Testing {
            x: "testing".to_string(),
            y: 13,
            z: vec!["testing".to_string(), "test".to_string()],
        };

        assert_eq!(
            test.keys(),
            vec!["x".to_string(), "y".to_string(), "z".to_string()]
        )
    }
}
