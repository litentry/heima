#!/bin/bash

# Entrypoint script for omni-executor that loads contract addresses

# Load contract addresses if available
if [ -f "/shared/contract-addresses.env" ]; then
    echo "Loading contract addresses from /shared/contract-addresses.env"
    source /shared/contract-addresses.env
    
    # Export the variables
    export OE_ENTRY_POINT_ADDRESS
    export OE_OMNI_FACTORY_ADDRESS
    export OE_OMNI_WALLET_IMPLEMENTATION_ADDRESS
    export OE_PAYMASTER_ADDRESS
    export OE_TEST_USDC_ADDRESS
    export OE_TEST_USDT_ADDRESS
    
    echo "Contract addresses loaded:"
    echo "  EntryPoint: $OE_ENTRY_POINT_ADDRESS"
    echo "  OmniFactory: $OE_OMNI_FACTORY_ADDRESS"
    echo "  WalletImplementation: $OE_OMNI_WALLET_IMPLEMENTATION_ADDRESS"
    echo "  Paymaster: $OE_PAYMASTER_ADDRESS"
    echo "  Test USDC: $OE_TEST_USDC_ADDRESS"
    echo "  Test USDT: $OE_TEST_USDT_ADDRESS"
else
    echo "No contract addresses file found at /shared/contract-addresses.env"
    echo "Using default addresses"
fi

# Execute the original command
exec "$@"