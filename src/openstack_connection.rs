use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::iter::DoubleEndedIterator;
use std::str::FromStr;

use chrono::prelude::*;
use chrono::Duration;

use enums::OSOperation;
use structs::{Action, ActionMap, Command, CommandMap, Resource, ResourceMap, ResourceTypeEnum};
use utils::{
    add_slash, get_first_value_from_hashmap_with_vec, hashmap_with_vec_to_json,
    make_hashmaps_from_dot_notation, read_yaml, remove_slash_start,
};
use uuid::Uuid;

use config::{OpenstackInfoMap, OpenstackTokenizer};
use client::{Client, Response};
use error::OpenstackError;


#[derive(Debug, Serialize, Deserialize)]
pub struct OpenstackConnection {
    pub config: OpenstackTokenizer,
    // #[serde(skip, default = "reqwest::Client::new")]
    // pub client: reqwest::Client,
    pub client: Client,
    pub token: Option<String>,
    pub token_expiry: Option<String>,
    pub endpoints: Option<HashMap<String, String>>,
    pub domain_id: Option<String>,
    pub user_id: Option<String>,
}

impl OpenstackConnection {
    pub fn new(config: OpenstackInfoMap) -> OpenstackConnection {
        // let client = reqwest::Client::new();
        // let client = reqwest::Client::builder()
        //     // .http1_title_case_headers()
        //     // .use_rustls_tls()
        //     // .danger_accept_invalid_certs(true)
        //     // .referer(true)
        //     .build()
        //     .unwrap();
        let client = Client::new();
        let config = OpenstackTokenizer::new(config);
        OpenstackConnection {
            config,
            client,
            token: None,
            token_expiry: None,
            endpoints: None,
            domain_id: None,
            user_id: None,
        }
    }

    // #[allow(dead_code)]
    // pub fn get<T: reqwest::IntoUrl>(&mut self, url: T) -> reqwest::RequestBuilder {
    //     self.request(reqwest::Method::GET, url)
    // }

    // #[allow(dead_code)]
    // pub fn post<T: reqwest::IntoUrl>(&mut self, url: T) -> reqwest::RequestBuilder {
    //     self.request(reqwest::Method::POST, url)
    // }

    // pub fn request<T: reqwest::IntoUrl>(
    //     &mut self,
    //     method: reqwest::Method,
    //     url: T,
    // ) -> reqwest::RequestBuilder {
    //     let mut headers = reqwest::header::HeaderMap::new();

    //     let expire_time: DateTime<Utc> = match &self.token_expiry {
    //         Some(x) => DateTime::parse_from_rfc3339(&x)
    //             .expect("no token expiry set")
    //             .with_timezone(&Utc),
    //         _ => DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1500, 0), Utc),
    //     };

    //     if expire_time - Duration::minutes(5) <= Utc::now() {
    //         &self.refresh_token().expect("error while refreshing token");
    //     }

    //     let token = self.token.clone();
    //     headers.insert(
    //         "X-Auth-Token",
    //         token.expect("a valid token").parse().unwrap(),
    //     );
    //     self.client.request(method, url).headers(headers)
    // }

    pub fn refresh_token(&mut self) -> Result<(), OpenstackError> {
        self.config.refresh_token()?;
        self.token = self.config.token.clone();
        match self.token.as_ref(){
            Some(x) => self.client.set_token(x),
            None => return Err(OpenstackError::new("something went wrong setting the token"))
        }
        self.token_expiry = self.config.token_expiry.clone();
        self.endpoints = self.config.endpoints.clone();
        self.domain_id = self.config.domain_id.clone();
        self.user_id = self.config.user_id.clone();
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Openstack {
    pub connection: OpenstackConnection,
    pub resources: ResourceMap,
    pub actions: ActionMap,
    pub commands: CommandMap,
}

impl Openstack {
    pub fn new(config: OpenstackInfoMap) -> Result<Self, OpenstackError> {
        let mut connection = OpenstackConnection::new(config);
        connection.refresh_token()?;
        let mut rc = ResourceMap::new();
        if let Some(x) = &connection.endpoints {
            rc.update_from_identity(x)
        };
        let ac = ActionMap::new();
        let cm = CommandMap::new();
        Ok(Openstack {
            connection,
            resources: rc,
            actions: ac,
            commands: cm,
        })
    }

    pub fn refresh_token(&mut self) -> Result<&mut Openstack, OpenstackError> {
        self.connection.refresh_token()?;
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn list(&mut self, res: String) -> Result<serde_json::Value, OpenstackError> {
        self.act(
            "list".to_string(),
            res.clone(),
            &HashMap::new(),
            &HashMap::new(),
        )
    }

    #[allow(dead_code)]
    pub fn delete(self, res: String, id: String) {}

    #[allow(dead_code)]
    pub fn get(self, res: String, id: String) {}

    #[allow(dead_code)]
    pub fn update(self, res: String, id: String) {}

    pub fn act(
        &mut self,
        op: String,
        res: String,
        op_args: &HashMap<String, Vec<serde_json::Value>>,
        res_args: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<serde_json::Value, OpenstackError> {
        if self.connection.endpoints.is_none() {
            self.refresh_token().expect("error while refreshing token");
        }
        let r = match self.resources.get_resource(res) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };

        let is_dry_run = match op_args.get("dry-run") {
            Some(_x) => true,
            None => false,
        };

        // let endpoint: String = match r.resource_type.clone(){
        //     ResourceTypeEnum::ResourceType(x) => x.endpoint,
        //     ResourceTypeEnum::String(x) => x,
        // };

        let mut post_body;

        let mut path = r.endpoint_path.clone();
        path = match get_first_value_from_hashmap_with_vec(res_args, "id") {
            Some(id) => format!("{}{}", add_slash(&path), id.as_str().unwrap()),
            None => path,
        };
        let renderer = handlebars::Handlebars::new();
        path = renderer.render_template(&path, &json!({"user_id": self.connection.user_id, "domain_id": self.connection.domain_id}))?;

        let http_method: http::Method;
        let matched_op;
        let maybe_action = self.actions.get_action(op.clone(), r.name.clone());
        if let Some(ref action) = maybe_action {
            let mut modified_res_args = res_args.clone();
            if get_first_value_from_hashmap_with_vec(res_args, "name") == None {
                if let Some(x) = res_args.get("file") {
                    modified_res_args.insert("name".to_string(), x.clone());
                }
            };

            post_body = action.make_body(&modified_res_args);
            let tmp_json = hashmap_with_vec_to_json(&modified_res_args);
            path = format!("{}{}", add_slash(&path), action.url_parameter);
            path = renderer.render_template(&path, &tmp_json).unwrap();
            http_method = http::Method::from_str(&action.http_method).expect(&format!(
                "the method {} from {} {} is not valid",
                action.http_method, action.action, action.resource
            ));
            matched_op = self
                .commands
                .map
                .get(&action.http_method)
                .expect("http method not mapped for action")
                .clone();
        } else if let Ok(op_parsed) = OSOperation::from_str(&op) {
            http_method = op_parsed.match_http_method();
            matched_op = self
                .commands
                .map
                .get(&op_parsed.to_string())
                .expect("commands is not complete")
                .clone();
            post_body = Openstack::handle_post_parameters(&r, &matched_op, res_args);
        } else {
            return Err(OpenstackError::new(&format!("'{}' is not a valid operation", &op)))
        }

        // let op_parsed = match OSOperation::from_str(&op){
        //     Ok(x) => x,
        //     Err(_e) => return Err(Error::new(ErrorKind::Other, format!("'{}' is not a valid operation", &op)))
        // };
        self._handle_special_body_parameters(&r, &matched_op, &mut post_body);

        // let prepared_url = self.make_url(
        //     matched_op,
        //     r.resource_type.clone(),
        //     path,
        //     maybe_action,
        //     Some(res_args),
        // )?;

        // println!("{:?}", matched_op);
        // println!("{:?}", r.resource_type);
        // println!("{:?}", path);
        // println!("{:?}", maybe_action);
        // println!("{:?}", res_args);

        self.make_url(
            matched_op,
            r.resource_type.clone(),
            path,
            &maybe_action,
            Some(res_args),
        );

        if is_dry_run {
            // let t = prepared_url.build().expect("Request cannot be build");
            println!("{:?} {:?}\nHeaders: {:?}", self.connection.client.method, self.connection.client.url, self.connection.client.headers);
            // return match t.body() {
            //     Some(x) => Ok(format!("{:?}", x).into()),
            //     None => Ok(post_body),
            // };
            return Ok(post_body);
            // return Ok(post_body)
        }

        // let mut response = match &post_body {
        //     serde_json::Value::Null => match prepared_url.send() {
        //         Ok(x) => x,
        //         Err(e) => return Err(OpenstackError::new(&format!("{}", e))),
        //     },
        //     _ => match prepared_url.json(&post_body).send() {
        //         Ok(x) => x,
        //         Err(e) => return Err(OpenstackError::new(&format!("{}", e))),
        //     },
        // };
        let mut response = Response::default();
        let mut matched_action = false;

        if let Some(act) = maybe_action{
            if (act.action == "upload") && (act.resource == "objects"){
                matched_action = true;
                let url = match self.connection.client.url.clone(){
                    Some(x) => format!("{}", x),
                    None => return Err(OpenstackError::new("url argument is required"))
                };
                let file = get_value(res_args, "file")?;

                match get_value(res_args, "parts"){
                    Ok(parts_string) => {
                        let container = get_value(res_args, "container")?;
                        let name = get_value(res_args, "name")?;
                        let skip_parts_string = get_value(res_args, "skip-parts")?;
                        let skip_first: usize = match skip_parts_string.parse(){
                            Ok(z) => z,
                            Err(e) => return Err(OpenstackError::new(&format!("{}", e)))
                        };
                        let parts: usize = match parts_string.parse(){
                            Ok(z) => z,
                            Err(e) => return Err(OpenstackError::new(&format!("{}", e)))
                        };
                        response = self.connection.client.upload_to_object_store_large_skip_parts(&file, &url, &container, &name, parts, skip_first)?;
                    },
                    _ => {
                        response = self.connection.client.upload_to_object_store(&file, &url)?;
                    }
                };
            }
            if (act.action == "download") && (act.resource == "objects"){
                matched_action = true;
                let url = match self.connection.client.url.clone(){
                    Some(x) => format!("{}", x),
                    None => return Err(OpenstackError::new("url argument is required"))
                };
                let file = get_value(res_args, "file")?;
                response = self.connection.client.download_from_object_store(&file, &url)?;
            }
        }
        if !matched_action{
            self.connection.client.set_json(post_body);
            response = self.connection.client.perform()?;
        }
        Openstack::handle_response(&mut response)
    }



    pub fn make_url(
        &mut self,
        com: Command,
        rt: ResourceTypeEnum,
        path: String,
        action: &Option<Action>,
        res_args: Option<&HashMap<String, Vec<serde_json::Value>>>,
    ) {
        let endpoint: String = match rt.clone() {
            ResourceTypeEnum::ResourceType(x) => x.endpoint,
            ResourceTypeEnum::String(x) => x,
        };
        // let endpoint = "https://httpbin.org/anything";
        let new_path = format!("{}{}", add_slash(&endpoint), remove_slash_start(&path));
        let method = http::Method::from_str(&com.http_method)
            .expect("command has not a valid http method");

        // if let Some(act) = action{
        //     if act.is_multipart{
        //         if let Some(ra) = res_args{
        //             if let Some(file_path_value) = get_first_value_from_hashmap_with_vec(ra, "file"){
        //                 let file_path = file_path_value.as_str().expect("file is a string");
        //                 // let patn_ok = format!("{}{}",add_slash("https://httpbin.org/anything"), remove_slash_start(&path));
        //                 // return Ok(self.connection.request_file(method, &patn_ok, file_path)?)
        //                 return Ok(self.connection.request_file(method, &new_path, file_path)?)
        //             }
        //         }
        //     }
        // }

        // Ok(self.connection.request(method, &new_path))
        // Ok(new_path)
        self.connection.client.set_method(&method.as_str().to_uppercase());
        self.connection.client.set_url(&new_path);
    }

    pub fn handle_response(response: &mut Response) -> Result<serde_json::Value, OpenstackError> {
        // if !response.status().is_success() {
        //     let error = match response.json::<serde_json::Value>() {
        //         Ok(x) => x,
        //         Err(_e) => json!({
        //             "error": response.text().unwrap()
        //         }),
        //     };
        //     return Err(OpenstackError::new(
        //         &serde_json::to_string_pretty(&error).unwrap()
        //     ));
        // }
        // match response.json::<serde_json::Value>() {
        //     Ok(x) => return Ok(x),
        //     Err(_e) => (), // Err(_e) => return Err(Error::new(ErrorKind::InvalidData, "Response is not valid json"))
        // };
        // match response.text() {
        //     Ok(x) => Ok(json!({ "response": x })),
        //     Err(_e) => Err(OpenstackError::new(
        //         "Response cannot be parsed",
        //     )),
        // }
        if !response.is_success() {
            let error = json!({
                "error": response.response()
            });
            return Err(OpenstackError::new(&serde_json::to_string_pretty(&error).unwrap()));
        }
        Ok(response.response().clone())
    }

    fn handle_post_parameters(
        res: &Resource,
        op: &Command,
        res_args: &HashMap<String, Vec<serde_json::Value>>,
    ) -> serde_json::Value {
        if op.has_body == false {
            return serde_json::Value::Null;
        }


        if let Some(ref post_param) = res.post_parameters {
            let mut data: Vec<(String, serde_json::Value)> = vec![];
            for item in post_param {
                let path = item.path.clone();
                if item.hidden {
                    data.push((path.clone(), Vec::<serde_json::Value>::new().into()))
                }
                let the_value: serde_json::Value;
                if let Some(x) = res_args.get(&item.name) {
                    if item.multiple {
                        the_value = x.clone().into();
                    } else {
                        the_value = x[0].clone().into()
                    }
                } else {
                    if let Some(x) = &item.default {
                        the_value = x.clone().into();
                    } else {
                        continue;
                    }
                }
                match item.the_type.as_ref() {
                    "string" => data.push((path.clone(), the_value)),
                    "number" => {
                        let v = match serde_json::Number::from_str(&the_value.as_str().unwrap()) {
                            Ok(x) => serde_json::Value::Number(x),
                            Err(_e) => the_value.clone(),
                        };
                        data.push((path.clone(), v));
                    }
                    _ => {}
                }
            }
            return make_hashmaps_from_dot_notation(data);
        };
        serde_json::Value::Null
    }

    fn _handle_special_body_parameters(
        &self,
        res: &Resource,
        com: &Command,
        body: &mut serde_json::Value,
    ) {
        if res.name == "credentials" {
            if let Some(ref mut x) = body.get_mut("credentials") {
                let blob = json!({"access": Uuid::new_v4(), "secret": Uuid::new_v4()});
                if let Some(y) = x.get_mut("blob") {
                    *y = format!("{}", blob).into();
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn resource_available(&self, res: String) -> Option<Resource> {
        let available = self.is_resource_available(res.clone());

        if available {
            if let Ok(resource) = self.resources.get_resource(res) {
                return Some(resource.clone());
            };
        };
        None
    }

    pub fn is_resource_available(&self, res: String) -> bool {
        let res = match self.resources.get_resource(res) {
            Ok(x) => x,
            Err(_e) => return false,
        };

        match &res.resource_type {
            ResourceTypeEnum::ResourceType(_x) => true,
            _ => false,
        }
    }
}

fn get_value(hashmap: &HashMap<String, Vec<serde_json::Value>>, key: &str) -> Result<String, OpenstackError>{
    let string = match get_first_value_from_hashmap_with_vec(hashmap, key){
        Some(x) => match x{
            serde_json::Value::String(y) => y,
            _ => return Err(OpenstackError::new(&format!("{} argument is required", key)))
        },
        None => return Err(OpenstackError::new(&format!("{} argument is required", key)))
    };
    Ok(string)
}

// list ec2 credentials /users/<user_id>/credentials/OS-EC2

// #[test]
// fn test_jaajajajaja(){
//     let empty = OpenstackInfoMap::default();
//     let mut opc = OpenstackConnection::new(empty);
//     opc.token_expiry = Some("2020-05-05T00:00:00Z".to_string());
//     opc.token = Some("abcdefghijklmnop".to_string());
//     println!("===================");

//     println!("{:?}", opc);
//     println!("===================");
//     let p = opc.request_file(reqwest::Method::PUT, "http://httpbin.org/anything", "Cargo.toml");
//     println!("{:?}", p);

//     let mut response = p.unwrap().send().unwrap();

//     println!("{}", serde_json::to_string_pretty(response.json::<serde_json::Value>().unwrap()));

//     assert!(false);
// }

// curl -g -i -X PUT https://object.api.ams.fuga.cloud:443/swift/v1/5af86bc2f74c49178f32f6f479e878cc/okej/kaas.sh -H "User-Agent: openstacksdk/0.18.1 keystoneauth1/3.11.0 python-requests/2.20.0 CPython/3.6.7" -H "X-Auth-Token: {SHA256}e326fd3d280fbfd507ad5b2347f086f6b8def459f0e61f12af0f6be02fcb59e4" -d '<_io.BufferedReader name='kaas.sh'>'

// curl -X PUT -T kaas.sh -H "X-Auth-Token: gAAAAABcoKJCMxkXr8W3BmW0lIPFr_qOyrTuVXztGR40wh2J8Y_6K8LH7rGdrQeFFbgi78s7FwnzXmKDstSTQHcRWbbLP7RX8iMrv23jt713YYZ9GTE4qSf3i4tQ4jI78GlbofqyVvWGIpEFeXA51KT1Wl8HHotjo2ambcGaWjIdBlOokS_B9cI" https://object.api.ams.fuga.cloud:443/swift/v1/5af86bc2f74c49178f32f6f479e878cc/okej/kaas.sh

// curl
// {
//   "args": {},
//   "data": "na fijn",
//   "files": {},
//   "form": {},
//   "headers": {
//     "Accept": "*/*",
//     "Content-Length": "7",
//     "Host": "httpbin.org",
//     "User-Agent": "curl/7.55.1",
//     "X-Auth-Token": "gAAAAABcoKJCMxkXr8W3BmW0lIPFr_qOyrTuVXztGR40wh2J8Y_6K8LH7rGdrQeFFbgi78s7FwnzXmKDstSTQHcRWbbLP7RX8iMrv23jt713YYZ9GTE4qSf3i4tQ4jI78GlbofqyVvWGIpEFeXA51KT1Wl8HHotjo2ambcGaWjIdBlOokS_B9cI"
//   },
//   "json": null,
//   "method": "PUT",
//   "origin": "80.115.188.213, 80.115.188.213",
//   "url": "https://httpbin.org/anything"
// }

// reqwest
// {
//   "args": {},
//   "data": "na fijn",
//   "files": {},
//   "form": {},
//   "headers": {
//     "Accept": "*/*",
//     "Accept-Encoding": "gzip",
//     "Content-Length": "7",
//     "Host": "httpbin.org",
//     "User-Agent": "reqwest/0.9.5",
//     "X-Auth-Token": "gAAAAABcoKTHirMYC5LXm0-dOU0yrpu-rTo5sN6WWdXKvTY6wxFgRi1n3dQXXWtImjdtGwz7EyEDba4ZjotGTTTt68VxdiWcxATnziwig5KrVuC7DxAytbJktYbfKBiceR61lH4ElZlJX34oKT9hipsY_6fkO1dmrkLHulBbTuAxGmYOqmXG8zc"
//   },
//   "json": null,
//   "method": "PUT",
//   "origin": "80.115.188.213, 80.115.188.213",
//   "url": "https://httpbin.org/anything"
// }

// python requests
// {
//   "args": {},
//   "data": "na fijn",
//   "files": {},
//   "form": {},
//   "headers": {
//     "Accept": "*/*",
//     "Accept-Encoding": "gzip, deflate",
//     "Content-Length": "108",
//     "Host": "httpbin.org",
//     "User-Agent": "python-requests/2.20.0",
//     "X-Auth-Token": "gAAAAABcoKJCMxkXr8W3BmW0lIPFr_qOyrTuVXztGR40wh2J8Y_6K8LH7rGdrQeFFbgi78s7FwnzXmKDstSTQHcRWbbLP7RX8iMrv23jt713YYZ9GTE4qSf3i4tQ4jI78GlbofqyVvWGIpEFeXA51KT1Wl8HHotjo2ambcGaWjIdBlOokS_B9cI"
//   },
//   "json": null,
//   "method": "PUT",
//   "origin": "80.115.188.213, 80.115.188.213",
//   "url": "https://httpbin.org/anything"
// }
