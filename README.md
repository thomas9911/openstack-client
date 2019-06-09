## Openstack rust client

A way to communicate to openstack apis using a commandline interface.

### Download latest version

Latest versions:
* [Windows](https://object.api.ams.fuga.cloud/swift/v1/5af86bc2f74c49178f32f6f479e878cc/rustci/latest/windows/openstack-client.exe "openstack-client download")
* [OSX](https://object.api.ams.fuga.cloud/swift/v1/5af86bc2f74c49178f32f6f479e878cc/rustci/latest/osx/openstack-client "openstack-client download")
* [Linux](https://object.api.ams.fuga.cloud/swift/v1/5af86bc2f74c49178f32f6f479e878cc/rustci/latest/linux/openstack-client "openstack-client download")

Pick your own
[version](https://object.api.ams.fuga.cloud/swift/v1/5af86bc2f74c49178f32f6f479e878cc/rustci/ "object store link to different versions")

### Configuration

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
