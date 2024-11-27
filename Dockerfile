# Stage 1: Build the listener binary


# Stage 2: Final image
FROM ubuntu:22.04

# Install dependency tools
RUN apt-get update && apt-get install -y \
    net-tools iptables iproute2 wget bash git curl \
    libc++1 libc++abi1 jq && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# COPY sp1_setup.sh .
# RUN curl -L "https://raw.githubusercontent.com/succinctlabs/sp1/main/sp1up/sp1up" | bash
# RUN bash sp1_setup.sh
# RUN echo 'export PATH=$PATH:/root/.sp1/bin' >> /root/.profile
# RUN /bin/sh -c ". /root/.profile && sp1up"

# ENV PATH="/root/.sp1/bin:${PATH}"
# RUN echo 'export PATH="$HOME/.sp1/bin:$PATH"' >> ~/.bashrc && \
#     bash -c "source ~/.profile && sp1up"

COPY prover ./
COPY listener ./
COPY generator_client ./
# COPY generator_config ./generator_config
# COPY sp1 ./sp1

# RUN cd ./sp1/examples/fibonacci && cargo build --release --bin fibonacci-script

# Download all necessary components and set permissions in a single RUN command to reduce layers
RUN wget -O supervisord http://public.artifacts.marlin.pro/projects/enclaves/supervisord_master_linux_amd64 && \
    chmod +x supervisord && \
    wget -O ip-to-vsock-transparent http://public.artifacts.marlin.pro/projects/enclaves/ip-to-vsock-transparent_v1.0.0_linux_amd64 && \
    chmod +x ip-to-vsock-transparent && \
    wget -O keygen http://public.artifacts.marlin.pro/projects/enclaves/keygen_v1.0.0_linux_amd64 && \
    chmod +x keygen && \
    wget -O attestation-server http://public.artifacts.marlin.pro/projects/enclaves/attestation-server_v2.0.0_linux_amd64 && \
    chmod +x attestation-server && \
    wget -O vsock-to-ip http://public.artifacts.marlin.pro/projects/enclaves/vsock-to-ip_v1.0.0_linux_amd64 && \
    chmod +x vsock-to-ip && \
    wget -O dnsproxy http://public.artifacts.marlin.pro/projects/enclaves/dnsproxy_v0.46.5_linux_amd64 && \
    chmod +x dnsproxy && \
    wget -O oyster-keygen http://public.artifacts.marlin.pro/projects/enclaves/keygen-secp256k1_v1.0.0_linux_amd64 && \
    chmod +x oyster-keygen 

COPY setup.sh ./
RUN chmod +x setup.sh

# supervisord config
COPY supervisord.conf /etc/supervisord.conf

COPY ./app/id.pub ./app/id.sec ./app/secp.pub ./app/secp.sec ./

# entry point
ENTRYPOINT [ "/app/setup.sh" ]
