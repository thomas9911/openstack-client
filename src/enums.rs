
use utils::convert_to_singular;
use std;

#[derive(Debug, Clone)]
pub enum OSOperation{
    List,
    Show,
    New,
    Delete,
    Update,
    Add,
    None,
}

impl std::str::FromStr for OSOperation{
    type Err = ();

    fn from_str(s: &str) -> Result<OSOperation, ()> {
        match s.to_lowercase().as_str() {
            "show" | "get" => Ok(OSOperation::Show),
            "list" | "ls" => Ok(OSOperation::List),
            "new" | "create" => Ok(OSOperation::New),
            "delete" | "remove" | "rm" => Ok(OSOperation::Delete),
            "patch" | "update" => Ok(OSOperation::Update),
            "add" | "put" | "append" => Ok(OSOperation::Add),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OSResourceType{
    Compute,
    Identity,
    Networking,
    Volume,
    Images,
    None,
}

impl std::str::FromStr for OSResourceType{
    type Err = ();

    fn from_str(s: &str) -> Result<OSResourceType, ()> {
        match s.to_lowercase().as_str() {
            "compute" => Ok(OSResourceType::Compute),
            "volume" => Ok(OSResourceType::Volume),
            "volumev2" => Ok(OSResourceType::Volume),
            "volumev3" => Ok(OSResourceType::Volume),
            "identity" => Ok(OSResourceType::Identity),
            "network" => Ok(OSResourceType::Networking),
            "image" => Ok(OSResourceType::Images),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
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
    None,
}

impl std::str::FromStr for OSResource{
    type Err = ();

    fn from_str(s: &str) -> Result<OSResource, ()> {
        match convert_to_singular(s).as_str() {
            "flavor" | "size" => Ok(OSResource::Flavors),
            "floating_ip" | "fip" => Ok(OSResource::FloatingIps),
            "image" | "operating_system" => Ok(OSResource::Images),
            "keypair" | "key" => Ok(OSResource::Keypairs),
            "network" => Ok(OSResource::Networks),
            "server" => Ok(OSResource::Servers),
            "server_group" => Ok(OSResource::ServerGroups),
            "subnet" => Ok(OSResource::Subnets),
            "port" => Ok(OSResource::Ports),
            "limit" => Ok(OSResource::Limits),
            "project" => Ok(OSResource::Projects),
            "domain" => Ok(OSResource::Domains),
            "group" => Ok(OSResource::Groups),
            "credential" => Ok(OSResource::Credentials),
            "user" => Ok(OSResource::Users),
            "address_scope" => Ok(OSResource::AddressScopes),
            "router" => Ok(OSResource::Routers),
            "security_group" => Ok(OSResource::SecurityGroups),
            "security_group_rule" => Ok(OSResource::SecurityGroupsRules),
            "availability_zone" => Ok(OSResource::AvailabilityZones),
            "volume" => Ok(OSResource::Volumes),
            "snapshot" => Ok(OSResource::Snapshots),
            "attachments" => Ok(OSResource::Attachments),
            "backups" => Ok(OSResource::Backups),
            _ => Err(()),
        }
    }
}

// impl std::fmt::Display for OSResource {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         let printable = match *self {
//             OSResource::Flavors => 'x',
//             OSResource::FloatingIps => ' ',
//         };
//         write!(f, "{}", printable)
//     }
// }

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
            OSResource::Images => "images".to_string(),
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
}