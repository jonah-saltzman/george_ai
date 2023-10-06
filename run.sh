#!/bin/bash

# Navigate to frontend and build it
cd frontend && trunk build --release && cd ..

# Navigate to backend and run it
cd backend && cargo run --release
