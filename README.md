# az_storage_sharedkey
Handling sharedkey authentication for Azure storage accounts


Initial and working version in the form of a binary file.
Next step is cleaning up this version and transform it to a library including the tests.
After that the interface can be improved to make it easier to consume.


In order to run the program you first need to start the Azurate-emulator via sourcing the script 

```console
$ source start-azurite.sh
```

The script assumes that you have `podman` installed, but changing to 'docker' is easy.

