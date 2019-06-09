from collections import OrderedDict

import oyaml as yaml

t = yaml.load(open("data/resources.yaml"))

r = OrderedDict(sorted(t.items(), key= lambda x: (x[1]["resource_type"], x[0])))

yaml.dump(r, open("data/resources.yaml.tmp", "w"), default_flow_style=False)