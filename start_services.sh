#!/bin/bash

# Check the environment variable
if [[ -z "$JWT_SECRET" ]]; then
    # If not set, generate a random 64-character string
    JWT_SECRET=$(openssl rand -base64 64)
    export JWT_SECRET
fi

# Function to start a service
start_service() {
    directory="$1"
    if [[ -d "$directory" ]]; then
        cd "$directory" || return
        cargo watch -c -w src -x run &
        pids+=($!)
        echo "Starting $directory ..."
        cd ..
    fi
}

# Function to stop all services
stop_services() {
    for pid in "${pids[@]}"; do
      if ps -p $pid > /dev/null
      then
        kill "$pid" >/dev/null 2>&1
      fi
    done
    exit 0
}

# Array to store the child process IDs
pids=()

# Trap the SIGINT (Ctrl+C) signal to stop all services
trap stop_services SIGINT

# Start the services
start_service "sonar"
start_service "star"
start_service "store"

# Wait for 3 seconds
sleep 5

# Check if all services are running
all_services_running=true
for pid in "${pids[@]}"; do
    if ps -p $pid > /dev/null
    then
      echo "$pid is running"
    else
      all_services_running=false
      break
    fi
done

# Start satellite service if all services are running
if "$all_services_running"; then
    start_service "satellite"
else
    echo "Failed to start all services."
    stop_services
fi

# Wait for child processes to finish
wait

