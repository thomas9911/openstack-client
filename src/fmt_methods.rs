
extern crate openstack;
extern crate serde_json;

use serde_json::{Value, Map};
use std::collections::HashMap;

use serde_json::to_value;



pub fn print_flavor_summary_data(data: openstack::Result<Vec<openstack::compute::FlavorSummary>>){
    let mut jsons: Vec<HashMap<String, Value>> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_flavor_summary(item));
            }
			println!("{}", serde_json::to_string_pretty(&jsons).unwrap())
        },
        _ => ()
    }
}
    
pub fn fmt_flavor_summary(f: openstack::compute::FlavorSummary) -> HashMap<String, Value>{
	let json: HashMap<String, Value> = [	
		("id".to_string(), Value::from(format!("{}", f.id()))),
		("name".to_string(), Value::from(format!("{}", f.name()))),
		("details".to_string(), fmt_flavor(f.details())),
	].iter().cloned().collect();
	json
}

pub fn fmt_flavor(data: openstack::Result<openstack::compute::Flavor>) -> Value{
	let hashmap: Map<String, Value> = match data{
		Ok(f) =>  [						("emphemeral_size".to_string(), to_value(f.emphemeral_size()).unwrap()),
					("extra_specs".to_string(), to_value(f.extra_specs()).unwrap()),
					("id".to_string(), to_value(f.id()).unwrap()),
					("is_public".to_string(), to_value(f.is_public()).unwrap()),
					("name".to_string(), to_value(f.name()).unwrap()),
					("ram_size".to_string(), to_value(f.ram_size()).unwrap()),
					("root_size".to_string(), to_value(f.root_size()).unwrap()),
					("swap_size".to_string(), to_value(f.swap_size()).unwrap()),
					("vcpu_count".to_string(), to_value(f.vcpu_count()).unwrap()),
				].iter().cloned().collect(),
		_ => [].iter().cloned().collect()
	};
	Value::from(hashmap)
}
    

pub fn print_floating_ip_data(data: openstack::Result<Vec<openstack::network::FloatingIp>>){
    let mut jsons: Vec<Value> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_floating_ip(item));
            }
            println!("{}", serde_json::to_string_pretty(&jsons).unwrap());
        },
        _ => ()
    }
}
        

pub fn fmt_floating_ip(f: openstack::network::FloatingIp) -> Value{
	let hashmap: Map<String, Value> = [
		("created_at".to_string(), Value::from(format!("{:?}", f.created_at()))),
		("description".to_string(), Value::from(format!("{:?}", f.description()))),
		("dns_domain".to_string(), Value::from(format!("{:?}", f.dns_domain()))),
		("dns_name".to_string(), Value::from(format!("{:?}", f.dns_name()))),
		("fixed_ip_address".to_string(), Value::from(format!("{:?}", f.fixed_ip_address()))),
		("floating_ip_address".to_string(), Value::from(format!("{:?}", f.floating_ip_address()))),
		("floating_network_id".to_string(), Value::from(format!("{:?}", f.floating_network_id()))),
		("floating_network".to_string(), Value::from(format!("{:?}", f.floating_network()))),
		("id".to_string(), Value::from(f.id().clone())),
		("is_associated".to_string(), Value::from(format!("{:?}", f.is_associated()))),
		("port_forwardings".to_string(), Value::from(format!("{:?}", f.port_forwardings()))),
		("port_id".to_string(), Value::from(format!("{:?}", f.port_id()))),
		("router_id".to_string(), Value::from(format!("{:?}", f.router_id()))),
		("port".to_string(), Value::from(format!("{:?}", f.port()))),
		("status".to_string(), Value::from(format!("{:?}", f.status()))),
		("updated_at".to_string(), Value::from(format!("{:?}", f.updated_at()))),
	].iter().cloned().collect();
	Value::from(hashmap)
}

pub fn print_image_data(data: openstack::Result<Vec<openstack::image::Image>>){
    let mut jsons: Vec<Value> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_image(item));
            }
            println!("{}", serde_json::to_string_pretty(&jsons).unwrap());
        },
        _ => ()
    }
}
        

pub fn fmt_image(i: openstack::image::Image) -> Value{
	let hashmap: Map<String, Value> = [
		("architecture".to_string(), Value::from(format!("{:?}", i.architecture()))),
		("checksum".to_string(), Value::from(format!("{:?}", i.checksum()))),
		("container_format".to_string(), Value::from(format!("{:?}", i.container_format()))),
		("created_at".to_string(), Value::from(format!("{:?}", i.created_at()))),
		("disk_format".to_string(), Value::from(format!("{:?}", i.disk_format()))),
		("id".to_string(), Value::from(i.id().clone())),
		("minimum_required_disk".to_string(), Value::from(format!("{:?}", i.minimum_required_disk()))),
		("minimum_required_ram".to_string(), Value::from(format!("{:?}", i.minimum_required_ram()))),
		("name".to_string(), Value::from(i.name().clone())),
		("size".to_string(), Value::from(i.size().unwrap_or(0))),
		("status".to_string(), Value::from(format!("{:?}", i.status()))),
		("updated_at".to_string(), Value::from(format!("{:?}", i.updated_at()))),
		("virtual_size".to_string(), Value::from(format!("{:?}", i.virtual_size()))),
		("visibility".to_string(), Value::from(format!("{:?}", i.visibility()))),
	].iter().cloned().collect();
	Value::from(hashmap)
}

pub fn print_key_pair_data(data: openstack::Result<Vec<openstack::compute::KeyPair>>){
    let mut jsons: Vec<Value> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_key_pair(item));
            }
            println!("{}", serde_json::to_string_pretty(&jsons).unwrap());
        },
        _ => ()
    }
}
        

pub fn fmt_key_pair(k: openstack::compute::KeyPair) -> Value{
	let hashmap: Map<String, Value> = [
		("fingerprint".to_string(), Value::from(k.fingerprint().clone())),
		("key_type".to_string(), Value::from(format!("{:?}", k.key_type()))),
		("name".to_string(), Value::from(k.name().clone())),
	].iter().cloned().collect();
	Value::from(hashmap)
}

pub fn print_network_data(data: openstack::Result<Vec<openstack::network::Network>>){
    let mut jsons: Vec<Value> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_network(item));
            }
            println!("{}", serde_json::to_string_pretty(&jsons).unwrap());
        },
        _ => ()
    }
}
        

pub fn fmt_network(n: openstack::network::Network) -> Value{
	let hashmap: Map<String, Value> = [
		("admin_state_up".to_string(), Value::from(format!("{:?}", n.admin_state_up()))),
		("availability_zones".to_string(), Value::from(format!("{:?}", n.availability_zones()))),
		("created_at".to_string(), Value::from(format!("{:?}", n.created_at()))),
		("description".to_string(), Value::from(format!("{:?}", n.description()))),
		("dns_domain".to_string(), Value::from(format!("{:?}", n.dns_domain()))),
		("external".to_string(), Value::from(format!("{:?}", n.external()))),
		("id".to_string(), Value::from(n.id().clone())),
		("is_default".to_string(), Value::from(format!("{:?}", n.is_default()))),
		("l2_adjacency".to_string(), Value::from(format!("{:?}", n.l2_adjacency()))),
		("mtu".to_string(), Value::from(format!("{:?}", n.mtu()))),
		("name".to_string(), Value::from(format!("{:?}", n.name()))),
		("port_security_enabled".to_string(), Value::from(format!("{:?}", n.port_security_enabled()))),
		("shared".to_string(), Value::from(format!("{:?}", n.shared()))),
		("updated_at".to_string(), Value::from(format!("{:?}", n.updated_at()))),
		("vlan_transparent".to_string(), Value::from(format!("{:?}", n.vlan_transparent()))),
	].iter().cloned().collect();
	Value::from(hashmap)
}

pub fn print_port_data(data: openstack::Result<Vec<openstack::network::Port>>){
    let mut jsons: Vec<Value> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_port(item));
            }
            println!("{}", serde_json::to_string_pretty(&jsons).unwrap());
        },
        _ => ()
    }
}
        

pub fn fmt_port(p: openstack::network::Port) -> Value{
	let hashmap: Map<String, Value> = [
		("admin_state_up".to_string(), Value::from(format!("{:?}", p.admin_state_up()))),
		("attached_to_server".to_string(), Value::from(format!("{:?}", p.attached_to_server()))),
		("created_at".to_string(), Value::from(format!("{:?}", p.created_at()))),
		("description".to_string(), Value::from(format!("{:?}", p.description()))),
		("device_id".to_string(), Value::from(format!("{:?}", p.device_id()))),
		("device_owner".to_string(), Value::from(format!("{:?}", p.device_owner()))),
		("dns_domain".to_string(), Value::from(format!("{:?}", p.dns_domain()))),
		("dns_name".to_string(), Value::from(format!("{:?}", p.dns_name()))),
		("extra_dhcp_opts".to_string(), Value::from(format!("{:?}", p.extra_dhcp_opts()))),
		("fixed_ips".to_string(), Value::from(format!("{:?}", p.fixed_ips()))),
		("mac_address".to_string(), Value::from(format!("{:?}", p.mac_address()))),
		("id".to_string(), Value::from(p.id().clone())),
		("name".to_string(), Value::from(format!("{:?}", p.name()))),
		("network".to_string(), Value::from(format!("{:?}", p.network()))),
		("network_id".to_string(), Value::from(format!("{:?}", p.network_id()))),
		("status".to_string(), Value::from(format!("{:?}", p.status()))),
		("updated_at".to_string(), Value::from(format!("{:?}", p.updated_at()))),
		("is_dirty".to_string(), Value::from(format!("{:?}", p.is_dirty()))),
	].iter().cloned().collect();
	Value::from(hashmap)
}

pub fn print_server_summary_data(data: openstack::Result<Vec<openstack::compute::ServerSummary>>){
    let mut jsons: Vec<Value> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_server_summary(item));
            }
            println!("{}", serde_json::to_string_pretty(&jsons).unwrap());
        },
        _ => ()
    }
}
        

pub fn fmt_server_summary(s: openstack::compute::ServerSummary) -> Value{
	let hashmap: Map<String, Value> = [
		("id".to_string(), Value::from(s.id().clone())),
		("name".to_string(), Value::from(s.name().clone())),
		("details".to_string(), fmt_server(s.details().unwrap())),
	].iter().cloned().collect();
	Value::from(hashmap)
}

pub fn print_server_data(data: openstack::Result<Vec<openstack::compute::Server>>){
    let mut jsons: Vec<Value> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_server(item));
            }
            println!("{}", serde_json::to_string_pretty(&jsons).unwrap());
        },
        _ => ()
    }
}
        

pub fn fmt_server(s: openstack::compute::Server) -> Value{
	let hashmap: Map<String, Value> = [
		("access_ipv4".to_string(), Value::from(format!("{:?}", s.access_ipv4()))),
		("access_ipv6".to_string(), Value::from(format!("{:?}", s.access_ipv6()))),
		("addresses".to_string(), Value::from(format!("{:?}", s.addresses()))),
		("availability_zone".to_string(), Value::from(format!("{:?}", s.availability_zone()))),
		("created_at".to_string(), Value::from(format!("{:?}", s.created_at()))),
		("description".to_string(), Value::from(format!("{:?}", s.description()))),
		("flavor".to_string(), Value::from(format!("{:?}", s.flavor()))),
		("floating_ip".to_string(), Value::from(format!("{:?}", s.floating_ip()))),
		("has_config_drive".to_string(), Value::from(format!("{:?}", s.has_config_drive()))),
		("has_image".to_string(), Value::from(format!("{:?}", s.has_image()))),
		("id".to_string(), Value::from(s.id().clone())),
		("image".to_string(), Value::from(format!("{:?}", s.image()))),
		("image_id".to_string(), Value::from(format!("{:?}", s.image_id()))),
		("key_pair".to_string(), Value::from(format!("{:?}", s.key_pair()))),
		("key_pair_name".to_string(), Value::from(format!("{:?}", s.key_pair_name()))),
		("name".to_string(), Value::from(s.name().clone())),
		("metadata".to_string(), Value::from(format!("{:?}", s.metadata()))),
		("power_state".to_string(), Value::from(format!("{:?}", s.power_state()))),
		("status".to_string(), Value::from(format!("{:?}", s.status()))),
		("updated_at".to_string(), Value::from(format!("{:?}", s.updated_at()))),
	].iter().cloned().collect();
	Value::from(hashmap)
}

pub fn print_subnet_data(data: openstack::Result<Vec<openstack::network::Subnet>>){
    let mut jsons: Vec<Value> = vec![];
    match data{
        Ok(x) => {
            for item in x{
                jsons.push(fmt_subnet(item));
            }
            println!("{}", serde_json::to_string_pretty(&jsons).unwrap());
        },
        _ => ()
    }
}
        

pub fn fmt_subnet(s: openstack::network::Subnet) -> Value{
	let hashmap: Map<String, Value> = [
		("allocation_pools".to_string(), Value::from(format!("{:?}", s.allocation_pools()))),
		("cidr".to_string(), Value::from(format!("{:?}", s.cidr()))),
		("created_at".to_string(), Value::from(format!("{:?}", s.created_at()))),
		("description".to_string(), Value::from(format!("{:?}", s.description()))),
		("dhcp_enabled".to_string(), Value::from(format!("{:?}", s.dhcp_enabled()))),
		("dns_nameservers".to_string(), Value::from(format!("{:?}", s.dns_nameservers()))),
		("gateway_ip".to_string(), Value::from(format!("{:?}", s.gateway_ip()))),
		("host_routes".to_string(), Value::from(format!("{:?}", s.host_routes()))),
		("id".to_string(), Value::from(s.id().clone())),
		("ip_version".to_string(), Value::from(format!("{:?}", s.ip_version()))),
		("ipv6_address_mode".to_string(), Value::from(format!("{:?}", s.ipv6_address_mode()))),
		("ipv6_router_advertisement_mode".to_string(), Value::from(format!("{:?}", s.ipv6_router_advertisement_mode()))),
		("name".to_string(), Value::from(format!("{:?}", s.name()))),
		("network".to_string(), Value::from(format!("{:?}", s.network()))),
		("network_id".to_string(), Value::from(format!("{:?}", s.network_id()))),
		("updated_at".to_string(), Value::from(format!("{:?}", s.updated_at()))),
	].iter().cloned().collect();
	Value::from(hashmap)
}
