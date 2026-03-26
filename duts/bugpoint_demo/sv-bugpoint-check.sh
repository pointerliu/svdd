#!/bin/bash

INPUT_FILE="$1"
WORK_DIR="$(pwd)"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

cd "$SCRIPT_DIR"

if [ ! -f "$INPUT_FILE" ]; then
    INPUT_FILE="$WORK_DIR/$INPUT_FILE"
fi

if [ ! -f "$INPUT_FILE" ]; then
    echo "Input file not found: $INPUT_FILE"
    exit 1
fi

INPUT_FILE="$(cd "$(dirname "$INPUT_FILE")" && pwd)/$(basename "$INPUT_FILE")"

echo "Building with verilator..."
rm -rf obj_dir

output=$(verilator --coverage-line -fno-table --timing --cc --exe \
    tb.cpp "$INPUT_FILE" \
    --build -o sim -CFLAGS "-std=c++17 -I obj_dir" 2>&1)

build_success=$?

if [ $build_success -ne 0 ]; then
    echo "Build failed - code structure broken"
    exit 1
fi

echo "Running simulation..."
sim_output=$(./obj_dir/sim 2>&1)
sim_exit=$?
echo "$sim_output"

if echo "$sim_output" | grep -q "Bug preserved"; then
    echo "Bug is preserved"
    exit 0
elif echo "$sim_output" | grep -q "Bug fixed"; then
    echo "Bug is fixed"
    exit 1
elif echo "$sim_output" | grep -q "Unexpected"; then
    echo "Unexpected behavior"
    exit 1
else
    echo "Unknown state"
    exit 1
fi
