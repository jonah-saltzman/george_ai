#!/bin/bash

RELEASE_FLAG=""

# Check for --release argument
if [[ $1 == "--release" ]]; then
    RELEASE_FLAG="--release"
fi

# Navigate to frontend and build it
cd frontend && trunk build $RELEASE_FLAG && cd ..

# Navigate to backend and run it
cd backend && cargo run $RELEASE_FLAG
