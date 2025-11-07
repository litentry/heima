import { calculateOmniAccount, getRootSignerFromTEE, getCounterfactualAddress, signUserOperation } from './aa-utils.js';
let walletAddress = null;
let smartWalletAddress = null;
let omniAccountHash = null;
let CONFIG = {};
let publicClient = null;
let FACTORY_ABI = null;
let ACCOUNT_ABI = null;

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
    console.log('🚀 Initializing X402 Demo App...');
    try {
        // Load configuration
        console.log('📡 Fetching config...');
        const response = await fetch('/config');
        if (!response.ok) throw new Error(`Failed to fetch config: ${response.status}`);
        CONFIG = await response.json();
        console.log('✓ Loaded config:', CONFIG);

        // Load Factory ABI
        console.log('📄 Loading Factory ABI...');
        const factoryAbiResponse = await fetch('/abis/OmniAccountFactory.json');
        if (!factoryAbiResponse.ok) throw new Error(`Failed to fetch Factory ABI: ${factoryAbiResponse.status}`);
        const factoryAbiData = await factoryAbiResponse.json();
        FACTORY_ABI = factoryAbiData.abi;

        // Load Account ABI
        console.log('📄 Loading Account ABI...');
        const accountAbiResponse = await fetch('/abis/OmniAccount.json');
        if (!accountAbiResponse.ok) throw new Error(`Failed to fetch Account ABI: ${accountAbiResponse.status}`);
        const accountAbiData = await accountAbiResponse.json();
        ACCOUNT_ABI = accountAbiData.abi;

        console.log('✓ Loaded ABIs successfully');

        // Initialize viem client
        console.log('🔗 Initializing viem client...');
        if (!window.viem) {
            throw new Error('Viem library not loaded. Check your internet connection.');
        }
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

        console.log('✓ Initialized viem client');
        console.log('✅ Application ready!');
        console.log('👉 Click "Connect MetaMask" to get started');
    } catch (error) {
        console.error('❌ Initialization failed:', error);
        const errorMessage = error.message || 'Unknown error';
        alert(`Failed to initialize the application.\n\nError: ${errorMessage}\n\nPlease check:\n1. Server is running on the correct port\n2. Configuration is correct\n3. Console for detailed errors`);
    }
}

// Load configuration when page loads
window.addEventListener('load', init);

/**
 * Connect to MetaMask wallet
 */
window.connectWallet = async function connectWallet() {
    console.log('🔐 Connecting wallet...');
    try {
        if (!window.ethereum) {
            alert('Please install MetaMask to use this demo\n\nVisit: https://metamask.io');
            return;
        }

        // Request account access
        const accounts = await window.ethereum.request({
            method: 'eth_requestAccounts'
        });

        walletAddress = accounts[0];
        console.log('✓ Connected wallet:', walletAddress);

        // Update UI
        document.getElementById('walletAddress').textContent = walletAddress;
        document.getElementById('smartWalletAddress').textContent = 'Loading...';

        // Show account section
        document.getElementById('connectSection').classList.add('hidden');
        document.getElementById('accountSection').classList.remove('hidden');

        // Switch to Arbitrum Sepolia if needed
        await switchToArbitrumSepolia();

        // Get counterfactual smart wallet address
        console.log('🔍 Calculating smart wallet address...');
        try {
            omniAccountHash = await calculateOmniAccount(walletAddress, CONFIG.clientId);
            console.log('✓ OmniAccount hash:', omniAccountHash);

            console.log('📡 Fetching root signer from TEE worker...');
            const rootSigner = await getRootSignerFromTEE(CONFIG.teeWorkerUrl, omniAccountHash);
            console.log('✓ Root signer:', rootSigner);

            const { stringToHex } = window.viem;
            const clientIdBytes = stringToHex(CONFIG.clientId);

            console.log('🏭 Getting counterfactual address from factory...');
            smartWalletAddress = await getCounterfactualAddress(publicClient, CONFIG.factoryAddress, FACTORY_ABI, omniAccountHash, OwnerType.Evm, clientIdBytes, rootSigner);
            console.log('✅ Smart wallet address:', smartWalletAddress);

            document.getElementById('smartWalletAddress').textContent = smartWalletAddress;
            document.getElementById('requestSection').classList.remove('hidden');

        } catch (addressError) {
            console.error('❌ Error fetching counterfactual address:', addressError);
            document.getElementById('smartWalletAddress').textContent =
                'Error: ' + addressError.message;
            document.getElementById('requestSection').classList.remove('hidden');
        }

    } catch (error) {
        console.error('❌ Error connecting wallet:', error);
        alert(`Failed to connect wallet\n\nError: ${error.message}\n\nPlease check:\n1. MetaMask is installed and unlocked\n2. You approved the connection request`);
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
    console.log('\n💰 === Starting X402 Payment Flow ===\n');
    const statusEl = document.getElementById('status');
    const requestBtn = document.getElementById('requestBtn');

    try {
        requestBtn.disabled = true;
        statusEl.className = 'status loading';
        statusEl.classList.remove('hidden');
        statusEl.innerHTML = 'Step 1/5: Requesting content...';

        // Step 1: Get payment requirements
        console.log('📡 Step 1: Requesting protected content (expect 402)...');
        const initialResponse = await fetch('/x402');
        if (initialResponse.status !== 402) {
            throw new Error(`Expected 402 Payment Required, got ${initialResponse.status}`);
        }

        const paymentInfo = await initialResponse.json();
        console.log('✓ Payment required:', paymentInfo);
        statusEl.innerHTML = `Step 2/5: Payment required: ${paymentInfo.payment.amount} ETH`;

        // Step 2: Build UserOperation (server-side)
        statusEl.innerHTML = 'Step 3/5: Building UserOperation...';
        console.log('🔨 Step 2: Building UserOperation...');

        const amountWei = Math.floor(parseFloat(paymentInfo.payment.amount) * 1e18).toString();
        console.log(`  Amount: ${paymentInfo.payment.amount} ETH = ${amountWei} wei`);

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
        console.log('✓ Built UserOp:', userOp);
        console.log(`  Smart wallet deployed: ${isDeployed}`);
        console.log('UserOp field types:', {
            nonce: typeof userOp.nonce,
            account_gas_limits: typeof userOp.account_gas_limits,
            pre_verification_gas: typeof userOp.pre_verification_gas,
            gas_fees: typeof userOp.gas_fees,
        });

        // Step 3: Sign the UserOperation
        statusEl.innerHTML = 'Step 4/5: Please sign in MetaMask...';
        console.log('✍️  Step 3: Signing UserOperation with MetaMask...');
        console.log('  Using wallet address:', walletAddress);
        console.log('  Expected signer:', walletAddress);
        const signature = await signUserOperation(publicClient, CONFIG.entrypointAddress, UserOpSigner, userOp, walletAddress);
        userOp.signature = signature;
        console.log('✓ Signed UserOp with signature:', signature.substring(0, 20) + '...');

        // Step 4: Submit payment
        statusEl.innerHTML = 'Step 5/5: Submitting payment...';
        console.log('📤 Step 4: Submitting payment to server...');

        const paymentHeader = btoa(JSON.stringify(userOp));
        const paymentResponse = await fetch('/x402', {
            method: 'GET',
            headers: { 'X-Payment': paymentHeader }
        });

        if (!paymentResponse.ok) {
            const errorData = await paymentResponse.json();
            console.error('❌ Payment failed:', errorData);
            throw new Error(errorData.error || 'Payment failed');
        }

        // Step 5: Show success
        const result = await paymentResponse.json();
        console.log('✅ Payment successful!', result);
        console.log('🎉 Transaction hash:', result.transactionHash);

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
        console.error('❌ Error requesting content:', error);
        console.error('Error stack:', error.stack);
        statusEl.className = 'status error';
        statusEl.innerHTML = `<strong>✗ Error:</strong><br>${error.message}<br><br><small>Check console for details</small>`;
    } finally {
        requestBtn.disabled = false;
        console.log('\n=== X402 Payment Flow Complete ===\n');
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
