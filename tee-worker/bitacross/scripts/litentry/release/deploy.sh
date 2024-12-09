#!/bin/bash

set -eo pipefail

# This script is used to perform actions on the target host, including:
# - generate: generate the systemd service files from the template
# - restart: restart the parachain, or the worker, or both
# - upgrade-worker: uprade the worker0 to the rev in local repo
#
# TODO:
# the combinations of flags are not yet well verified/organised, especially the following:
# --only-worker
# --build
# --discard

# ------------------------------
# path setting
# ------------------------------

ROOTDIR=$(git rev-parse --show-toplevel)
BASEDIR=/opt/litentry
PARACHAIN_BASEDIR="$BASEDIR/parachain"
WORKER_BASEDIR="$BASEDIR/worker"
BACKUP_BASEDIR="$BASEDIR/backup"
LOG_BACKUP_BASEDIR="$BACKUP_BASEDIR/log"
WORKER_BACKUP_BASEDIR="$BACKUP_BASEDIR/worker"
RELAYCHAIN_ALICE_BASEDIR="$PARACHAIN_BASEDIR/relay-alice"
RELAYCHAIN_BOB_BASEDIR="$PARACHAIN_BASEDIR/relay-bob"
PARACHAIN_ALICE_BASEDIR="$PARACHAIN_BASEDIR/para-alice"

# ------------------------------
# default arg setting
# ------------------------------

BUILD=false
DISCARD=false
WORKER_CONFIG=
CHAIN=paseo
ONLY_WORKER=false
PARACHAIN_HOST=localhost
PARACHAIN_PORT=9944
DOCKER_IMAGE=litentry/litentry-parachain:tee-prod
COPY_FROM_DOCKER=false
PRODUCTION=false
ACTION=

# ------------------------------
# Some global setting
# ------------------------------

WORKER_COUNT=
PARACHAIN_ID=
OLD_MRENCLAVE=
NEW_MRENCLAVE=
OLD_SHARD=
LATEST_FINALIZED_BLOCK=

SGX_SDK=/opt/intel/sgxsdk
SGX_ENCLAVE_SIGNER=$SGX_SDK/bin/x64/sgx_sign

# ------------------------------
# main()
# ------------------------------

function main {
  # 0/ check if $USER has sudo
  if sudo -l -U $USER 2>/dev/null | grep -q 'may run the following'; then
    source "$SGX_SDK/environment"
  else
    echo "$USER doesn't have sudo permission"
    exit 1
  fi

  # 1/ create folders if missing
  sudo mkdir -p "$BASEDIR"
  sudo chown $USER:$GROUPS "$BASEDIR" 
  for d in "$LOG_BACKUP_BASEDIR" "$WORKER_BACKUP_BASEDIR" "$RELAYCHAIN_ALICE_BASEDIR" "$RELAYCHAIN_BOB_BASEDIR" \
    "$PARACHAIN_ALICE_BASEDIR" "$WORKER_BASEDIR"; do
    mkdir -p "$d"
  done

  # 2/ parse command lines
  echo "Parsing command line ..."
  while [ $# -gt 0 ]; do
    case "$1" in
      -h|--help)
        display_help
        exit 0
        ;;
      -b|--build)
        BUILD=true
        shift
        ;;
      -d|--discard)
        DISCARD=true
        shift
        ;;
      -c|--config)
        WORKER_CONFIG="$(realpath -s $2)"
        shift 2
        ;;
      -a|--only-worker)
        ONLY_WORKER=true
        shift
        ;;
      -x|--chain)
        CHAIN="$2"
        shift 2
        ;;
      -p|--parachain-port)
        PARACHAIN_PORT="$2"
        shift 2
        ;;
      -z|--parachain-host)
        PARACHAIN_HOST="$2"
        shift 2
        ;;
      -v|--copy-from-docker)
        COPY_FROM_DOCKER=true
        DOCKER_IMAGE="$2"
        shift 2
        ;;
      --prod)
        PRODUCTION=true
        shift
        ;;
      generate|restart|upgrade-worker)
        ACTION="$1"
        shift
        ;;
      *)
        echo "Error: unknown option or subcommand $1"
        display_help
        exit 1
        ;;
    esac
  done

  # 3/ sanity checks
  if [ ! -f "$WORKER_CONFIG" ]; then
    echo "Worker config not found: $WORKER_CONFIG"
    exit 1
  fi

  WORKER_COUNT=$(cat "$WORKER_CONFIG" | jq '.workers | length')
  echo "Worker count: $WORKER_COUNT"

  # TODO: check flags conflict, e.g.
  # - having `--discard` together with `upgrade-worker` doesn't make sense
  # - `upgrade-worker` should ignore the `--only-worker` flag

  # 4/ main business logic
  case "$ACTION" in
    generate)
      backup_services
      generate_services
      exit
      ;;
    restart)
      backup_logs
      backup_workers
      stop_services
      prune
      build
      setup_working_dir
      if [ "$ONLY_WORKER" = true ]; then
        remove_clean_reset
      fi
      restart_services
      exit
      ;;
    upgrade-worker)
      # build the new worker, the code must be under $ROOTDIR/tee-worker already
      build_worker
      # update the schedule
      set_scheduled_enclave

      # wait until sidechain stalls
      wait_for_sidechain
      backup_workers
      stop_worker_services
      get_old_mrenclave
      # TODO: actually we only need the copy-up
      setup_working_dir
      migrate_shard
      remove_clean_reset
      restart_services
      exit
      ;;
    *)
      echo "Unknown action: $ACTION"
      exit 1 ;;
  esac
}

# ------------------------------
# helper functions
# ------------------------------

function print_divider {
  echo "------------------------------------------------------------"
}

function display_help {
  echo "usage: ./deploy.sh <subcommands> [options]"
  echo ""
  echo "subcommands:"
  echo "  generate           Generate the parachain and worker systemd files"
  echo "  restart            Restart the services"
  echo "  upgrade-worker     Upgrade the worker"
  echo ""
  echo "options:"
  echo "  -h, --help                  Display this help message and exit"
  echo "  -b, --build                 Build the parachain and worker binaries (default: false)"
  echo "  -d, --discard               Clean the existing state for parachain and worker (default: false)"
  echo "  -c, --config <config.json>  Config file for the worker"
  echo "  -a, --only-worker           Start only the worker (default: false)"
  echo "  -x, --chain                 Chain type for launching the parachain network (default: paseo)"
  echo "  -h, --parachain-host        Parachain ws URL (default: localhost)"
  echo "  -p, --parachain-port        Parachain ws port (default: 9944)"
  echo "  -v, --copy-from-docker      Copy the parachain binary from a docker image (default: litentry/litentry-parachain:tee-prod)"
  echo "  --prod                      Use a prod configuration to build and run the worker (default: false)"
  echo ""
  echo "examples:"
  echo "  ./deploy.sh generate --config tmp.json"
  echo "  ./deploy.sh restart --config tmp.json --discard --build"
  echo "  ./deploy.sh restart --config tmp.json --only-worker"
  echo "  ./deploy.sh upgrade-worker --config tmp.json --only-worker"
  echo ""
  echo "notes:"
  echo "  - This script requires an OS that supports systemd."
  echo "  - It is mandatory to provide a JSON config file for the worker."
  echo "  - jq is required to be installed on the system "
  echo ""
  echo "For more information or assistance, please contact Litentry parachain team."
}

# TODO: in fact, this function only backs up the parachain logs
#       maybe we want to remove it as it's not so critical anyway 
function backup_logs {
  echo "Backing up logs ..."
  now=$(date +"%Y%m%d-%H%M%S")
  outdir="$LOG_BACKUP_BASEDIR/log-$now"
  mkdir -p "$outdir"
  cp "$PARACHAIN_BASEDIR"/*.log "$outdir" || true
  echo "Logs backed up into $outdir"
}

function backup_workers {
  echo "Backing up workers ..."
  now=$(date +"%Y%m%d-%H%M%S")
  cd "$WORKER_BASEDIR" || exit
  for i in $(ls -d * 2>/dev/null); do
    outdir="$WORKER_BACKUP_BASEDIR/$i-$now"
    cp -rf "$i" "$outdir"
    echo "Worker backed up into $outdir"
  done
}

function backup_services {
  echo "Backing up services ..."
  now=$(date +"%Y%m%d-%H%M%S")
  cd /etc/systemd/system || exit
  outdir="$WORKER_BACKUP_BASEDIR/service-$now"
  mkdir -p "$outdir"
  for f in para-alice.service relay-alice.service relay-bob.service $(ls worker*.service 2>/dev/null); do
    cp "$f" "$outdir" || true
  done
}

function prune {
  if [ "$DISCARD" = true ]; then
    echo "Pruning the existing state ..."
    rm -rf "$PARACHAIN_BASEDIR"/*
    rm -rf "$WORKER_BASEDIR"/*
  fi
}

function generate_services {
  echo "Generating systemd service files ..."
    cd "$ROOTDIR/tee-worker/scripts/litentry/release"
    cp template/* .
    sed -i "s/CHAIN/$CHAIN/g" *.service
    sed -i "s/USER/$USER/g" *.service
    for ((i = 0; i < WORKER_COUNT; i++)); do
      cp worker.service worker$i.service
      sed -i "s/NUMBER/$i/g" worker$i.service
      # populate args
      flags=$(cat "$WORKER_CONFIG" | jq -r ".workers[$i].flags[]")
      subcommand_flags=$(cat "$WORKER_CONFIG" | jq -r ".workers[$i].subcommand_flags[]")
      args=
      for flag in $flags; do
        args+=" $flag"
      done
      args+=" run"
      for subcommand_flag in $subcommand_flags; do
        args+=" $subcommand_flag"
      done
      sed -i "s;ARGS;$args;" worker$i.service
    done
    rm worker.service
    sudo cp *.service -f /etc/systemd/system/
    rm *.service
    sudo systemctl daemon-reload
    echo "Done, please check files under /etc/systemd/system/"
    echo "Restart the services to take effect"
}

function build_worker {
  echo "Building worker ..."
  cd $ROOTDIR/tee-worker/ || exit 
  if [ "$PRODUCTION" = true ]; then
    # we will get an error if SGX_COMMERCIAL_KEY is not set for prod
    SGX_PRODUCTION=1 make
  else
    # use SW mode for dev
    SGX_MODE=SW make
  fi
}

# TODO: take github rev into consideration
function build {
  if [ "$BUILD" = true ]; then
    echo "Building the parachain and worker binaries ..."

    # download polkadot
    echo "Downloading polkadot binary ..."
  
    for f in polkadot-execute-worker polkadot-prepare-worker polkadot; do
      url="https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-v1.1.0/$f"
      polkadot_bin="$PARACHAIN_BASEDIR/$f"
      wget -O "$polkadot_bin" -q "$url"
      chmod a+x "$polkadot_bin"
      if [ ! -s "$polkadot_bin" ]; then
        echo "$polkadot_bin is 0 bytes, download URL: $url" && exit 1
      fi
    done

    if ! "$polkadot_bin" --version &> /dev/null; then
      echo "Cannot execute $polkadot_bin, wrong executable?" && exit 1
    fi

    # pull or build parachain
    if [ "$COPY_FROM_DOCKER" = true ]; then
      echo "Pulling binary from $DOCKER_IMAGE ..."
      docker pull "$DOCKER_IMAGE"
      docker cp "$(docker create --rm $DOCKER_IMAGE):/usr/local/bin/litentry-collator" "$PARACHAIN_BASEDIR"
    else
      echo "Building parachain binary ..."
      cd "$ROOTDIR" || exit
      if [ "$PRODUCTION" = true ]; then
        cargo build --locked --profile production
      else
        pwd
        make build-node
      fi
      cp "$ROOTDIR/target/release/litentry-collator" "$PARACHAIN_BASEDIR"
    fi
    chmod a+x "$PARACHAIN_BASEDIR/litentry-collator"
  fi
}

function restart_services {
  sudo systemctl daemon-reload
  if [ "$ONLY_WORKER" = false ]; then
    echo "Restarting parachain services ..."

    cd "$PARACHAIN_BASEDIR" || exit
    ./polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo-local-chain-spec.json
    ./litentry-collator export-genesis-state --chain $CHAIN-dev > genesis-state
    ./litentry-collator export-genesis-wasm --chain $CHAIN-dev > genesis-wasm

    sudo systemctl restart relay-alice.service
    sleep 5
    sudo systemctl restart relay-bob.service
    sleep 5
    sudo systemctl restart para-alice.service
    sleep 5
    register_parachain    
  fi

  echo "Restarting worker services ..."
  for ((i = 0; i < WORKER_COUNT; i++)); do
    sudo systemctl restart "worker$i.service"
    sleep 5
  done
  echo "Done"
}

function stop_worker_services {
  echo "Stopping worker services ..."
  for ((i = 0; i < WORKER_COUNT; i++)); do
    sudo systemctl stop "worker$i.service"
    sleep 5
  done
}

function stop_parachain_services {
  echo "Stopping parachain services ..."
  sudo systemctl stop para-alice.service relay-alice.service relay-bob.service
}

function stop_services {
  stop_worker_services

  # TODO: it means we can't stop parachain service alone
  #       this needs to be done directly via `systemctl`
  if [ "$ONLY_WORKER" = false ]; then
    stop_parachain_services
  fi
}

function register_parachain {
  echo "Register parathread now ..."
  cd "$ROOTDIR" || exit
  export PARACHAIN_ID=$(grep -i "${CHAIN}_para_id" primitives/core/src/lib.rs | sed 's/.* = //;s/\;.*//')
  cd "$ROOTDIR/ts-tests" || exit 
  if [[ -z "$NODE_ENV" ]]; then
      echo "NODE_ENV=ci" > .env
  else
      echo "NODE_ENV=$NODE_ENV" > .env
  fi
  # The genesis state path file needs to be updated as it is hardcoded to be /tmp/parachain_dev 
  jq --arg genesis_state "$PARACHAIN_BASEDIR/genesis-state" --arg genesis_wasm "$PARACHAIN_BASEDIR/genesis-wasm" '.genesis_state_path = $genesis_state | .genesis_wasm_path = $genesis_wasm' config.ci.json > config.ci.json.1
  mv config.ci.json.1 config.ci.json
  pnpm install
  pnpm run register-parathread 2>&1 | tee "$PARACHAIN_BASEDIR/register-parathread.log"
  print_divider

  echo "Upgrade parathread to parachain now ..."
  # Wait for 90s to allow onboarding finish, after that we do the upgrade
  sleep 90
  pnpm run upgrade-parathread 2>&1 | tee "$PARACHAIN_BASEDIR/upgrade-parathread.log"
  print_divider

  echo "done. please check $PARACHAIN_BASEDIR for generated files if need"
  print_divider
  git restore config.ci.json
}

function setup_working_dir {
    echo "Setting up working dir ..."
    cd "$ROOTDIR/tee-worker/bin" || exit

    if [ "$PRODUCTION" = false ]; then
      for f in 'key.txt' 'spid.txt'; do
        [ -f "$f" ] || touch "$f"
      done
    fi

    for ((i = 0; i < WORKER_COUNT; i++)); do
      worker_dir="$WORKER_BASEDIR/w$i"
      mkdir -p "$worker_dir"
      for f in 'key.txt' 'spid.txt' 'enclave.signed.so' 'litentry-worker'; do
        [ -f "$f" ] && cp -f "$f" "$worker_dir"
      done

      cd "$worker_dir"
      [ -f light_client_db.bin/db.bin.backup ] && cp -f light_client_db.bin/db.bin.backup light_client_db.bin/db.bin

      enclave_account=$(./litentry-worker signing-key | grep -oP '^Enclave account: \K.*$$')

      if [ "$PRODUCTION" = true ]; then
        echo "Transferring balance to the enclave account $enclave_account ..."
        cd $ROOTDIR/scripts/ts-utils/ || exit
        pnpm install
        pnpm exec ts-node transfer.ts $enclave_account
      fi
    done
}

function get_old_mrenclave {
  cd "$WORKER_BASEDIR/w0" || exit
  OLD_SHARD=$(./litentry-worker mrenclave)
  $SGX_ENCLAVE_SIGNER dump -enclave ./enclave.signed.so -dumpfile df.out
  OLD_MRENCLAVE=$($ROOTDIR/tee-worker/extract_identity < df.out | awk '{print $2}')
  rm df.out
  echo "old shard: $OLD_SHARD"
  echo "old mrenclave: $OLD_MRENCLAVE"
}

function set_scheduled_enclave {
  echo "Setting scheduled enclave ..."
  cd $ROOTDIR/tee-worker || exit 
  NEW_MRENCLAVE=$(make mrenclave 2>&1 | grep MRENCLAVE | awk '{print $2}')
  echo "new mrenclave: $NEW_MRENCLAVE"

  latest_sidechain_block

  echo "Setting up the new worker on chain ..."
  cd $ROOTDIR/ts-tests/ || exit 
  pnpm install
  pnpm run setup-enclave $NEW_MRENCLAVE $SCHEDULED_UPDATE_BLOCK
}

function wait_for_sidechain {
  echo "Waiting for sidechain to reach block $SCHEDULED_UPDATE_BLOCK ..."
  found=false
  for _ in $(seq 1 30); do
    sleep 20
    block_number=$(grep -F 'Enclave produced sidechain blocks' $WORKER_BASEDIR/w0/worker.log | tail -n 1 | sed 's/.*\[//;s/]//')
    echo "current sidechain block: $block_number"
    if [ $((block_number+1)) -eq $SCHEDULED_UPDATE_BLOCK ]; then
      echo "we should stall soon ..."
    fi
    if tail -n 50 $WORKER_BASEDIR/w0/worker.log | grep -q "Skipping sidechain block $SCHEDULED_UPDATE_BLOCK due to mismatch MRENCLAVE"; then
      echo "we reach $SCHEDULED_UPDATE_BLOCK now"
      found=true
      break
    fi
  done
  if [ $found = false ]; then
    echo "not reached, timeout"
    exit 1
  fi
}

function migrate_shard {
  echo "Migrating shards for workers ..."
  for ((i = 0; i < WORKER_COUNT; i++)); do
    cd "$WORKER_BASEDIR/w$i" || exit
    echo "old MRENCLAVE: $OLD_MRENCLAVE"
    echo "new MRENCLAVE: $NEW_MRENCLAVE"
    ./litentry-worker migrate-shard --old-shard $OLD_MRENCLAVE --new-shard $NEW_MRENCLAVE

    cd shards || exit
    rm -rf $OLD_SHARD
  done
  echo "Done"
}

function remove_clean_reset {
  echo "Removing --clean-reset flag for workers ..."
  for ((i = 0; i < WORKER_COUNT; i++)); do
    sudo sed -i 's/--clean-reset//' /etc/systemd/system/worker$i.service
  done
  echo "Done"
}

# TODO: here we only read worker0 logs here
function latest_sidechain_block {
  block_number=$(grep -F 'Enclave produced sidechain blocks' $WORKER_BASEDIR/w0/worker.log | tail -n 1 | sed 's/.*\[//;s/]//')
  SCHEDULED_UPDATE_BLOCK=$((block_number + 30))
  echo "Current sidechain block: $block_number, scheduled update block: $SCHEDULED_UPDATE_BLOCK"
}

# TODO: unused
function _latest_parentchain_block {
  # JSON-RPC request payload
  request='{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}'

  # Make the JSON-RPC request and retrieve the latest finalized block
  response=$(curl -s -H "Content-Type: application/json" -d "$request" http://$PARACHAIN_HOST:$PARACHAIN_PORT)
  hex_number=$(echo "$response" | grep -oP '(?<="number":")[^"]+')
  LATEST_FINALIZED_BLOCK=$(printf "%d" "$hex_number")
  echo "Current parachain block: $LATEST_FINALIZED_BLOCK"
}

main "$@"
