#!/bin/bash

if [[ -z "$ENV" ]]; then
    export ENV="DEV"
fi

# Start depencies
cd ./sonar && ./start_sonar.sh &
cd ./store && ./start_store.sh &
cd ./star && ./start_star.sh &

sleep 5

if pgrep -x "start_sonar.sh" && pgrep -x "start_store.sh" && pgrep -x "start_star.sh"; then
    cd ./satellite && ./start_satellite.sh
fi
