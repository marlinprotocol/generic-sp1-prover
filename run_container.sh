#!/bin/sh

docker build -t sp1-prover .

docker rm -f `docker ps -aq`
docker rmi $(docker images -f "dangling=true" -q)
docker rm -f kalypso-sp1-prover

docker run -d --name market-20 --restart always sp1-prover
docker logs market-20 -f
