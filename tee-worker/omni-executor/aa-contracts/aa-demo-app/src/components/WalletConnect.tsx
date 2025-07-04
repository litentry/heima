'use client'

import { useState } from 'react'
import { ConnectButton } from '@rainbow-me/rainbowkit'
import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { useAccount } from 'wagmi'

type WalletType = 'evm' | 'solana'

export function WalletConnect() {
  const [activeWallet, setActiveWallet] = useState<WalletType>('evm')
  const { address: evmAddress, isConnected: isEvmConnected } = useAccount()
  const { publicKey: solanaAddress, connected: isSolanaConnected } = useWallet()

  return (
    <div className="w-full max-w-md mx-auto p-6 bg-white rounded-lg shadow-lg">
      <h2 className="text-2xl font-bold mb-6 text-center">Connect Wallet</h2>
      
      {/* Wallet Type Selector */}
      <div className="flex mb-6 bg-gray-100 rounded-lg p-1">
        <button
          onClick={() => setActiveWallet('evm')}
          className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors ${
            activeWallet === 'evm'
              ? 'bg-white text-gray-900 shadow-sm'
              : 'text-gray-500 hover:text-gray-700'
          }`}
        >
          EVM (Ethereum)
        </button>
        <button
          onClick={() => setActiveWallet('solana')}
          className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors ${
            activeWallet === 'solana'
              ? 'bg-white text-gray-900 shadow-sm'
              : 'text-gray-500 hover:text-gray-700'
          }`}
        >
          Solana
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
      </div>
    </div>
  )
}