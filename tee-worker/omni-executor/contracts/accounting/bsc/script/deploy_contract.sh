#!/bin/bash

show_help() {
    cat << EOF
Usage: ${0##*/} COMMAND [OPTIONS]

Commands:
  deploy-contract       Deploy a contract with the given options.
  deposit-funds         Deposit funds with the given options.
  withdraw-funds        Withdraw funds with the given options.
  set-admin             Set the admin address.
  set-worker            Set the worker address.
  execute-payment       Execute a payment with the given options.
  is-admin              Check if the given address is the admin.
  is-worker             Check if the given address is the worker.
  balance               Get the balance of the contract.
  get-user-record       Get the payment record of the given user.

Options:
  --contract-address ADDRESS  Specify the contract address.(DEFAULT: 0x5FbDB2315678afecb367f032d93F642f64180aa3)
  --rpc-url URL               Specify the RPC URL.(DEFAULT: http://localhost:8545)
  --private-key KEY           Provide the private key.(DEFAULT: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
  --amount AMOUNT             Specify the amount.(DEFAULT: 10)
  --beneficiary ADDRESS       Specify the user address.(DEFAULT: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)
  --admin ADDRESS             Specify the admin address.(DEFAULT: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)
  --worker ADDRESS            Specify the worker address.(DEFAULT: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)
  --help                      Display this help and exit.

Examples:
  ${0##*/} deploy-contract --contract-address 0x1234 --rpc-url https://example.com --private-key abcdef123456
  ${0##*/} deposit-funds --contract-address 0x1234 --rpc-url https://example.com --private-key abcdef123456 --amount 100
EOF
}

# Ensure a command is provided
if [[ "$#" -lt 1 ]]; then
    show_help
    exit 1
fi

# Capture the command and shift to the options
COMMAND="$1"
shift

# These are for testing with anvil locally
DEFAULT_RPC_URL=http://localhost:8545
DEFAULT_AMOUNT=10
DEFAULT_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
DEFAULT_ADMIN_PUBLIC_KEY=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
DEFAULT_WORKER_PUBLIC_KEY=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
DEFAULT_BENEFICIARY_PUBLIC_KEY=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
DEFAULT_CONTRACT_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --beneficiary)
            BENEFICIARY_PUBLIC_KEY="$2"
            shift 2
            ;;
        --contract-address)
            CONTRACT_ADDRESS="$2"
            shift 2
            ;;
        --amount)
            AMOUNT="$2"
            shift 2
            ;;
        --admin)
            ADMIN_PUBLIC_KEY="$2"
            shift 2
            ;;
        --worker)
            WORKER_PUBLIC_KEY="$2"
            shift 2
            ;;
        --rpc-url)
            RPC_URL="$2"
            shift 2
            ;;
        --private-key)
            PRIVATE_KEY="$2"
            shift 2
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown parameter: $1"
            show_help
            exit 1
            ;;
    esac
done

# Use default values if not provided
RPC_URL="${RPC_URL:-$DEFAULT_RPC_URL}"
AMOUNT="${AMOUNT:-$DEFAULT_AMOUNT}"
WORKER_PUBLIC_KEY="${WORKER_PUBLIC_KEY:-$DEFAULT_WORKER_PUBLIC_KEY}"
ADMIN_PUBLIC_KEY="${ADMIN_PUBLIC_KEY:-$DEFAULT_ADMIN_PUBLIC_KEY}"
PRIVATE_KEY="${PRIVATE_KEY:-$DEFAULT_PRIVATE_KEY}"
CONTRACT_ADDRESS="${CONTRACT_ADDRESS:-$DEFAULT_CONTRACT_ADDRESS}"
BENEFICIARY_PUBLIC_KEY="${BENEFICIARY_PUBLIC_KEY:-$DEFAULT_BENEFICIARY_PUBLIC_KEY}"

# Execute the corresponding logic based on the command
case $COMMAND in
    deploy-contract)
        if [[ -n "$RPC_URL" && -n "$PRIVATE_KEY" ]]; then
            echo "RPC URL: $RPC_URL"
            echo "Admin Public Key: $ADMIN_PUBLIC_KEY"
            echo "Worker Public Key: $WORKER_PUBLIC_KEY"

            export PRIVATE_KEY=$PRIVATE_KEY
            export ADMIN_PUBLIC_KEY=$ADMIN_PUBLIC_KEY
            export WORKER_PUBLIC_KEY=$WORKER_PUBLIC_KEY

            forge script AccountingContract.s.sol:DeployContract --rpc-url $RPC_URL --broadcast
        else
            echo "Missing required arguments."
            show_help
            exit 1
        fi
        ;;

    deposit-funds)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$PRIVATE_KEY" || -z "$AMOUNT" ]]; then
            echo "Missing required arguments for depositing funds."
            show_help
            exit 1
        fi
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Amount: $AMOUNT"

        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS
        export PRIVATE_KEY=$PRIVATE_KEY
        export AMOUNT=$AMOUNT

        forge script AccountingContract.s.sol:DepositFunds --rpc-url $RPC_URL --broadcast
        ;;
    withdraw-funds)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$PRIVATE_KEY" || -z "$AMOUNT" || -z "$BENEFICIARY_PUBLIC_KEY" ]]; then
            echo "Missing required arguments for withdrawing funds."
            show_help
            exit 1
        fi
        echo "Withdrawing funds..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Amount: $AMOUNT"
        echo "Beneficiary Public Key: $BENEFICIARY_PUBLIC_KEY"

        export PRIVATE_KEY=$PRIVATE_KEY
        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS
        export BENEFICIARY_PUBLIC_KEY=$BENEFICIARY_PUBLIC_KEY
        export AMOUNT=$AMOUNT

        forge script AccountingContract.s.sol:WithdrawFunds --rpc-url $RPC_URL --broadcast
        ;;
    set-worker)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$PRIVATE_KEY" || -z "$WORKER_PUBLIC_KEY" ]]; then
            echo "Missing required arguments for setting admin."
            show_help
            exit 1
        fi
        echo "Setting worker..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Worker Public Key: $WORKER_PUBLIC_KEY"

        export PRIVATE_KEY=$PRIVATE_KEY
        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS
        export WORKER_PUBLIC_KEY=$WORKER_PUBLIC_KEY

        forge script AccountingContract.s.sol:SetWorker --rpc-url $RPC_URL --broadcast
        ;;
    set-admin)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$PRIVATE_KEY" || -z "$ADMIN_PUBLIC_KEY" ]]; then
            echo "Missing required arguments for setting admin."
            show_help
            exit 1
        fi
        echo "Setting admin..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Admin Public Key: $ADMIN_PUBLIC_KEY"

        export PRIVATE_KEY=$PRIVATE_KEY
        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS
        export ADMIN_PUBLIC_KEY=$ADMIN_PUBLIC_KEY

        forge script AccountingContract.s.sol:SetAdmin --rpc-url $RPC_URL --broadcast
        ;;
    accept-admin)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$PRIVATE_KEY" || -z "$ADMIN_PUBLIC_KEY" ]]; then
            echo "Missing required arguments for setting admin."
            show_help
            exit 1
        fi
        echo "Accepting admin..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Admin Public Key: $ADMIN_PUBLIC_KEY"

        export PRIVATE_KEY=$PRIVATE_KEY
        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS
        export ADMIN_PUBLIC_KEY=$ADMIN_PUBLIC_KEY

        forge script AccountingContract.s.sol:AcceptAdmin --rpc-url $RPC_URL --broadcast
        ;;
    execute-payment)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$PRIVATE_KEY" || -z "$AMOUNT" || -z "$BENEFICIARY_PUBLIC_KEY" ]]; then
            echo "Missing required arguments for executing payment."
            show_help
            exit 1
        fi
        echo "Executing payment..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Payment Amount: $PAYMENT_AMOUNT"

        export PRIVATE_KEY=$PRIVATE_KEY
        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS
        export BENEFICIARY_PUBLIC_KEY=$BENEFICIARY_PUBLIC_KEY
        export AMOUNT=$AMOUNT

        forge script AccountingContract.s.sol:ExecutePayment --rpc-url $RPC_URL --broadcast
        ;;
    is-admin)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$ADMIN_PUBLIC_KEY" ]]; then
            echo "Missing required arguments for checking admin status."
            show_help
            exit 1
        fi
        echo "Checking admin status..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Admin Public Key: $ADMIN_PUBLIC_KEY"

        export ADMIN_PUBLIC_KEY=$ADMIN_PUBLIC_KEY
        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS

        forge script AccountingContract.s.sol:IsAdmin --rpc-url $RPC_URL --broadcast
        ;;
    is-worker)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$WORKER_PUBLIC_KEY" ]]; then
            echo "Missing required arguments for checking worker status."
            show_help
            exit 1
        fi
        echo "Checking worker status..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Worker Public Key: $WORKER_PUBLIC_KEY"

        export WORKER_PUBLIC_KEY=$WORKER_PUBLIC_KEY
        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS

        forge script AccountingContract.s.sol:IsWorker --rpc-url $RPC_URL --broadcast
        ;;
    balance)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" ]]; then
            echo "Missing required arguments for checking balance."
            show_help
            exit 1
        fi
        echo "Checking balance..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"

        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS
        forge script AccountingContract.s.sol:Balance --rpc-url $RPC_URL --broadcast
        ;;
    get-user-record)
        if [[ -z "$CONTRACT_ADDRESS" || -z "$RPC_URL" || -z "$BENEFICIARY_PUBLIC_KEY" ]]; then
            echo "Missing required arguments for getting user record."
            show_help
            exit 1
        fi
        echo "Getting user record..."
        echo "Contract Address: $CONTRACT_ADDRESS"
        echo "RPC URL: $RPC_URL"
        echo "Beneficiary Public Key: $BENEFICIARY_PUBLIC_KEY"

        export CONTRACT_ADDRESS=$CONTRACT_ADDRESS
        export BENEFICIARY_PUBLIC_KEY=$BENEFICIARY_PUBLIC_KEY

        forge script AccountingContract.s.sol:GetUserRecords --rpc-url $RPC_URL --broadcast
        ;;
    *)
        echo "Unknown command: $COMMAND"
        show_help
        exit 1
        ;;
esac
