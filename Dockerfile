# Stage 1: Build the listener binary
FROM rust:latest AS builder

WORKDIR /usr/src/kalypso-unified

COPY kalypso-unified .

RUN cargo build --release --bin listener

# Stage 2: Final image
FROM rust:latest

WORKDIR /usr/kalypso

COPY sp1_setup.sh .

RUN bash sp1_setup.sh
RUN echo 'export PATH=$PATH:/root/.sp1/bin' >> /root/.profile
RUN /bin/sh -c ". /root/.profile && sp1up"

# Copy the built listener binary from the builder stage
COPY --from=builder /usr/src/kalypso-unified/target/release/listener /usr/kalypso/kalypso-listener

COPY generator_config ./generator_config
COPY sp1 ./sp1

# replace this with git clone
COPY rsa/program ./sp1/examples/kalypso-program/program

COPY rsa/script ./sp1/examples/kalypso-program/script

COPY start.sh .
RUN chmod +x start.sh

CMD ["/bin/sh", "-c", ". /root/.profile && ./start.sh"]
