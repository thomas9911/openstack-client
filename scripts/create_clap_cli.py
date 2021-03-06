import oyaml as yaml
import json
from collections import OrderedDict as od
from pprint import pprint
from copy import deepcopy

from random import sample

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

MY_NONE = sample("abcdefghijklmnopqrstuvwxyz"*150, 50)

def snake_to_kebabcase(val):
    t = val.split("_")
    if len(t)>1:
        return "-".join(t)
    return val


def kebab_to_snakecase(val):
    t = val.split("-")
    if len(t)>1:
        return "_".join(t)
    return val


def make_singular(text):
    if text[-1] == 's' or text[-1] == 's':
        return text[:-1]
    else:
        return text


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

with open("data/actions.yaml") as f:
    actions = yaml.load(f)

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
        ("args", []),
        ('visible_aliases', [make_singular(new_k)])
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
            tmp['placement'] = l.get("placement", "body")
            d = l.get('default')
            if d is not None:
                tmp['default_value'] = d

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
    if command == 'call':
        del stuff['subcommands']
        stuff['args'].extend([{
            "method": {
                "help": "http method to use",
                "takes_value": True,
                "possible_values": ["POST", "GET", "PATCH", "DELETE", "PUT", "OPTIONS", "HEAD", "CONNECT", "TRACE"],
                "required": True
            }
        }, {
            "type": {
                "help": "the openstack type to use, such as 'compute' or 'image'",
                "takes_value": True,
                "required": True
            }
        }, {
            "endpoint": {
                "help": "endpoint or path to send the call to",
                 "takes_value": True,
                 "required": True
            }
        }, {
            "body": {
                "help": 'a json object as a string, for example "{\\"test\\": \\"test\\"}"',
                "takes_value": True,
                "value_name": "BODY",
            }
        }, {
            "header": {
                "help": 'set additional headers, for example "X-Testing: test"',
                "takes_value": True,
                "long": "header",
                "short": "H",
                "multiple": True,
            },
        }])

    clap_app["subcommands"].append({command: stuff})

    if not data['has_body']:
        for i, item in enumerate(clap_app["subcommands"]):
            if item.get(command):
                for j, item in enumerate(clap_app["subcommands"][i][command]['subcommands']):
                    for resource in clap_app["subcommands"][i][command]['subcommands'][j]:
                        for k, arg in enumerate(clap_app["subcommands"][i][command]['subcommands'][j][resource]['args']):
                            to_delete = []
                            for name in arg:
                                try:
                                    if clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'][k][name]["placement"] == 'body':
                                        to_delete.append((k, name))
                                except Exception:
                                    pass
                            for delete in to_delete:
                                # pass
                                del clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'][delete[0]][delete[1]]

                        # clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'] = []
    for i, item in enumerate(clap_app["subcommands"]):
        try:
            for j, item in enumerate(clap_app["subcommands"][i][command]['subcommands']):
                for resource in clap_app["subcommands"][i][command]['subcommands'][j]:
                    clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'] = [x for x in clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'] if x]
                    for k, arg in enumerate(clap_app["subcommands"][i][command]['subcommands'][j][resource]['args']):
                        for name in arg:
                            del clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'][k][name]['placement']
        except KeyError:
            pass
    if data['requires_id']:
        for i, item in enumerate(clap_app["subcommands"]):
            if item.get(command):
                try:
                    for j, item in enumerate(clap_app["subcommands"][i][command]['subcommands']):
                        for resource in clap_app["subcommands"][i][command]['subcommands'][j]:
                            clap_app["subcommands"][i][command]['subcommands'][j][resource]['args'].append({"id": id_blub})
                except KeyError:
                    clap_app["subcommands"][i][command]['args'].append({"id": id_blub})

for action, data in actions.items():

    # print(action)
    # print(data)
    new_stuff = od([
        ("about", data['help']),
        ("visible_aliases", data['aliases']),
        ("case_insensitive", True),
        ("index", 1),
        ('subcommands', []),
        # ("args", [{"resource": resources_blub}]),
        ("args", [{"dry-run": {
            "long": "dry-run",
            "help": "prints the post body of the request, does not send the request"
        }}]),
    ])
    for rs_data in data['resources']:
        resource = rs_data['resource']
        new_rs = snake_to_kebabcase(resource)
        val = od([(new_rs, od([
            ("index", 1),
            ("case_insensitive", True),
            ("about", rs_data['help']),
            ("args", []),
            # ('subcommands', []),
            ('visible_aliases', [make_singular(new_rs)])
        ]))])
        if rs_data['requires_id']:
            val[new_rs]['args'].append({"id": id_blub})

        for l in rs_data.get('post_parameters', []):
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
                # tmp['placement'] = l.get("placement", "body")
                d = l.get('default')
                if d is not None:
                    tmp['default_value'] = d

                val[new_rs]["args"].append({l['name']: tmp.copy()})
            # tmp = od({})
            # if param.get('help'):
            #     tmp['help'] = param['help']
            # if param.get('required', False):
            #     tmp['required'] = True
            #     # print(action, tmp)
            # tmp['long'] = param['name']
            # tmp['multiple'] = False

            # tmp['takes_value'] = True
            # d = param.get('default', None)
            # if d is not None:
            #     tmp['default_value'] = d
            # val[new_rs]['args'].append({param['name']: tmp})
        new_stuff['subcommands'].append(val)

        # print(action, resource, val)

    clap_app["subcommands"].append({action: new_stuff})

print(yaml.dump(clap_app, default_flow_style=False))
# print(json.dumps(list(clap_app["subcommands"]), indent=2))

