#!/bin/bash

# Check the environment variable
if [[ -z "$JWT_SECRET" ]]; then
    # If not set, generate a random 64-character string
    JWT_SECRET=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 64)
    export JWT_SECRET
fi

if [[ "$ENV" == "DEV" ]]; then
    # If set to "DEV", run the cargo command
    cargo watch -c -w src -x run
else
    echo "ENV is not set to DEV. Skipping command."
fi
