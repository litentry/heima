export interface BaseResponse {
    jsonrpc: '2.0';
    id: string | number;
}

export type RequestEmailVerificationCodeResponse = null;

export type GetNextIntentIdResponse = number;

export interface GetShieldingKeyResponse {
    n: string;
    e: string;
}

export interface UserLoginResponse {
    access_token: string;
    id_token: string;
    backend_response: {
        code: number;
        message: string;
        data: {
            user_id: string | null;
        };
    };
}

export interface GetWeb3SignInMessageResponse {
    client_id: string;
    omni_account: string;
    message_code: string;
}

export interface AddWalletResponse {
    backend_response: {
        code: number; // 10000 for success
        message: string; // "OK" for success
        data: {}; // Additional data (usually empty for add wallet)
    };
}

export interface ExportWalletResponse {
    ciphertext: string;
    aad: string;
    nonce: string;
}

export interface SubmitUserOpResponse {
    user_op_hash: string;
}

export type GetSmartWalletRootSignerResponse = string;
export interface TransferWithdrawResponse {
    backend_response: {
        code: number;
        message: string;
        data: {
            tx_hash: string;
            transfer_id: string;
            chain_id: number;
        };
    };
}

export interface SubmitSwapOrderResponse {
    backend_response: {
        limit_order_response?: {
            code: number;
            message: string;
            data: {
                order_id?: number;
            };
        };
        market_order_response?: {
            code: number;
            message: string;
            data: {
                tx_hash?: string[];
            };
        };
    };
}

export interface SignLimitOrderResponse {
    intent_id: number;
    order_id: number;
    chain_id: number;
    signed_tx: string[];
}

export type NotifyLimitOrderResultResponse = null;

export interface SubmitUserOpTestResponse {
    transaction_hash: string | null;
}

export interface GetHyperliquidSignatureDataResponse {
    main_address: string;
    hyperliquid_signature_data: {
        action: {
            type: 'approve_agent' | 'withdraw3' | 'approve_builder_fee';
            signatureChainId?: string;
            hyperliquidChain?: string;
            agentAddress?: string;
            agentName?: string;
            nonce?: number;
            amount?: string;
            time?: number;
            destination?: string;
            maxFeeRate?: string;
            builder?: string;
        };
        nonce: number;
        signature: string;
    };
}

// todo: add other types
