#!/bin/bash

# Check if the environment variable is set to "DEV"
if [[ "$ENV" == "DEV" ]]; then
    # If set to "DEV", run the cargo command
    cargo watch -c -w src -x run
else
    echo "ENV is not set to DEV. Skipping command."
fi
