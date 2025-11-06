// Main application logic
let walletAddress = null;
let smartWalletAddress = null;
let omniAccountHash = null;
let CONFIG = {};

// Load configuration when page loads
window.addEventListener('DOMContentLoaded', async () => {
    const response = await fetch('/config');
    CONFIG = await response.json();
    console.log('Loaded config:', CONFIG);
});

/**
 * Calculate OmniAccount hash from EVM address
 */
async function calculateOmniAccount(address) {
    const encoder = new TextEncoder();
    const clientIdBytes = encoder.encode(CONFIG.clientId || 'wildmeta');
    const identityTypeBytes = encoder.encode('evm');

    // Convert address hex to bytes
    const addressHex = address.slice(2).toLowerCase();
    const addressBytes = new Uint8Array(20);
    for (let i = 0; i < addressHex.length; i += 2) {
        addressBytes[i / 2] = parseInt(addressHex.substring(i, i + 2), 16);
    }

    // Combine all bytes
    const combined = new Uint8Array(
        clientIdBytes.length + identityTypeBytes.length + addressBytes.length
    );
    combined.set(clientIdBytes, 0);
    combined.set(identityTypeBytes, clientIdBytes.length);
    combined.set(addressBytes, clientIdBytes.length + identityTypeBytes.length);

    // Calculate SHA256 hash
    const hashBuffer = await crypto.subtle.digest('SHA-256', combined);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return '0x' + hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

/**
 * Connect to MetaMask wallet
 */
async function connectWallet() {
    try {
        if (!window.ethereum) {
            alert('Please install MetaMask to use this demo');
            return;
        }

        // Request account access
        const accounts = await window.ethereum.request({
            method: 'eth_requestAccounts'
        });

        walletAddress = accounts[0];
        console.log('Connected wallet:', walletAddress);

        // Update UI
        document.getElementById('walletAddress').textContent = walletAddress;
        document.getElementById('smartWalletAddress').textContent = 'Loading...';

        // Show account section
        document.getElementById('connectSection').classList.add('hidden');
        document.getElementById('accountSection').classList.remove('hidden');

        // Switch to Arbitrum Sepolia if needed
        await switchToArbitrumSepolia();

        // Get counterfactual address
        try {
            omniAccountHash = await calculateOmniAccount(walletAddress);
            console.log('OmniAccount hash:', omniAccountHash);

            // Call server to get counterfactual address
            const response = await fetch('/api/getCounterfactualAddress', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ omniAccountHash, walletAddress })
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || 'Failed to get counterfactual address');
            }

            const result = await response.json();
            smartWalletAddress = result.address;

            document.getElementById('smartWalletAddress').textContent = smartWalletAddress;
            document.getElementById('requestSection').classList.remove('hidden');

        } catch (addressError) {
            console.error('Error fetching counterfactual address:', addressError);
            document.getElementById('smartWalletAddress').textContent =
                'Error: ' + addressError.message;
            document.getElementById('requestSection').classList.remove('hidden');
        }

    } catch (error) {
        console.error('Error connecting wallet:', error);
        alert('Failed to connect wallet: ' + error.message);
    }
}

/**
 * Switch to Arbitrum Sepolia network
 */
async function switchToArbitrumSepolia() {
    try {
        await window.ethereum.request({
            method: 'wallet_switchEthereumChain',
            params: [{ chainId: '0x66eee' }], // 421614 in hex
        });
    } catch (switchError) {
        if (switchError.code === 4902) {
            try {
                await window.ethereum.request({
                    method: 'wallet_addEthereumChain',
                    params: [{
                        chainId: '0x66eee',
                        chainName: 'Arbitrum Sepolia',
                        nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
                        rpcUrls: ['https://sepolia-rollup.arbitrum.io/rpc'],
                        blockExplorerUrls: ['https://sepolia.arbiscan.io/']
                    }],
                });
            } catch (addError) {
                console.error('Error adding network:', addError);
            }
        }
    }
}

/**
 * Sign UserOperation with MetaMask using EIP-712
 */
async function signUserOperation(userOp) {
    const domain = {
        name: 'ERC4337',
        version: '1',
        chainId: CONFIG.chainId,
        verifyingContract: CONFIG.entrypointAddress,
    };

    const types = {
        PackedUserOperation: [
            { name: 'sender', type: 'address' },
            { name: 'nonce', type: 'uint256' },
            { name: 'initCode', type: 'bytes' },
            { name: 'callData', type: 'bytes' },
            { name: 'accountGasLimits', type: 'bytes32' },
            { name: 'preVerificationGas', type: 'uint256' },
            { name: 'gasFees', type: 'bytes32' },
            { name: 'paymasterAndData', type: 'bytes' },
        ],
    };

    const message = {
        sender: userOp.sender,
        nonce: userOp.nonce,
        initCode: userOp.init_code,
        callData: userOp.call_data,
        accountGasLimits: userOp.account_gas_limits,
        preVerificationGas: userOp.pre_verification_gas,
        gasFees: userOp.gas_fees,
        paymasterAndData: userOp.paymaster_and_data,
    };

    const signature = await window.ethereum.request({
        method: 'eth_signTypedData_v4',
        params: [walletAddress, JSON.stringify({ domain, types, primaryType: 'PackedUserOperation', message })],
    });

    // Add UserOpSigner.Owner prefix (0x00)
    return '0x00' + signature.slice(2);
}

/**
 * Request protected content from x402 endpoint
 */
async function requestProtectedContent() {
    const statusEl = document.getElementById('status');
    const requestBtn = document.getElementById('requestBtn');

    try {
        requestBtn.disabled = true;
        statusEl.className = 'status loading';
        statusEl.classList.remove('hidden');
        statusEl.innerHTML = 'Step 1/5: Requesting content...';

        // Step 1: Get payment requirements
        const initialResponse = await fetch('/x402');
        if (initialResponse.status !== 402) {
            throw new Error('Expected 402 Payment Required response');
        }

        const paymentInfo = await initialResponse.json();
        console.log('Payment required:', paymentInfo);
        statusEl.innerHTML = `Step 2/5: Payment required: ${paymentInfo.payment.amount} ETH`;

        // Step 2: Build UserOperation (server-side)
        statusEl.innerHTML = 'Step 3/5: Building UserOperation...';

        const amountWei = Math.floor(parseFloat(paymentInfo.payment.amount) * 1e18).toString();

        const buildResponse = await fetch('/api/buildUserOp', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                smartWalletAddress,
                recipient: paymentInfo.payment.recipient,
                amount: amountWei,
                omniAccountHash
            })
        });

        if (!buildResponse.ok) {
            const error = await buildResponse.json();
            throw new Error(error.error || 'Failed to build UserOp');
        }

        const { userOp, isDeployed } = await buildResponse.json();
        console.log('Built UserOp:', userOp, 'isDeployed:', isDeployed);

        // Step 3: Sign the UserOperation
        statusEl.innerHTML = 'Step 4/5: Please sign in MetaMask...';
        const signature = await signUserOperation(userOp);
        userOp.signature = signature;
        console.log('Signed UserOp');

        // Step 4: Submit payment
        statusEl.innerHTML = 'Step 5/5: Submitting payment...';

        const paymentHeader = btoa(JSON.stringify(userOp));
        const paymentResponse = await fetch('/x402', {
            method: 'GET',
            headers: { 'X-Payment': paymentHeader }
        });

        if (!paymentResponse.ok) {
            const errorData = await paymentResponse.json();
            throw new Error(errorData.error || 'Payment failed');
        }

        // Step 5: Show success
        const result = await paymentResponse.json();
        console.log('Payment successful:', result);

        statusEl.className = 'status success';
        statusEl.innerHTML = `
            <strong>✓ Payment Successful!</strong><br><br>
            <strong>Protected Content:</strong> ${result.content.data}<br><br>
            <strong>Timestamp:</strong> ${result.content.timestamp}<br>
            <strong>Amount Paid:</strong> ${result.content.paidAmount} ETH<br>
            <div class="tx-hash">
                <strong>Transaction Hash:</strong><br>
                ${result.transactionHash || 'Pending...'}
            </div>
        `;

        if (result.transactionHash) {
            const explorerUrl = `https://sepolia.arbiscan.io/tx/${result.transactionHash}`;
            statusEl.innerHTML += `<br><a href="${explorerUrl}" target="_blank" style="color: #667eea; text-decoration: underline;">View on Arbiscan</a>`;
        }

    } catch (error) {
        console.error('Error requesting content:', error);
        statusEl.className = 'status error';
        statusEl.innerHTML = `<strong>✗ Error:</strong><br>${error.message}`;
    } finally {
        requestBtn.disabled = false;
    }
}

// Listen for account changes
if (window.ethereum) {
    window.ethereum.on('accountsChanged', (accounts) => {
        if (accounts.length === 0 || accounts[0] !== walletAddress) {
            location.reload();
        }
    });

    window.ethereum.on('chainChanged', () => location.reload());
}
