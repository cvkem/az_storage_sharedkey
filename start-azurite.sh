
#  Start/execute this file via command:  source start-azurite.sh

# You can also use docker if you prefer  (not explicitely tested, but the interfaces should be compatible)
container_runtime=podman


echo "Starting Azurite-emulator (blob-storage only)" 
echo for more information see:  https://docs.azure.cn/en-us/storage/common/storage-connect-azurite?tabs=blob-storage
echo
echo Using defaults storage-account:
echo "    Account name: devstoreaccount1"
echo "    Account key: Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw=="
echo
echo "Find the container-id: podman ps" 
echo "Exec shell in container: podman exec -it 996c81dc00ab /bin/sh"
echo "watch logs:   tail -f /tmp/debug.log" 
echo


# just the blob service (production-style-url) with debugging
${container_runtime} run -p 10000:10000 mcr.microsoft.com/azure-storage/azurite \
	azurite-blob \
	--blobHost 0.0.0.0 \
	--debug /tmp/debug.log


