import oyaml as yaml
import json
from collections import OrderedDict as od
from pprint import pprint
from copy import deepcopy
# def parse_rust_enum_notation(t):
#     z = [x.split("=>")[0].split("|") for x in t.splitlines() if x.strip()]
#     pos_vals = [eval(x.strip()) for y in z for x in y]
#     return pos_vals

# t = """
#             "show" | "get" => OSOperation::Show,
#             "list" | "ls" => OSOperation::List,
#             "new" | "create" => OSOperation::New,
#             "delete" | "remove" | "rm" => OSOperation::Delete,
#             "patch" | "update" => OSOperation::Update,
#             "add" | "put" | "append"
# """

def snake_to_kebabcase(val):
    t = val.split("_")
    if len(t)>1:
        return "-".join(t)
    return val

clap_app = od([
    ("name", "openstack-client"),
    ("settings", ['ArgRequiredElseHelp']),
    ("args", [
        od([
            ("os-cloud", {
            "long": "os-cloud",
            "help": "use this as the cloud name from the clouds.yaml",
            "takes_value": True,
            })
        ])
        ]
    ),
    ("subcommands", [])])

with open("data/commands.yaml") as f:
    commands = yaml.load(f)

with open("data/resources.yaml") as f:
    resources = yaml.load(f)


# print(commands)
# pos_vals = parse_rust_enum_notation(t)
# clap_app["args"].append(od({"help": "", "index": 1, "possible_values": pos_vals}))

# possible_resources = [snake_to_kebabcase(x) for x in resources.keys()]
# resources_blub = od([
#     ("help", "resource to use"),
#     ("possible_values", possible_resources),
#     ("required", True),
#     ("index", 1),
#     # ("case_insensitive", True),
#     ])
sub_args = []
for k, v in resources.items():
    new_k = snake_to_kebabcase(k)
    val = od([(new_k, od([
        ("index", 1),
        ("case_insensitive", True),
        ("about", "resource A"),
        ("args", [])
    ]))])
    for l in v.get("post_parameters", []):
        if not l.get('hidden', False):
            tmp = od({})
            if l.get('help'):
                tmp['help'] = l['help']
            if l.get('required', False):
                tmp['required'] = True
            # if not l.get('required', False):
            tmp['long'] = l['name']

            tmp['takes_value'] = True
            tmp['multiple'] = l.get('multiple', False)

            val[new_k]["args"].append({l['name']: tmp.copy()})

    sub_args.append(val)

id_blub = od([
    ("help", "id of object that will be used"),
    ("required", True),
    ("index", 1),
    ])

for command, data in commands.items():
    stuff = od([
        ("about", data['help']),
        ("visible_aliases", data['aliases']),
        ("case_insensitive", True),
        ("index", 1),
        # ("args", [{"resource": resources_blub}]),
        ("args", [{"dry-run": {
            "long": "dry-run",
            "help": "prints the post body of the request, does not send the request"
        }}]),
        ("subcommands", deepcopy(sub_args))
    ])
    clap_app["subcommands"].append({command: stuff})

    if not data['has_body']:
        for i, item in enumerate(clap_app["subcommands"]):
            if item.get(command):
                for j, item in enumerate(clap_app["subcommands"][i][command]['subcommands']):
                    for resource in clap_app["subcommands"][i][command]['subcommands'][j]:
                        clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'] = []
    if data['requires_id']:
        for i, item in enumerate(clap_app["subcommands"]):
            if item.get(command):
                for j, item in enumerate(clap_app["subcommands"][i][command]['subcommands']):
                    for resource in clap_app["subcommands"][i][command]['subcommands'][j]:
                        clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'].append({"id": id_blub})

print(yaml.dump(clap_app, default_flow_style=False))
# print(json.dumps(clap_app, indent=2))
