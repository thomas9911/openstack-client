## Openstack rust client

super not ready for anything.

Depends heavily on rust-openstack.

### generate fmt_methods.rs command

windows:
```sh
py scripts\get-methods-from-rustdoc.py > src\fmt_methods.rs
```

linux:
```sh
python3 scripts/get-methods-from-rustdoc.py > src/fmt_methods.rs
```