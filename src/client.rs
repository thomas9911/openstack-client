
use std::collections::HashMap;
use std::io::{stdout, Read, Write};
use std::fs::File;

use curl::easy::{Easy, List};
use serde_json::Value as JSONValue;
use error::OpenstackError;
use memmap::MmapOptions;
use indicatif::{ProgressBar, ProgressStyle};

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
pub struct Response ( JSONValue, u32, Vec<String> );


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
            }).unwrap();

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

    // pub fn upload_to_object_store() -> Result<Response, OpenstackError> {
    //     upload_to_object_store(file: &mut File, name: &str, container: &str, object_store_url: &str, token: &str)
    // }
}


fn upload_to_object_store(
    file: &mut File,
    name: &str,
    container: &str,
    object_store_url: &str,
    token: &str,
) -> Result<String, OpenstackError> {
    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;

    let mut data = Vec::new();

    let mut easy = Easy::new();
    easy.url(&format!("{}/{}/{}", object_store_url, container, name))?;
    easy.upload(true)?;
    easy.http_headers(headers)?;
    easy.progress(true)?;

    {
        let progress_bar = make_progress_bar(file.metadata()?.len());
        let mut transfer = easy.transfer();
        transfer.progress_function(|_a, _b, _c, d| {
            if d as u32 != 0 {
                progress_bar.set_position(d as u64);
            }
            true
        })?;
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
        transfer.read_function(|into| Ok(file.read(into).unwrap()))?;
        transfer.perform()?;
        progress_bar.finish();
    }

    // println!("{:?}", String::from_utf8(data).unwrap_or(String::from("")));
    Ok(String::from_utf8(data).unwrap_or(String::from("")))
}

fn download_from_object_store(
    file: &mut File,
    name: &str,
    container: &str,
    object_store_url: &str,
    token: &str,
) -> Result<(String, u32), OpenstackError> {
    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;

    let urlpath = &format!("{}/{}/{}", object_store_url, container, name);

    let mut easy = Easy::new();
    easy.url(urlpath)?;
    easy.get(true)?;
    easy.http_headers(headers)?;
    easy.progress(true)?;

    {
        let progress_bar = make_progress_bar(0);
        let mut transfer = easy.transfer();
        transfer.progress_function(|a, b, _c, _d| {
            if a as u32 != 0 {
                progress_bar.set_length(a as u64);
                progress_bar.set_position(b as u64);
            }
            true
        })?;
        transfer.write_function(|data| Ok(file.write(data).unwrap()))?;
        transfer.perform()?;
        progress_bar.finish();
    }

    let statuscode = easy.response_code()?;

    Ok((String::from(""), statuscode))
}


fn upload_to_object_store_dynamic_large_objects(
    file: &File,
    name: &str,
    container: &str,
    object_store_url: &str,
    token: &str,
    parts: usize,
) -> Result<(String, u32), OpenstackError> {
    let fileurl = format!("{}/{}/{}", object_store_url, container, name);

    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;

    let mut easy = Easy::new();
    easy.url(&fileurl)?;
    easy.upload(true)?;
    easy.http_headers(headers)?;
    easy.write_function(|into| Ok(stdout().write(into).unwrap()))?;
    easy.perform()?;

    let mut responses = Vec::new();
    let mut success = true;
    {
        let mmap = unsafe { MmapOptions::new().map(file)? };
        let amount = (mmap.len() as f32 / (parts as f32 - 0.1)) as usize;

        let chuckies = mmap.chunks(amount);
        for (index, mut chunk) in chuckies.enumerate() {
            let response = upload_part(&mut chunk, index, &fileurl, token)?;
            let text = response.0;
            let statuscode = response.1;
            if statuscode != 201 {
                responses.push((text, statuscode));
                success = false;
                break;
            }
        }
    }
    if success == true {
        set_dynamic_manifest(&fileurl, container, name, token)?;
    }

    match success {
        true => Ok((String::from(""), 200)),
        false => Ok(responses[0].clone()),
    }
}

fn open_file(filename: &str) -> Result<File, OpenstackError> {
    let filepath = is_file(filename)?;
    Ok(File::open(filepath)?)
}

fn create_file(filename: &str) -> Result<File, OpenstackError> {
    make_sure_folder_exists(filename)?;
    Ok(File::create(filename)?)
}

fn is_file(filename: &str) -> Result<std::path::PathBuf, OpenstackError> {
    let filepath = std::path::PathBuf::from(filename);
    if !filepath.is_file() {
        return Err(
            OpenstackError::new(&format!("'{}' does not exist", filename))
        );
    }
    Ok(filepath)
}

fn make_sure_folder_exists(filename: &str) -> Result<(), OpenstackError> {
    let filepath = std::path::PathBuf::from(filename);
    let parent = match filepath.parent(){
        Some(x) => x.to_path_buf(),
        None => return Ok(())
    };
    std::fs::DirBuilder::new().recursive(true).create(parent)?;
    Ok(())
}

fn set_dynamic_manifest(
    fileurl: &str,
    container: &str,
    filename: &str,
    token: &str,
) -> Result<(String, u32), OpenstackError> {
    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;
    headers.append(&format!("X-Object-Manifest: {}/{}/", container, filename))?;

    let mut data = Vec::new();
    let mut easy = Easy::new();
    easy.url(&fileurl)?;
    easy.upload(true)?;
    easy.http_headers(headers)?;
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }
    Ok((
        String::from_utf8(data).unwrap_or(String::from("")),
        easy.response_code()?,
    ))
}

fn upload_part(
    chunk: &mut &[u8],
    index: usize,
    fileurl: &str,
    token: &str,
) -> Result<(String, u32), OpenstackError> {
    let mut headers = List::new();
    headers.append(&format!("X-Auth-Token: {}", token))?;

    let mut data = Vec::new();
    let mut easy = Easy::new();
    easy.url(&format!("{}/{:08}", fileurl, index))?;
    easy.upload(true)?;
    easy.progress(true)?;
    easy.http_headers(headers)?;
    {
        let progress_bar = make_progress_bar(chunk.len() as u64);
        let mut transfer = easy.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.progress_function(|_a, _b, _c, d| {
            if d as u32 != 0 {
                progress_bar.set_position(d as u64);
            }
            true
        })?;
        transfer.read_function(|into| Ok(chunk.read(into).unwrap()))?;
        transfer.perform()?;
        progress_bar.finish();
    }
    Ok((
        String::from_utf8(data).unwrap_or(String::from("")),
        easy.response_code()?,
    ))
}

fn make_progress_bar(length: u64) -> ProgressBar{
    let progress_bar = ProgressBar::new(length);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {wide_bar:.cyan/blue} {bytes:>7}/{total_bytes:7}"),
    );
    progress_bar
}