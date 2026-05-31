#!/usr/bin/env bash
# Deploy DemoUSDC + MockVerifier + SimplePrivacyPool to Arbitrum Sepolia.
#
# Prerequisites:
#   cp .env.example .env   # fill in PRIVATE_KEY (and ETHERSCAN_API_KEY for verification)
#   forge build            # ensure contracts compile
#
# Usage (run from the privacy-pool/ directory):
#   bash script/deploy.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

CHAIN_ID=421614  # Arbitrum Sepolia
VERIFICATION_DELAY=5  # seconds between verification retries

# ── Load .env from the project root (privacy-pool/) ───────────────────────────
ENV_FILE="$ROOT_DIR/.env"
if [[ ! -f "$ENV_FILE" ]]; then
  echo "Error: $ENV_FILE not found. Run: cp .env.example .env and fill in PRIVATE_KEY." >&2
  exit 1
fi
# shellcheck disable=SC1090
set -a; source "$ENV_FILE"; set +a

# ── Validate required vars ─────────────────────────────────────────────────────
if [[ -z "${PRIVATE_KEY:-}" ]]; then
  echo "Error: PRIVATE_KEY is not set in .env" >&2
  exit 1
fi

RPC_URL="${ARBITRUM_SEPOLIA_RPC_URL:-https://sepolia-rollup.arbitrum.io/rpc}"

# ── Build forge script args ────────────────────────────────────────────────────
FORGE_CMD=(
  script script/Deploy.s.sol
  --rpc-url "$RPC_URL"
  --private-key "$PRIVATE_KEY"
  --broadcast
)

# Etherscan v2 verification — enabled when ETHERSCAN_API_KEY is present and non-empty.
# --chain-id lets Forge resolve the correct Etherscan endpoint automatically (no --verifier-url needed).
if [[ -n "${ETHERSCAN_API_KEY:-}" ]]; then
  echo "ETHERSCAN_API_KEY found — verification enabled (Etherscan v2)"
  FORGE_CMD+=(--verify --chain-id "$CHAIN_ID" --etherscan-api-key "$ETHERSCAN_API_KEY" --delay "$VERIFICATION_DELAY")
else
  echo "ETHERSCAN_API_KEY not set — skipping verification (set it in .env to verify)"
fi

# ── Run ────────────────────────────────────────────────────────────────────────
cd "$ROOT_DIR"
echo "Deploying to Arbitrum Sepolia (chain $CHAIN_ID, rpc: $RPC_URL)..."
forge "${FORGE_CMD[@]}"
