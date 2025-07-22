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
