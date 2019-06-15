
use std::collections::HashMap;
use std::io::{stdout, Read, Write};
use std::fs::File;

use curl::easy::{Easy, List};
use serde_json::Value as JSONValue;
use error::OpenstackError;



use objectstore::{create_file, download_from_object_store, open_file, upload_to_object_store, upload_to_object_store_dynamic_large_objects};

#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
    #[serde(skip, default = "Easy::new")]
    pub handle: Easy,
    pub headers: HashMap<String, String>,
    pub url: Option<String>,
    pub method: Option<String>,
    pub json: JSONValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response ( pub JSONValue, pub u32, pub Vec<String> );


impl Response{
    pub fn status(&self) -> u32{
        self.1
    }

    pub fn response(&self) -> JSONValue{
        self.0.clone()
    }

    pub fn headers(&self) -> Vec<String>{
        self.2.clone()
    }

    pub fn parsed_headers(&self) -> HashMap<String, String>{
        let mut headers = HashMap::new();
        for item in self.headers(){
            let new_item = item.clone();
            let mut split = new_item.split(':');
            let key = match split.next(){
                Some(x) => x.trim(),
                None => continue
            };
            let value = match split.next(){
                Some(x) => x.trim(),
                None => continue
            };
            headers.insert(key.to_string(), value.to_string());
        }
        headers
    }

    pub fn is_success(&self) -> bool{
        !((self.1 / 100 == 4) | (self.1 / 100 == 5))
    }
}

impl Default for Response{
    fn default() -> Self{
        Response{0: JSONValue::Null, 1: 0, 2: vec![]}
    }
}

impl Client {
    pub fn new() -> Self {
        let handle = Easy::new();
        let headers: HashMap<String, String> = HashMap::new();
        let url = None;
        let method = None;
        let json = JSONValue::Null;
        Client { handle, headers, url, method, json }
    }

    pub fn set_token(&mut self, token: &str) {
        self.set_header("x-auth-token", token);
    }

    pub fn get_token(&mut self) -> Option<&String> {
        self.headers.get("x-auth-token")
    }

    pub fn set_header(&mut self, header: &str, header_value: &str) {
        self.headers
            .insert(header.to_string(), header_value.to_string());
    }

    pub fn set_url(&mut self, url: &str){
        self.url = Some(String::from(url));
    }

    pub fn set_method(&mut self, method: &str){
        self.method = Some(String::from(method));
    }

    pub fn set_json(&mut self, json: JSONValue){
        self.json = json;
    }

    pub fn perform(&mut self) -> Result<Response, OpenstackError>{
        self.check_valid()?;
        let method = self.method.clone().expect("this is checked to be not none");
        let url = self.url.clone().expect("this is checked to be not none");
        let json = self.json.clone();
        self.request(&method, &url, json)
    }

    fn check_valid(&mut self) -> Result<(), OpenstackError>{
        let mut errors = vec![];
        if self.url.is_none(){
            errors.push("url is not set")
        }
        if self.method.is_none(){
            errors.push("method is not set")
        }
        if !errors.is_empty(){
            let mut e = String::from("");
            for item in errors{
                if e == String::from(""){
                    e = item.to_string();
                } else{
                    e = format!("{} and {}", e, item);
                };
            }
            return Err(OpenstackError::new(&e))
        }
        Ok(())
    }

    fn headers_to_list(hashmap: HashMap<String, String>) -> List {
        let mut local_headers = List::new();
        for (header, header_value) in hashmap.iter() {
            local_headers
                .append(&format!("{}: {}", header, header_value))
                .unwrap()
        }
        local_headers
    }

    pub fn request(
        &mut self,
        method: &str,
        url: &str,
        json: JSONValue,
    ) -> Result<Response, OpenstackError> {
        let mut data = Vec::new();

        self.handle.url(url)?;
        self.handle.custom_request(method)?;

        let mut local_headers = Self::headers_to_list(self.headers.clone());

        match method.to_lowercase().as_ref() {
            "post" => {
                self.handle.post_fields_copy(json.to_string().as_bytes())?;
                local_headers.append("Content-Type: application/json")?;
            }
            "put" | "patch" => {
                self.handle.upload(true)?;
                self.handle.in_filesize(json.to_string().len() as u64)?;
                local_headers.append("Content-Type: application/json")?;
            }
            _ => {}
        }
        self.handle.http_headers(local_headers)?;

        let mut remote_headers = vec![];

        {
            let body = json.to_string();
            let mut transfer = self.handle.transfer();
            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
            transfer.header_function(|header| {
                remote_headers.push(String::from_utf8(header.to_vec()).unwrap_or(String::from(" : ")));
                true
            })?;

            match method.to_lowercase().as_ref() {
                "put"| "patch" => transfer.read_function(|into| Ok(body.as_bytes().read(into).unwrap()))?,
                _ => ()
            };
            transfer.perform()?;
        }
        // self.handle.send(format!("{}", json).as_bytes())?;

        let rv = String::from_utf8(data).unwrap_or("".to_string());
        let response_data = match rv.parse(){
            Ok(x) => x,
            Err(_e) => rv.into()
        };

        Ok(Response{0: response_data, 1: self.handle.response_code()?, 2: remote_headers})
        // Ok((rv.parse().unwrap(), self.handle.response_code()?))
    }
    pub fn get(&mut self, url: &str) -> Result<Response, OpenstackError> {
        self.request("GET", url, JSONValue::String("".to_string()))
    }
    pub fn post(&mut self, url: &str, json: JSONValue) -> Result<Response, OpenstackError> {
        self.request("POST", url, json)
    }
    pub fn put(&mut self, url: &str, json: JSONValue) -> Result<Response, OpenstackError> {
        self.request("PUT", url, json)
    }
    pub fn patch(&mut self, url: &str, json: JSONValue) -> Result<Response, OpenstackError> {
        self.request("PATCH", url, json)
    }
    pub fn delete(&mut self, url: &str) -> Result<Response, OpenstackError> {
        self.request("DELETE", url, JSONValue::String("".to_string()))
    }
    pub fn option(&mut self, url: &str) -> Result<Response, OpenstackError> {
        self.request("OPTION", url, JSONValue::String("".to_string()))
    }
    pub fn head(&mut self, url: &str) -> Result<Response, OpenstackError> {
        self.request("HEAD", url, JSONValue::String("".to_string()))
    }

    fn open_file_ect(&mut self, filename: &str) -> Result<(File, String), OpenstackError> {
        let token = match self.get_token(){
            Some(x) => x,
            None => return Err(OpenstackError::new("token is not set"))
        };

        let file = open_file(filename)?;

        Ok((file, token.to_string()))
    }

    pub fn upload_to_object_store(&mut self, filename: &str, objectstore_url: &str) -> Result<Response, OpenstackError> {
        let (mut file, token) = self.open_file_ect(filename)?;
        upload_to_object_store(&mut file, objectstore_url, &token)
    }

    pub fn upload_to_object_store_large(&mut self, filename: &str, objectstore_url: &str, container: &str, name: &str) -> Result<Response, OpenstackError> {
        let (mut file, token) = self.open_file_ect(filename)?;
        upload_to_object_store_dynamic_large_objects(&mut file, name, container, objectstore_url, &token, 20, 0)
    }

    pub fn upload_to_object_store_large_with_parts(&mut self, filename: &str, objectstore_url: &str, container: &str, name: &str, parts: usize) -> Result<Response, OpenstackError> {
        let (mut file, token) = self.open_file_ect(filename)?;
        upload_to_object_store_dynamic_large_objects(&mut file, name, container, objectstore_url, &token, parts, 0)
    }

    pub fn upload_to_object_store_large_skip_parts(&mut self, filename: &str, objectstore_url: &str, container: &str, name: &str, parts: usize, skip_first: usize) -> Result<Response, OpenstackError> {
        let (mut file, token) = self.open_file_ect(filename)?;
        upload_to_object_store_dynamic_large_objects(&mut file, name, container, objectstore_url, &token, parts, skip_first)
    }

    pub fn download_from_object_store(&mut self, outfile: &str, objectstore_url: &str) -> Result<Response, OpenstackError> {
        let token = match self.get_token(){
            Some(x) => x,
            None => return Err(OpenstackError::new("token is not set"))
        };
        let mut file = create_file(outfile)?;
        download_from_object_store(&mut file, objectstore_url, token)
    }
}
