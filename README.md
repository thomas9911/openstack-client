## Openstack rust client

A way to communicate to openstack apis using a commandline interface.


Place your `clouds.yaml` next to the openstack-client binary with the following format:

```yaml
clouds:
  your_cloud:
    auth:
      auth_url:
      username:
      password:
      user_domain_id:
      project_domain_id:
      project_id:
    region_name:
    interface:
```


### Design decisions

* input format is switched compared to the official python cli. The format is just like kubectl: \<command> \<resource>
* output is the same as the openstack api and (most of the time) in json format
