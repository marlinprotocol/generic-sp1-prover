#!/bin/sh

# # Run kalypso-listener in the background and redirect its output to a log file while printing to the console
# ./kalypso-listener 2>&1 | tee kalypso-listener.log &

# # Change directory and run the kalypso-program
# cd sp1/examples
# cargo run --release --bin kalypso-program

# Change directory and build the kalypso-program
cd sp1/examples
cargo build --release --bin kalypso-program

# Run kalypso-program in the background and log to the console and file
cargo run --release --bin kalypso-program 2>&1 | tee kalypso-program.log &

# Return to the initial directory
cd ../..

sleep 60
# Run kalypso-listener in the foreground and log to the console and file
./kalypso-listener
