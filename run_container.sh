#!/bin/sh

docker build -t sp1-prover .

docker rm -f `docker ps -aq`
docker rmi $(docker images -f "dangling=true" -q)
docker rm -f kalypso-sp1-prover

docker run -d --name kalypso-sp1-prover --restart always sp1-prover
docker logs kalypso-sp1-prover -f
