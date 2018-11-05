
use utils::convert_to_singular;
use std;

#[derive(Debug, Clone)]
pub enum OSOperation{
    List,
    Show,
    New,
    Delete,
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
    Subnets,
    Ports,
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
            "subnet" => Ok(OSResource::Subnets),
            "port" => Ok(OSResource::Ports),
            _ => Err(()),
        }
    }
}
