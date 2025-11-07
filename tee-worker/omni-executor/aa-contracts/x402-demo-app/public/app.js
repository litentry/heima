import { calculateOmniAccount, getRootSignerFromTEE, getCounterfactualAddress, signUserOperation } from './aa-utils.js';
let walletAddress = null;
let smartWalletAddress = null;
let omniAccountHash = null;
let CONFIG = {};
let publicClient = null;
let FACTORY_ABI = null;
let ACCOUNT_ABI = null;
let ENTRYPOINT_ABI = null;

const UserOpSigner = {
    Owner: 0x00,
    RootKey: 0x01,
    SessionKey: 0x02,
    Passkey: 0x03,
};

const OwnerType = {
    Evm: 6,  // 0x06 for EVM addresses
};



async function init() {
    try {
        const response = await fetch('/config');
        CONFIG = await response.json();
        console.log('Loaded config:', CONFIG);

        const factoryAbiResponse = await fetch('/abis/OmniAccountFactory.json');
        const factoryAbiData = await factoryAbiResponse.json();
        FACTORY_ABI = factoryAbiData.abi;

        const accountAbiResponse = await fetch('/abis/OmniAccount.json');
        const accountAbiData = await accountAbiResponse.json();
        ACCOUNT_ABI = accountAbiData.abi;

        const entrypointAbiResponse = await fetch('/EntryPoint.json');
        const entrypointAbiData = await entrypointAbiResponse.json();
        ENTRYPOINT_ABI = entrypointAbiData.abi;

        const { createPublicClient, http } = window.viem;
        publicClient = createPublicClient({
            chain: {
                id: CONFIG.chainId,
                name: 'Arbitrum Sepolia',
                network: 'arbitrum-sepolia',
                nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
                rpcUrls: {
                    default: { http: [CONFIG.rpcUrl] },
                    public: { http: [CONFIG.rpcUrl] },
                }
            },
            transport: http(CONFIG.rpcUrl)
        });

        console.log('Initialized viem client');
    } catch (error) {
        console.error('Initialization failed:', error);
        alert('Failed to initialize the application. Please check the console for errors.');
    }
}

// Load configuration when page loads
window.addEventListener('load', init);

/**
 * Connect to MetaMask wallet
 */
window.connectWallet = async function connectWallet() {
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
            omniAccountHash = await calculateOmniAccount(walletAddress, CONFIG.clientId);
            console.log('OmniAccount hash:', omniAccountHash);

            const rootSigner = await getRootSignerFromTEE(CONFIG.teeWorkerUrl, omniAccountHash);
            const { stringToHex } = window.viem;
            const clientIdBytes = stringToHex(CONFIG.clientId);

            smartWalletAddress = await getCounterfactualAddress(publicClient, CONFIG.factoryAddress, FACTORY_ABI, omniAccountHash, OwnerType.Evm, clientIdBytes, rootSigner);

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
 * Request protected content from x402 endpoint
 */
window.requestProtectedContent = async function requestProtectedContent() {
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
        const signature = await signUserOperation(publicClient, CONFIG.entrypointAddress, ENTRYPOINT_ABI, UserOpSigner, userOp, walletAddress);
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
