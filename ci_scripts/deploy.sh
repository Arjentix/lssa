#!/usr/bin/env bash
set -e

# Base directory for deployment
LSSA_DIR="/opt/lssa"

# Expect GITHUB_ACTOR to be passed as first argument or environment variable
GITHUB_ACTOR="${1:-${GITHUB_ACTOR:-unknown}}"

# Function to log messages with timestamp
log_deploy() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S %Z')] $1" >> "${LSSA_DIR}/deploy.log"
}

# Error handler
handle_error() {
  echo "✗ Deployment failed by: ${GITHUB_ACTOR}"
  log_deploy "Deployment failed by: ${GITHUB_ACTOR}"
  exit 1
}

# Set trap to catch any errors
trap 'handle_error' ERR

# Log deployment info
log_deploy "Deployment initiated by: ${GITHUB_ACTOR}"

# Navigate to code directory
cd "${LSSA_DIR}/code"

# Stop current sequencer if running
echo "Stopping current sequencer..."
if pgrep -f sequencer_runner > /dev/null; then
  pkill -SIGINT -f sequencer_runner || true
  sleep 2
  # Force kill if still running
  pkill -9 -f sequencer_runner || true
fi

# Clone or update repository
if [ -d ".git" ]; then
  echo "Updating existing repository..."
  git fetch origin
  git checkout main
  git reset --hard origin/main
else
  echo "Cloning repository..."
  git clone https://github.com/vacp2p/nescience-testnet.git .
  git checkout main
fi

# Build sequencer_runner and wallet in release mode
echo "Building sequencer_runner and wallet..."
cargo build --release --bin sequencer_runner --bin wallet

# Run sequencer_runner with config
echo "Starting sequencer_runner..."
nohup ./target/release/sequencer_runner --config "${LSSA_DIR}/configs/sequencer" > "${LSSA_DIR}/sequencer.log" 2>&1 &

# Wait 5 seconds and check health using wallet
sleep 5
if ./target/release/wallet command check-health; then
  echo "✓ Sequencer started successfully and is healthy"
  log_deploy "Deployment completed successfully by: ${GITHUB_ACTOR}"
  exit 0
else
  echo "✗ Sequencer failed health check"
  tail -n 50 "${LSSA_DIR}/sequencer.log"
  handle_error
fi
