import { useState } from 'react'
import { ConnectButton } from '@rainbow-me/rainbowkit'
import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { useAccount } from 'wagmi'
import { Mail, AlertCircle } from 'lucide-react'
import { useAuth } from '@/contexts/AuthContext'
import { isValidEmail } from '@/lib/aa-utils'

type WalletType = 'evm' | 'solana' | 'email'

export function WalletConnect() {
    const [activeWallet, setActiveWallet] = useState<WalletType>('evm')
    const [emailInput, setEmailInput] = useState('')
    const [emailError, setEmailError] = useState('')
    const [isProcessing, setIsProcessing] = useState(false)

    const { address: evmAddress, isConnected: isEvmConnected } = useAccount()
    const { publicKey: solanaAddress, connected: isSolanaConnected } = useWallet()
    const { authType, identifier, setEmailAuth, clearAuth } = useAuth()

    return (
        <div className="w-full max-w-md mx-auto p-6 bg-white rounded-lg shadow-lg">
            <h2 className="text-2xl font-bold mb-6 text-center">Connect Account</h2>

            {/* Wallet Type Selector */}
            <div className="flex mb-6 bg-gray-100 rounded-lg p-1">
                <button
                    onClick={() => setActiveWallet('evm')}
                    className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
                        activeWallet === 'evm'
                            ? 'bg-white text-gray-900 shadow-sm'
                            : 'text-gray-500 hover:text-gray-700'
                    }`}
                >
                    EVM
                </button>
                <button
                    onClick={() => setActiveWallet('solana')}
                    className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
                        activeWallet === 'solana'
                            ? 'bg-white text-gray-900 shadow-sm'
                            : 'text-gray-500 hover:text-gray-700'
                    }`}
                >
                    Solana
                </button>
                <button
                    onClick={() => setActiveWallet('email')}
                    className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
                        activeWallet === 'email'
                            ? 'bg-white text-gray-900 shadow-sm'
                            : 'text-gray-500 hover:text-gray-700'
                    }`}
                >
                    <div className="flex items-center justify-center gap-1">
                        <Mail className="w-4 h-4" />
                        Email
                    </div>
                </button>
            </div>

            {/* Wallet Connection */}
            <div className="space-y-4">
                {activeWallet === 'evm' && (
                    <div>
                        <h3 className="text-lg font-semibold mb-3">EVM Wallet</h3>
                        <ConnectButton.Custom>
                            {({
                                account,
                                chain,
                                openAccountModal,
                                openChainModal,
                                openConnectModal,
                                authenticationStatus,
                                mounted,
                            }) => {
                                const ready = mounted && authenticationStatus !== 'loading'
                                const connected =
                                    ready &&
                                    account &&
                                    chain &&
                                    (!authenticationStatus || authenticationStatus === 'authenticated')

                                return (
                                    <div
                                        {...(!ready && {
                                            'aria-hidden': true,
                                            style: {
                                                opacity: 0,
                                                pointerEvents: 'none',
                                                userSelect: 'none',
                                            },
                                        })}
                                    >
                                        {(() => {
                                            if (!connected) {
                                                return (
                                                    <button
                                                        onClick={openConnectModal}
                                                        className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
                                                    >
                                                        Connect EVM Wallet
                                                    </button>
                                                )
                                            }

                                            if (chain.unsupported) {
                                                return (
                                                    <button
                                                        onClick={openChainModal}
                                                        className="w-full bg-red-600 hover:bg-red-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
                                                    >
                                                        Wrong network
                                                    </button>
                                                )
                                            }

                                            return (
                                                <div className="flex gap-2">
                                                    <button
                                                        onClick={openChainModal}
                                                        className="flex-1 bg-gray-100 hover:bg-gray-200 text-gray-800 font-medium py-3 px-4 rounded-lg transition-colors"
                                                    >
                                                        {chain.hasIcon && (
                                                            <div className="w-4 h-4 mr-2 inline-block">
                                                                {chain.iconUrl && (
                                                                    <img
                                                                        alt={chain.name ?? 'Chain icon'}
                                                                        src={chain.iconUrl}
                                                                        className="w-4 h-4"
                                                                    />
                                                                )}
                                                            </div>
                                                        )}
                                                        {chain.name}
                                                    </button>

                                                    <button
                                                        onClick={openAccountModal}
                                                        className="flex-1 bg-gray-100 hover:bg-gray-200 text-gray-800 font-medium py-3 px-4 rounded-lg transition-colors"
                                                    >
                                                        {account.displayName}
                                                        {account.displayBalance
                                                            ? ` (${account.displayBalance})`
                                                            : ''}
                                                    </button>
                                                </div>
                                            )
                                        })()}
                                    </div>
                                )
                            }}
                        </ConnectButton.Custom>

                        {isEvmConnected && evmAddress && (
                            <div className="mt-4 p-3 bg-green-50 border border-green-200 rounded-lg">
                                <p className="text-sm text-green-700">
                                    <span className="font-medium">Connected:</span> {evmAddress}
                                </p>
                            </div>
                        )}
                    </div>
                )}

                {activeWallet === 'email' && (
                    <div>
                        <h3 className="text-lg font-semibold mb-3">Email Account</h3>

                        {authType === 'email' && identifier ? (
                            <div className="space-y-3">
                                <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                                    <p className="text-sm text-green-800">
                                        Connected with email:
                                    </p>
                                    <p className="font-medium text-green-900 mt-1">
                                        {identifier}
                                    </p>
                                </div>
                                <button
                                    onClick={() => {
                                        clearAuth();
                                        setEmailInput('');
                                        setEmailError('');
                                    }}
                                    className="w-full bg-gray-100 hover:bg-gray-200 text-gray-800 font-medium py-3 px-4 rounded-lg transition-colors"
                                >
                                    Use Different Email
                                </button>
                            </div>
                        ) : (
                            <div className="space-y-3">
                                <div>
                                    <input
                                        type="email"
                                        placeholder="Enter your email address"
                                        value={emailInput}
                                        onChange={(e) => {
                                            setEmailInput(e.target.value);
                                            setEmailError('');
                                        }}
                                        className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                        disabled={isProcessing}
                                    />
                                    {emailError && (
                                        <div className="mt-2 flex items-center gap-2 text-red-600">
                                            <AlertCircle className="w-4 h-4" />
                                            <p className="text-sm">{emailError}</p>
                                        </div>
                                    )}
                                </div>

                                <button
                                    onClick={async () => {
                                        const trimmedEmail = emailInput.trim();

                                        if (!trimmedEmail) {
                                            setEmailError('Please enter an email address');
                                            return;
                                        }

                                        if (!isValidEmail(trimmedEmail)) {
                                            setEmailError('Please enter a valid email address');
                                            return;
                                        }

                                        setIsProcessing(true);
                                        setEmailError('');

                                        try {
                                            await setEmailAuth(trimmedEmail);
                                        } catch (error) {
                                            setEmailError('Failed to connect with email. Please try again.');
                                            console.error('Email auth error:', error);
                                        } finally {
                                            setIsProcessing(false);
                                        }
                                    }}
                                    disabled={isProcessing || !emailInput.trim()}
                                    className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 text-white font-medium py-3 px-4 rounded-lg transition-colors"
                                >
                                    {isProcessing ? 'Processing...' : 'Continue with Email'}
                                </button>

                                <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
                                    <p className="text-xs text-blue-800">
                                        <strong>Note:</strong> Email accounts use the TEE worker as a trusted signer. No verification code is needed for transactions.
                                    </p>
                                </div>
                            </div>
                        )}
                    </div>
                )}

                {activeWallet === 'solana' && (
                    <div>
                        <h3 className="text-lg font-semibold mb-3">Solana Wallet</h3>
                        <WalletMultiButton className="!w-full !bg-purple-600 hover:!bg-purple-700 !text-white !font-medium !py-3 !px-4 !rounded-lg !transition-colors" />

                        {isSolanaConnected && solanaAddress && (
                            <div className="mt-4 p-3 bg-green-50 border border-green-200 rounded-lg">
                                <p className="text-sm text-green-700">
                                    <span className="font-medium">Connected:</span> {solanaAddress.toString()}
                                </p>
                            </div>
                        )}
                    </div>
                )}
            </div>

            {/* Connection Status */}
            <div className="mt-6 pt-4 border-t border-gray-200">
                <div className="flex justify-between text-sm">
                    <span className="text-gray-600">EVM:</span>
                    <span className={isEvmConnected ? 'text-green-600' : 'text-gray-400'}>
                        {isEvmConnected ? 'Connected' : 'Not connected'}
                    </span>
                </div>
                <div className="flex justify-between text-sm mt-1">
                    <span className="text-gray-600">Solana:</span>
                    <span className={isSolanaConnected ? 'text-green-600' : 'text-gray-400'}>
                        {isSolanaConnected ? 'Connected' : 'Not connected'}
                    </span>
                </div>
                <div className="flex justify-between text-sm mt-1">
                    <span className="text-gray-600">Email:</span>
                    <span className={authType === 'email' && identifier ? 'text-green-600' : 'text-gray-400'}>
                        {authType === 'email' && identifier ? 'Connected' : 'Not connected'}
                    </span>
                </div>
            </div>
        </div>
    )
}