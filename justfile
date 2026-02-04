# Load environment variables from .env file
set dotenv-load

# Default recipe: list available commands
default:
    @just --list

# ==========================================
# Native Development
# ==========================================

# Run the application locally
run:
    cargo run

# Build the application in release mode
build:
    cargo build --release

# Check the code for compilation errors
check:
    cargo check

# Run the test suite
test:
    cargo test --workspace

# Format the code using rustfmt
fmt:
    cargo fmt

# Run clippy lints
clippy:
    cargo clippy -- -D warnings

# ==========================================
# Database Management
# ==========================================

# Create the database using sqlx
db-create:
    sqlx database create

# Run database migrations
db-migrate:
    sqlx migrate run

# Revert the last migration
db-revert:
    sqlx migrate revert

# Drop the database
db-drop:
    sqlx database drop --force

# Reset the database: drop, create, and migrate
db-reset: db-drop db-create db-migrate

# Prepare SQLx offline data (required for Docker build)
# This generates/updates sqlx-data.json based on current queries
db-prepare:
    cargo sqlx prepare --workspace

# ==========================================
# Docker Operations
# ==========================================

# Build the Docker image (automatically runs db-prepare first)
docker-build: db-prepare
    docker build -t cayopay-server .

# Run the Docker container interactively
# Maps host port 3000 to container port 3000
# Passes .env file for configuration
# --init ensures signals are forwarded (fixing Ctrl+C)
docker-run:
    docker run --rm -it --init --name cayopay-server -p $PORT:$PORT --env-file .env.docker cayopay-server

# Run the Docker container in the background (detached)
docker-up:
    docker run --rm -d --init --name cayopay-server -p $PORT:$PORT --env-file .env.docker cayopay-server

# Stop the running Docker container
docker-stop:
    docker stop cayopay-server
