pub trait SerdeList{
    fn items(&self) -> Vec<(String, serde_json::Value)>;
    fn keys(&self) -> Vec<String>;
    fn values(&self) -> Vec<serde_json::Value>;
}