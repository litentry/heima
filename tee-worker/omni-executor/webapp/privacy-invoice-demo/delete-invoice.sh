#!/bin/bash
# Delete a specific invoice by ID
# Usage: ./delete-invoice.sh inv_xxx

INVOICE_ID="${1:-}"
RPC_URL="${2:-ws://localhost:2100}"

if [ -z "$INVOICE_ID" ]; then
    echo "Usage: $0 <invoice_id> [rpc_url]"
    echo "Example: $0 inv_e1c1a2ed-1057-4d5b-846e-46a9f2f6bfc3"
    exit 1
fi

echo "Deleting invoice: $INVOICE_ID"
echo "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"omni_deleteInvoice\",\"params\":{\"invoice_id\":\"$INVOICE_ID\"}}" | websocat "$RPC_URL"
echo ""
