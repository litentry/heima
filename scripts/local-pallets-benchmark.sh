#!/usr/bin/env bash

# This script can be used for running litentry's benchmarks.
#
# The litentry binary is required to be compiled with make build-node-benchmarks
# in release mode.

set -e

BINARY="./target/release/litentry-collator"


if [[ ! -f "${BINARY}" ]]; then
    echo "binary '${BINARY}' does not exist."
    echo "ensure that the litentry binary is compiled with 'make build-node-benchmarks' and in release mode."
    exit 1
fi

function help {
    echo "USAGE:"
    echo "  ${0} litentry|litmuts|rococo pallet_name benchmark_method  [--check]"
    echo ""
    echo "EXAMPLES:"
    echo "  ${0}                       " "list all benchmarks and provide a selection to choose from"
    echo "  ${0} --check               " "list all benchmarks and provide a selection to choose from, runs in 'check' mode (reduced steps and repetitions)"
    echo "  ${0} foo bar               " "run a benchmark for pallet 'foo' and benchmark 'bar'"
    echo "  ${0} foo bar --all         " "run a benchmark for all pallets"
    echo "  ${0} foo bar --all --check " "run a benchmark for all pallets in 'check' mode (reduced steps and repetitions)"
}

WEIGHTS_PATH="$(pwd)/weights"
if [ ! -d $WEIGHTS_PATH ];then
  mkdir -p  weights/

fi

CHAIN_TYPE="--chain=${1}-dev"
PALLET=${2//-/_}
EXTRINSIC=${3//-/_}

function choose_and_bench {
    readarray -t options < <(${BINARY} benchmark pallet --list $CHAIN_TYPE | sed 1d)
    options+=('EXIT')

    select opt in "${options[@]}"; do
        IFS=', ' read -ra parts <<< "${opt}"
        [[ "${opt}" == 'EXIT' ]] && exit 0

        bench "${parts[0]}" "${parts[1]}" "${2}"
        break
    done
}

function bench {
    echo "pallet  ${1}-weights.rs"
    OUTPUT="--output=$(pwd)/weights/${1}_weights.rs"

    echo "benchmarking '${1}::${2}' --check=${3}, writing results to '${OUTPUT}'"

    if [[ $PALLET == *"parachain_staking"* ]]; then
        echo "will run ${1} benchmark code"
        STEPS=25
        REPEAT=20
    else
        echo "will run other pallet (${1}) benchmark code"
        STEPS=20
        REPEAT=50
    fi

    WASMTIME_BACKTRACE_DETAILS=1 ${BINARY} benchmark pallet \
              $CHAIN_TYPE \
              --execution=wasm  \
              --db-cache=20 \
              --wasm-execution=compiled \
              --pallet="$PALLET" \
              --extrinsic="$EXTRINSIC" \
              --heap-pages=4096 \
              --steps="$STEPS" \
              --repeat="$REPEAT" \
              --header=./LICENSE_HEADER \
              $TEMPLATE \
              $OUTPUT

}

if [[ "${@}" =~ "--help" ]]; then
    help
else
    CHECK=0
    if [[ "${@}" =~ "--check" ]]; then
        CHECK=1
        set -o noglob && set -- ${@/'--check'} && set +o noglob
    fi

    ALL=0
    if [[ "${@}" =~ "--all" ]]; then
        ALL=1
    fi

    if [[ "${ALL}" -eq 1 ]]; then
        bench '*' '*' "${CHECK}" "weights/"
    elif [[ $# -ne 2 ]]; then
        choose_and_bench "${CHECK}"
    else
        bench "${2}" "${3}" "${CHECK}"
    fi
fi
