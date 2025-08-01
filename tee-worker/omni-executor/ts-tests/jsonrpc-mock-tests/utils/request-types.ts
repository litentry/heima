export interface RequestEmailVerificationCodeParams {
    client_id: string;
    user_email: string;
}

export interface GetNextIntentIdParams {
    omni_account: string;
}

export interface UserLoginParams {
    user_id: {
        type: string;
        value: string;
    };
    user_auth: {
        type: string;
        value: string;
    };
    client_id: string;
    client_auth: {
        type: string;
        value: {
            google_code: string;
            invite_code: string;
        };
    };
}

export interface GetWeb3SignInMessageParams {
    client_id: string;
    omni_account: string;
}

export interface ExportWalletParams {
    key: string;
    google_code: string;
    chain_id: number;
    wallet_index: number;
    wallet_address: string;
}

export interface TransferWithdrawParams {
    request_id?: number;
    chain_id: number;
    wallet_index: number;
    recipient_address: string;
    token_ca: string;
    amount: string;
    google_code: string;
    lang?: string;
}

export interface SubmitSwapOrderParams {
    intent_id: number;
    order_type: 'market' | 'limit';
    swap_type: 1 | 2;
    from_chain_id: number;
    from_token_ca?: string;
    from_amount: string;
    to_chain_id: number;
    to_token_ca?: string;
    double_out: boolean;
    is_one_click: boolean;
    token_cap?: string;
    price_usd?: string;
    usd_worth: string;
    trailing_percent?: number;
    wallet_index: number;
}

export interface SignLimitOrderParams {
    intent_id: number;
    order_id: number;
    chain_id: number;
    wallet_index: number;
    unsigned_tx: string[];
}

export interface NotifyLimitOrderResultParams {
    intent_id: number;
    result: string;
    message?: string;
}
