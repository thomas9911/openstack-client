
use utils::{convert_to_singular, convert_to_multiple};
use std;
use std::fmt;
use strum::IntoEnumIterator;
use reqwest::{Method};

use strum::AsStaticRef;
use structopt::StructOpt;

use std::string::ToString;
use std::io::{stdout, stderr, Error, ErrorKind};


#[derive(Debug, Clone, EnumIter, StructOpt)]
pub enum OSOperation{
    List,
    Show,
    New,
    Delete,
    Update,
    Add,
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

// #[derive(Debug, Copy, Clone, EnumIter)]
// pub enum OSResource{
//     Flavors,
//     FloatingIps,
//     Images,
//     Keypairs,
//     Networks,
//     Servers,
//     ServerGroups,
//     Subnets,
//     Ports,
//     Limits,
//     Projects,
//     Domains,
//     Groups,
//     Credentials,
//     Users,
//     AddressScopes,
//     Routers,
//     SecurityGroups,
//     SecurityGroupsRules,
//     AvailabilityZones,
//     Volumes,
//     Snapshots,
//     Attachments,
//     Backups,
//     None,
// }

#[derive(Debug, Copy, Clone, EnumIter, Serialize, Deserialize, PartialEq, Eq, StructOpt)]
#[serde(rename_all = "snake_case")]
pub enum OSResource{
    Flavors,
    FloatingIps,
    Images,
    Keypairs,
    Networks,
    Servers,
    ServerGroups,
    Subnets,
    Ports,
    Limits,
    Projects,
    Domains,
    Groups,
    Credentials,
    Users,
    AddressScopes,
    Routers,
    SecurityGroups,
    SecurityGroupsRules,
    AvailabilityZones,
    Volumes,
    Snapshots,
    Attachments,
    Backups,
    #[serde(rename="")]
    None,
}

// -
// // impl std::str::FromStr for OSResource{
// //     type Err = ();

// //     fn from_str(s: &str) -> Result<OSResource, ()> {
// //         match convert_to_singular(s).as_str() {
// //             "flavor" | "size" => Ok(OSResource::Flavors),
// //             "floating_ip" | "fip" => Ok(OSResource::FloatingIps),
// //             "image" | "operating_system" => Ok(OSResource::Images),
// //             "keypair" | "key" => Ok(OSResource::Keypairs),
// //             "network" => Ok(OSResource::Networks),
// //             "server" => Ok(OSResource::Servers),
// //             "server_group" => Ok(OSResource::ServerGroups),
// //             "subnet" => Ok(OSResource::Subnets),
// //             "port" => Ok(OSResource::Ports),
// //             "limit" => Ok(OSResource::Limits),
// //             "project" => Ok(OSResource::Projects),
// //             "domain" => Ok(OSResource::Domains),
// //             "group" => Ok(OSResource::Groups),
// //             "credential" => Ok(OSResource::Credentials),
// //             "user" => Ok(OSResource::Users),
// //             "address_scope" => Ok(OSResource::AddressScopes),
// //             "router" => Ok(OSResource::Routers),
// //             "security_group" => Ok(OSResource::SecurityGroups),
// //             "security_group_rule" => Ok(OSResource::SecurityGroupsRules),
// //             "availability_zone" => Ok(OSResource::AvailabilityZones),
// //             "volume" => Ok(OSResource::Volumes),
// //             "snapshot" => Ok(OSResource::Snapshots),
// //             "attachments" => Ok(OSResource::Attachments),
// //             "backups" => Ok(OSResource::Backups),
// //             _ => Err(()),
// //         }
// //     }
// // }

// impl std::str::FromStr for OSResource{
//     type Err = ();

//     fn from_str(s: &str) -> Result<OSResource, ()> {
//         let rs = OSResource::from(s);
//         match rs {
//             OSResource::None => Err(()),
//             _ => Ok(rs)
//         }
//     }
// }

// impl<'a> From<&'a str> for OSResource{
//     fn from(s: &str) -> OSResource {
//         match convert_to_singular(s).as_str() {
//             "flavor" | "size" => OSResource::Flavors,
//             "floating_ip" | "fip" => OSResource::FloatingIps,
//             "image" | "operating_system" => OSResource::Images,
//             "keypair" | "key" => OSResource::Keypairs,
//             "network" => OSResource::Networks,
//             "server" => OSResource::Servers,
//             "server_group" => OSResource::ServerGroups,
//             "subnet" => OSResource::Subnets,
//             "port" => OSResource::Ports,
//             "limit" => OSResource::Limits,
//             "project" => OSResource::Projects,
//             "domain" => OSResource::Domains,
//             "group" => OSResource::Groups,
//             "credential" => OSResource::Credentials,
//             "user" => OSResource::Users,
//             "address_scope" => OSResource::AddressScopes,
//             "router" => OSResource::Routers,
//             "security_group" => OSResource::SecurityGroups,
//             "security_group_rule" => OSResource::SecurityGroupsRules,
//             "availability_zone" => OSResource::AvailabilityZones,
//             "volume" => OSResource::Volumes,
//             "snapshot" => OSResource::Snapshots,
//             "attachments" => OSResource::Attachments,
//             "backups" => OSResource::Backups,
//             _ => OSResource::None,
//         }
//     }
// }

// impl From<OSResource> for String{
//     fn from(s: OSResource) -> String {
//         match s {
//             OSResource::Flavors => "flavors".into(),
//             OSResource::FloatingIps => "floating_ip".into(),
//             OSResource::Images => "images".into(),
//             OSResource::Keypairs => "keypairs".into(),
//             OSResource::Networks => "networks".into(),
//             OSResource::Servers => "servers".into(),
//             OSResource::ServerGroups => "server_groups".into(),
//             OSResource::Subnets => "subnets".into(),
//             OSResource::Ports => "ports".into(),
//             OSResource::Limits => "limits".into(),
//             OSResource::Projects => "projects".into(),
//             OSResource::Domains => "domains".into(),
//             OSResource::Groups => "groups".into(),
//             OSResource::Credentials => "credentials".into(),
//             OSResource::Users => "users".into(),
//             OSResource::AddressScopes => "address_scopes".into(),
//             OSResource::Routers => "routers".into(),
//             OSResource::SecurityGroups => "security_groups".into(),
//             OSResource::SecurityGroupsRules => "security_groups_rules".into(),
//             OSResource::AvailabilityZones => "availability_zones".into(),
//             OSResource::Volumes => "volumes".into(),
//             OSResource::Snapshots => "snapshots".into(),
//             OSResource::Attachments => "attachments".into(),
//             OSResource::Backups => "backups".into(),
//             OSResource::None => "".into(),
//         }
//     }
// }

impl fmt::Display for OSResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r: String = (*self).into();
        write!(f, "{}", r)
    }
}

impl std::str::FromStr for OSResource{
    type Err = Error;

    fn from_str(s: &str) -> Result<OSResource, Error> {
        let rs = OSResource::from(s);
        match rs {
            OSResource::None => Err(Error::new(ErrorKind::Other, "kaas")),
            _ => Ok(rs)
        }
    }
}

impl<'a> From<&'a str> for OSResource{
    fn from(s: &str) -> Self{
        Self::from(s.to_string())
    }
}

impl From<String> for OSResource{
    fn from(s: String) -> Self{
        match serde_json::from_str(&format!("{:?}", convert_to_multiple(s))){
            Ok(x) => x,
            _ => OSResource::None
        }
    }
}

impl Into<String> for OSResource{
    fn into(self) -> String{
        let tmp = serde_json::to_string(&self).unwrap();
        tmp[1..(tmp.len()-1)].to_string()
    }
}



impl OSResource{
    pub fn match_type(&self) -> OSResourceType{
        match *self{
            OSResource::Flavors => OSResourceType::Compute,
            OSResource::FloatingIps => OSResourceType::Networking,
            OSResource::Images => OSResourceType::Images,
            OSResource::Keypairs => OSResourceType::Compute,
            OSResource::Networks => OSResourceType::Networking,
            OSResource::Servers => OSResourceType::Compute,
            OSResource::ServerGroups => OSResourceType::Compute,
            OSResource::Subnets => OSResourceType::Networking,
            OSResource::Ports => OSResourceType::Networking,
            OSResource::Limits => OSResourceType::Compute,
            OSResource::Projects => OSResourceType::Identity,
            OSResource::Domains => OSResourceType::Identity,
            OSResource::Groups => OSResourceType::Identity,
            OSResource::Credentials => OSResourceType::Identity,
            OSResource::Users => OSResourceType::Identity,
            OSResource::AddressScopes => OSResourceType::Networking,
            OSResource::Routers => OSResourceType::Networking,
            OSResource::SecurityGroups => OSResourceType::Networking,
            OSResource::SecurityGroupsRules => OSResourceType::Networking,
            OSResource::AvailabilityZones => OSResourceType::Networking,
            OSResource::Volumes => OSResourceType::Volume,
            OSResource::Snapshots => OSResourceType::Volume,
            OSResource::Attachments => OSResourceType::Volume,
            OSResource::Backups => OSResourceType::Volume,
            OSResource::None => OSResourceType::None,
        }
    }
    pub fn match_endpoint(&self) -> String{
        match *self{
            OSResource::Flavors => "flavors".to_string(),
            OSResource::FloatingIps => "v2.0/floatingips".to_string(),
            OSResource::Images => "v2/images".to_string(),
            OSResource::Keypairs => "os-keypairs".to_string(),
            OSResource::Networks => "v2.0/networks".to_string(),
            OSResource::Servers => "servers".to_string(),
            OSResource::ServerGroups => "os-server-groups".to_string(),
            OSResource::Subnets => "v2.0/subnets".to_string(),
            OSResource::Ports => "v2.0/ports".to_string(),
            OSResource::Limits => "limits".to_string(),
            OSResource::Projects => "projects".to_string(),
            OSResource::Domains => "domains".to_string(),
            OSResource::Groups => "group".to_string(),
            OSResource::Credentials => "credentials".to_string(),
            OSResource::Users => "users".to_string(),
            OSResource::AddressScopes => "v2.0/address-scopes".to_string(),
            OSResource::Routers => "v2.0/routers".to_string(),
            OSResource::SecurityGroups => "v2.0/security-groups".to_string(),
            OSResource::SecurityGroupsRules => "v2.0/security-groups-rules".to_string(),
            OSResource::AvailabilityZones => "v2.0/availability_zones".to_string(),
            OSResource::Volumes => "volumes".to_string(),
            OSResource::Snapshots => "snapshots".to_string(),
            OSResource::Attachments => "attachments".to_string(),
            OSResource::Backups => "backups".to_string(),
            OSResource::None => "".to_string(),
        }
    }

    pub fn list() -> Vec<String>{
        OSResource::iter().filter(|x| x != &OSResource::None).map(|x| x.into()).collect::<Vec<String>>()
    }
}