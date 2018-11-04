from bs4 import BeautifulSoup
import re
import requests

ONLY_ONE = True

def convert(name):
    # https://stackoverflow.com/questions/1175208/elegant-python-function-to-convert-camelcase-to-snake-case
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()

def get_methods_and_stuff(url):
    r = requests.get(url)

    soup = BeautifulSoup(r.content, 'html.parser')


    # print(soup)

    # print(soup.title)
    methods = []
    for item in soup.find_all("div", class_=re.compile("sidebar-links")):
        for k in item.find_all("a"):
            if str(k.get('href')).startswith("#method"):
                methods.append(k.string)

    for item in soup.find("a", class_="struct"):
        title = convert(item.string)

    obj = ""
    for item in soup.find("span", class_="in-band"):
        if item.string and not item.string.startswith("Struct"):
            obj += item.string

    obj_var=title[0].lower()
    return methods, title, obj, obj_var


def format_middle_part(methods, templates, title, obj, obj_var, subtitle):

    middle_template =  templates["middle_template"]        
    middle_template_normal = templates["middle_template_normal"]
    middle_template_some_int = templates["middle_template_some_int"]
    middle_template_object = templates['middle_template_object']
    
    middle = ""

    for method in methods:
        if method not in ["delete", "save", "associate", "dissociate", "extra_dhcp_opts_mut", "start", "reboot", "stop"] and not method.startswith("set") and not method.startswith("with"):
            if method in ['id', 'fingerprint']:
                middle += middle_template_normal.format(method=method, obj_var=obj_var)
            elif method in ['name']:
                if title in ['subnet', 'port', 'network']:
                    middle += middle_template.format(method=method, obj_var=obj_var)
                else:
                    middle += middle_template_normal.format(method=method, obj_var=obj_var)
            elif method in ['size']:
                middle += middle_template_some_int.format(method=method, obj_var=obj_var)
            elif method in ['details']:
                middle += middle_template_object.format(method=method, obj_var=obj_var, subtitle=subtitle)
            else:
                middle += middle_template.format(method=method, obj_var=obj_var)

    return middle

def make_hashmap_functions(templates, methods, title, obj, obj_var):
    # fn_template = """fn fmt_{title}({obj_var}: {obj}) -> HashMap<String, String>{{\n"""
    # middle_template =        """\n\t\t("{method}".to_string(), format!("{{:?}}", {obj_var}.{method}())),"""
    # middle_template_normal = """\n\t\t("{method}".to_string(), format!("{{}}", {obj_var}.{method}())),"""
    # middle_template_some_int = """\n\t\t("{method}".to_string(), format!("{{}}", {obj_var}.{method}().unwrap_or(0))),"""

    fn_template = templates["fn_template"]
    # middle_template =  templates["middle_template"]        
    # middle_template_normal = templates["middle_template_normal"]
    # middle_template_some_int = templates["middle_template_some_int"]

    fn = fn_template.format(title=title, obj_var=obj_var, obj=obj)
    start = """\tlet json: HashMap<String, String> = ["""
    middle = "\t"
    end = """\n\t].iter().cloned().collect();\n\tjson\n}"""

    subtitle = title.split("_")[-1]

    middle = format_middle_part(methods, templates, title, obj, obj_var, subtitle)
    # for method in methods:
    #     if method not in ["delete", "save", "associate", "dissociate", "extra_dhcp_opts_mut"] and not method.startswith("set") and not method.startswith("with"):
    #         if method in ['id']:
    #             middle += middle_template_normal.format(method=method, obj_var=obj_var)
    #         elif method in ['name']:
    #             if title in ['subnet', 'port']:
    #                 middle += middle_template.format(method=method, obj_var=obj_var)
    #             else:
    #                 middle += middle_template_normal.format(method=method, obj_var=obj_var)
    #         elif method in ['size']:
    #             middle += middle_template_some_int.format(method=method, obj_var=obj_var)
    #         else:
    #             middle += middle_template.format(method=method, obj_var=obj_var)

    txt2 = fn + start + middle + end

    txt1 = """
pub fn print_{title}_data(data: openstack::Result<Vec<{obj}>>){{
    let mut stuff: Vec<HashMap<String, String>> = vec![];
    match data{{
        Ok(x) => {{
            for item in x{{
                stuff.push(fmt_{title}(item));
            }}
            println!("{{:#?}}", stuff);
        }},
        _ => ()
    }}
}}
    """.format(title=title, obj=obj)

    print(txt1)
    print()
    print(txt2)

def make_flavors_code(methods, title, obj, obj_var):
    global ONLY_ONE
    if ONLY_ONE is True:
        ONLY_ONE = False
        return ""
    # template_method = """\t\t\t\t\t("{method}".to_string(), Value::from({obj_var}.{method}().clone())),\n"""
    template_method = """\t\t\t\t\t("{method}".to_string(), to_value({obj_var}.{method}()).unwrap()),\n"""

    first = """
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
		Ok(f) =>  [	"""
    

    second = """				].iter().cloned().collect(),
		_ => [].iter().cloned().collect()
	};
	Value::from(hashmap)
}
    """

    end = ""
    end += first
    for method in methods:
        end += template_method.format(method=method, title=title, obj=obj, obj_var=obj_var)
    end += second
    return end

def make_values_functions(templates, methods, title, obj, obj_var):
    if title not in ["flavor", "flavor_summary"]:
        fn_template = templates["fn_template"]

        fn = fn_template.format(title=title, obj_var=obj_var, obj=obj)
        start = """\tlet hashmap: Map<String, Value> = ["""
        middle = "\t"
        end = """\n\t].iter().cloned().collect();\n\tValue::from(hashmap)\n}"""

        subtitle = title.split("_")[0]

        middle = format_middle_part(methods, templates, title, obj, obj_var, subtitle)

        txt2 = fn + start + middle + end

        txt1 = """
pub fn print_{title}_data(data: openstack::Result<Vec<{obj}>>){{
    let mut jsons: Vec<Value> = vec![];
    match data{{
        Ok(x) => {{
            for item in x{{
                jsons.push(fmt_{title}(item));
            }}
            println!("{{}}", serde_json::to_string_pretty(&jsons).unwrap());
        }},
        _ => ()
    }}
}}
        """.format(title=title, obj=obj)
        print(txt1)
        print(txt2)
    else:
        print(make_flavors_code(methods, title, obj, obj_var))

urls = ["https://dtantsur.github.io/rust-openstack/openstack/compute/struct.FlavorSummary.html",
        "https://dtantsur.github.io/rust-openstack/openstack/compute/struct.Flavor.html", 
        "https://dtantsur.github.io/rust-openstack/openstack/network/struct.FloatingIp.html",
        "https://dtantsur.github.io/rust-openstack/openstack/image/struct.Image.html",
        "https://dtantsur.github.io/rust-openstack/openstack/compute/struct.KeyPair.html",
        "https://dtantsur.github.io/rust-openstack/openstack/network/struct.Network.html",
        "https://dtantsur.github.io/rust-openstack/openstack/network/struct.Port.html",
        "https://dtantsur.github.io/rust-openstack/openstack/compute/struct.ServerSummary.html",
        "https://dtantsur.github.io/rust-openstack/openstack/compute/struct.Server.html",
        "https://dtantsur.github.io/rust-openstack/openstack/network/struct.Subnet.html"]


templates_hashmap = {
    "fn_template": """fn fmt_{title}({obj_var}: {obj}) -> HashMap<String, String>{{\n""",
    "middle_template": """\n\t\t("{method}".to_string(), format!("{{:?}}", {obj_var}.{method}())),""",
    "middle_template_normal": """\n\t\t("{method}".to_string(), format!("{{}}", {obj_var}.{method}())),""",
    "middle_template_some_int": """\n\t\t("{method}".to_string(), format!("{{}}", {obj_var}.{method}().unwrap_or(0))),""",
}

templates_value = {
    "fn_template": """pub fn fmt_{title}({obj_var}: {obj}) -> Value{{\n""",
    "middle_template": """\n\t\t("{method}".to_string(), Value::from(format!("{{:?}}", {obj_var}.{method}()))),""",
    "middle_template_normal": """\n\t\t("{method}".to_string(), Value::from({obj_var}.{method}().clone())),""",
    "middle_template_some_int": """\n\t\t("{method}".to_string(), Value::from({obj_var}.{method}().unwrap_or(0))),""",
    "middle_template_object": """\n\t\t("{method}".to_string(), fmt_{subtitle}({obj_var}.{method}().unwrap())),"""
}

imports = """
extern crate openstack;
extern crate serde_json;

use serde_json::{Value, Map};
use std::collections::HashMap;

use serde_json::to_value;
"""

print(imports)
print("// Generated by 'get-methods-from-rustdoc.py' ")
for url in urls:
    methods, title, obj, obj_var = get_methods_and_stuff(url)
    # make_hashmap_functions(templates_hashmap, methods, title, obj, obj_var)
    make_values_functions(templates_value, methods, title, obj, obj_var)
