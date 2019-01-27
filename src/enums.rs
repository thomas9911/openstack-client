
use std;
use std::fmt;
use reqwest::Method;


#[derive(Debug, Clone, PartialEq, EnumIter)]
pub enum OSOperation{
    List,
    Show,
    New,
    Delete,
    Update,
    Add,
    Call,
    None,
}

impl<'a> From<&'a str> for OSOperation{
    fn from(s: &str) -> OSOperation {
        match s.to_lowercase().as_str() {
            "show" | "get" => OSOperation::Show,
            "list" | "ls" => OSOperation::List,
            "new" | "create" => OSOperation::New,
            "delete" | "remove" | "rm" => OSOperation::Delete,
            "patch" | "update" => OSOperation::Update,
            "add" | "put" | "append" => OSOperation::Add,
            "call" | "do" | "raw" => OSOperation::Call,
            _ => OSOperation::None,
        }
    }
}

impl std::str::FromStr for OSOperation{
    type Err = ();

    fn from_str(s: &str) -> Result<OSOperation, ()>{
        match OSOperation::from(s){
            OSOperation::None => Err(()),
            _ => Ok(OSOperation::from(s))
        }
    }
}

impl From<OSOperation> for String{
    fn from(s: OSOperation) -> String {
        match s {
            OSOperation::List => "list".into(),
            OSOperation::Show => "show".into(),
            OSOperation::New => "new".into(),
            OSOperation::Delete => "delete".into(),
            OSOperation::Update => "update".into(),
            OSOperation::Add => "add".into(),
            OSOperation::Call => "call".into(),
            OSOperation::None => "".into(),
        }
    }
}

impl fmt::Display for OSOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r: String = self.clone().into();
        write!(f, "{}", r)
    }
}

impl OSOperation{
    pub fn match_http_method(&self) -> Method{
        match *self{
            OSOperation::List => Method::GET,
            OSOperation::Show => Method::GET,
            OSOperation::New => Method::POST,
            OSOperation::Delete => Method::DELETE,
            OSOperation::Update => Method::PATCH,
            OSOperation::Add => Method::PUT,
            OSOperation::Call => Method::GET,
            OSOperation::None => Method::GET,
        }
    }
}

#[derive(EnumIter, Debug, Copy, Clone)]
pub enum OSResourceType{
    Compute,
    Identity,
    Networking,
    Volume,
    Images,
    None,
}

impl<'a> From<&'a str> for OSResourceType{
    fn from(s: &str) -> OSResourceType {
        match s.to_lowercase().as_str() {
            "compute" => OSResourceType::Compute,
            "volume" => OSResourceType::Volume,
            "volumev2" => OSResourceType::Volume,
            "volumev3" => OSResourceType::Volume,
            "identity" => OSResourceType::Identity,
            "network" => OSResourceType::Networking,
            "image" => OSResourceType::Images,
            _ => OSResourceType::None,
        }
    }
}

impl From<OSResourceType> for String{
    fn from(s: OSResourceType) -> String {
        match s {
            OSResourceType::Compute => "compute".into(),
            // OSResourceType::Volume => "volume".into(),
            // OSResourceType::Volume => "volumev2".into(),
            OSResourceType::Volume => "volumev3".into(),
            OSResourceType::Identity => "identity".into(),
            OSResourceType::Networking => "network".into(),
            OSResourceType::Images => "image".into(),
            OSResourceType::None => "".into(),
        }
    }
}

impl fmt::Display for OSResourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r: String = (*self).into();
        write!(f, "{}", r)
    }
}
