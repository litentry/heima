'use client'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { WagmiProvider } from 'wagmi'
import { RainbowKitProvider } from '@rainbow-me/rainbowkit'
import { config } from '@/lib/wagmi'
import { SolanaWalletProvider } from '@/lib/solana'
import { AuthProvider } from '@/contexts/AuthContext'

import '@rainbow-me/rainbowkit/styles.css'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
    },
  },
})

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider>
          <SolanaWalletProvider>
            <AuthProvider>
              {children}
            </AuthProvider>
          </SolanaWalletProvider>
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  )
}