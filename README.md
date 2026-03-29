# az_storage_sharedkey
Handling sharedkey authentication for Azure storage accounts

[Azure storage](https://learn.microsoft.com/en-us/rest/api/storageservices/) has the option to authenticate reqeusts via a storage-account key (next to other more secure options like using EntraID). However, when working with storage keys you 
need to compute a HMAC over part of the request, as described by [microsoft](https://learn.microsoft.com/en-us/rest/api/storageservices/authorize-with-shared-key#blob-queue-and-file-services-shared-key-authorization). This library allows you to easily and idiomatically with Azure shared keys in Rust.




## Testing
This 'az_storage_shared_key' library has a series of unit-tests, which can be fired with `cargo test`.
If you run this you will see that the unit-tests all succeed, but the integration-test (see `tests/azure_test.rs`) as this test will actually try to create a blob in storage. In order to let this test succeed you need to run the Azurate-emulator via the next command. 

```console
$ podman run -p 10000:10000 mcr.microsoft.com/azure-storage/azurite \
	azurite-blob \
	--blobHost 0.0.0.0 \
	--debug /tmp/debug.log
```
This command will start the Azurite Storage emulator (in linux) on localhost:10000 and it will use the same URL-structure as you find on Azure.
The test-program assumes the service is running at 'devstoreaccount1.azurite.local:10000', so in order to make the tests succeed you need to add the
line:
```console
127.0.0.1       devstoreaccount1.azurite.local
```
to the file '/etc/hosts' on linux.The script is configured to start Azure with Production-like URL's. 

The above command will run the storage-service in a container via `podman`. However, the same line of code should work with `docker`, or you can run azurite natively on your local machine.

If you want to inspect the generated logs you need to run a shell inside the container via:
```console
podman exec -it $(podman ps |grep azurite |cut -d " " -f 1) /bin/sh
```
And within this new shell run:
```console
$ tail -f /tmp/debug.log
```
to track the logs as they are created in azurite.


