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
