#!/bin/sh
nitro-cli terminate-enclave --all
FILE=nitro-enclave.eif
if [ -f "$FILE" ]; then
    rm $FILE
fi
docker rmi -f $(docker images -a -q)
docker build --no-cache ./ -t nitroimg
nitro-cli build-enclave --docker-uri nitroimg:latest --output-file nitro-enclave.eif
numactl --membind=0 nitro-cli run-enclave --cpu-count 4 --memory 29000 --eif-path nitro-enclave.eif --enclave-cid 88 --debug-mode --attach-console
