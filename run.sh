#!/bin/bash

read -p "Enter the number of worker servers: " num_workers

declare -a server_addresses
declare -a server_pids
temp_file=$(mktemp)

for ((i=1; i<=$num_workers; i++))
do
    echo "Starting worker server $i..."

    pushd ./worker > /dev/null
    SERVER_NAME=$i cargo run > "$temp_file" 2>&1 &
    server_pid=$!
    popd > /dev/null

    if [ -z "$server_pid" ]; then # Check if the PID is empty
        echo "Failed to capture PID for worker server $i."
        continue
    fi

    echo "Captured PID: $server_pid" # Debugging output

    while [ ! -s "$temp_file" ]  # Check if file is not empty
    do
        sleep 1
    done
    
    # Read the server address from the temporary file
    server_address=$(grep 'Server running on:' "$temp_file" | cut -d ' ' -f 4-)

    echo "Server address: $server_address"  # Debugging output

    # Store the server address and PID in the arrays
    server_addresses[$i]="http://$server_address"
    server_pids[$i]=$server_pid
    
    # Clear the temporary file for the next iteration
    > "$temp_file"
done

rm "$temp_file"

echo "Worker servers started:"
for ((i=1; i<=${#server_addresses[@]}; i++))
do
    echo "Server address: ${server_addresses[$i]}, PID: ${server_pids[$i]}"
done

temp_lb_output=$(mktemp)

echo "Starting load balancer..."
pushd ./load_balancer > /dev/null
LB_WORKER_HOSTS=$(IFS=,; echo "${server_addresses[*]}")
export LB_WORKER_HOSTS
cargo run > "$temp_lb_output" 2>&1 &
load_balancer_pid=$!
popd > /dev/null

while [ ! -s "$temp_lb_output" ]  # Check if file is not empty
do
    sleep 1
done

lb_address=$(grep -o 'Listening on http://[0-9\.]*:[0-9]*' "$temp_lb_output")
if [ -n "$lb_address" ]; then
    echo "$lb_address"
else
    echo "Load balancer address not found in output."
fi

rm "$temp_lb_output"

read -p "Press Enter to shut down the servers..."

echo "Shutting down worker servers..."
for pid in "${server_pids[@]}"
do
    if [ -n "$pid" ]; then  # Check if the PID is non-empty
        echo "Stopping server with PID $pid..."
        kill "$pid"
    else
        echo "PID is empty, skipping..."
    fi
done

echo "Shutting down load balancer..."
echo "Stopping server with PID $load_balancer_pid..."
kill "$load_balancer_pid"