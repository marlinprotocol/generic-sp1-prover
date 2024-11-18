# Files Overview
## 1. Dockerfile
Used to create the Docker container for the Nitro Enclave.
Sets up the environment and builds an Enclave Image File (EIF).
Uses supervisord as the entry point to manage the prover binary inside the enclave.
## 2. supervisord.conf
Specifies the entry point and process management inside the Nitro Enclave.
Manages the lifecycle of the Fibonacci prover binary.
## 3. build.sh
Orchestrates the process of:
Building the Docker image.
Generating the EIF.
Preparing the Nitro Enclave environment for execution.
## 4. prover/
Contains the precompiled SP1 Fibonacci prover binary.
This binary computes Fibonacci sequences using SP1.

# How to run
- ./build.sh