import sys
import json

from collections import Mapping, OrderedDict
class Slicer(Mapping):
    def __init__(self, a_dict, make_flat=False):
        if isinstance(a_dict, Mapping):
            self.data = a_dict
        elif isinstance(a_dict, list):
            self.data = a_dict
        else:
            raise TypeError

        self.flat = make_flat

    def __repr__(self):
        return self.data.__repr__()

    def __getitem__(self, key):
        keys = key.split(".")
        tmp = self.data
        for k in keys:
            if isinstance(tmp, list):
                if k.isdigit():
                    k = int(k)
                    tmp = tmp[k]
                else:
                    tmp = [x[k] for x in tmp if isinstance(x, (Mapping, list))]
                if self.flat:
                    if self._is_nested_list(tmp):
                        tmp = self._flatten(tmp)
            else:
                tmp = tmp[k]

        return tmp

    # def __setitem__(self, key, value):
    #     self.values[key] = value

    # def __delitem__(self, key):
    #     del self.values[key]

    def get(self, key):
        return self.data.get(key)

    def keys(self):
        return self.data.keys()

    def values(self):
        return self.data.values()

    def items(self):
        return self.data.items()

    @staticmethod
    def _flatten(z):
        return [x for y in z for x in y]

    @staticmethod
    def _is_nested_list(z):
        try:
            if isinstance(z[0][0], str):
                return False
            return True
        except:
            return False

    def __iter__(self):
        return self.data.__iter__()

    def __len__(self):
        return self.data.__len__()


def test_slicer():
    h = Slicer(OrderedDict(
        {"hallo": {"derp": [{"doei": {"sjon": [15, 17]}}, {"doei": 20}]}}), make_flat=True)
    h = Slicer({
        "hallo": {
            "derp": [
                {
                    "doei": {
                        "sjon": [15, 17]
                    }
                },
                {
                    "doei": 20
                }
            ]
        }
    }, make_flat=True)


    assert [15, 17] == h['hallo.derp.doei.sjon']



if __name__ == '__main__':
    try:
        piped_input = sys.stdin.read()
        data = json.loads(piped_input)

        first_arg = sys.argv[1]

        print(Slicer(data, make_flat=True)[first_arg])
    except json.decoder.JSONDecodeError:
        print('invalid JSON')
    except IndexError:
        print('expecting filter argument')
