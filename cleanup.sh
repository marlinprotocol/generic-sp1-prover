docker rm -f `docker ps -aq`
docker rmi $(docker images -f "dangling=true" -q)
docker rm -f kalypso-sp1-prover