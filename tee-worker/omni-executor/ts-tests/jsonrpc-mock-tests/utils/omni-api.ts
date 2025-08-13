import { JsonRpcClient } from './client';
import { JsonRpcMethods } from './methods';
import {
    RequestEmailVerificationCodeParams,
    UserLoginParams,
    GetWeb3SignInMessageParams,
    GetNextIntentIdParams,
    ExportWalletParams,
    SubmitUserOpParams,
    SubmitUserOpTestParams,
    GetSmartWalletRootSignerParams,
    TransferWithdrawParams,
    SubmitSwapOrderParams,
    SignLimitOrderParams,
    NotifyLimitOrderResultParams,
} from './request-types';
import {
    RequestEmailVerificationCodeResponse,
    GetShieldingKeyResponse,
    UserLoginResponse,
    GetWeb3SignInMessageResponse,
    GetNextIntentIdResponse,
    ExportWalletResponse,
    AddWalletResponse,
    SubmitUserOpResponse,
    SubmitUserOpTestResponse,
    GetSmartWalletRootSignerResponse,
    TransferWithdrawResponse,
    SubmitSwapOrderResponse,
    SignLimitOrderResponse,
    NotifyLimitOrderResultResponse,
} from './response-types';

export class OmniApi {
    private static instance: OmniApi;
    private client: JsonRpcClient;

    private constructor() {
        this.client = JsonRpcClient.getInstance();
    }

    static getInstance(): OmniApi {
        if (!OmniApi.instance) {
            OmniApi.instance = new OmniApi();
        }
        return OmniApi.instance;
    }

    async requestEmailVerificationCode(
        params: RequestEmailVerificationCodeParams
    ): Promise<RequestEmailVerificationCodeResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniRequestEmailVerificationCode, params);

        return response;
    }

    async getShieldingKey(): Promise<GetShieldingKeyResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniGetShieldingKey, {});

        return response;
    }

    async getNextIntentId(params: GetNextIntentIdParams): Promise<GetNextIntentIdResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniGetNextIntentId, params);

        return response;
    }

    async getWeb3SignInMessage(params: GetWeb3SignInMessageParams): Promise<GetWeb3SignInMessageResponse> {
        // Pass parameters as array for positional arguments (matching aa-demo-app)
        const positionParams = [params.client_id, params.omni_account];
        const response = await this.client.call(JsonRpcMethods.OmniGetWeb3SignInMessage, positionParams);

        return response;
    }

    async userLogin(params: UserLoginParams): Promise<UserLoginResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniUserLogin, params);

        return response;
    }

    async addWallet(token: string): Promise<AddWalletResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniAddWallet, {}, token);

        return response;
    }

    async exportWallet(params: ExportWalletParams, token: string): Promise<ExportWalletResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniExportWallet, params, token);

        return response;
    }

    async submitSwapOrder(params: SubmitSwapOrderParams, token: string): Promise<SubmitSwapOrderResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniSubmitSwapOrder, params, token);

        return response;
    }

    async signLimitOrder(params: SignLimitOrderParams, token: string): Promise<SignLimitOrderResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniSignLimitOrder, params, token);

        return response;
    }

    async notifyLimitOrderResult(
        params: NotifyLimitOrderResultParams,
        token: string
    ): Promise<NotifyLimitOrderResultResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniNotifyLimitOrderResult, params, token);

        return response;
    }
    async submitUserOp(params: SubmitUserOpParams, token: string): Promise<SubmitUserOpResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniSubmitUserOp, params, token);

        return response;
    }

    async transferWithdraw(params: TransferWithdrawParams, token: string): Promise<TransferWithdrawResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniTransferWithdraw, params, token);

        return response;
    }

    async getSmartWalletRootSigner(
        omniAccount: string,
        chainType: string = 'evm',
        walletIndex: number = 0
    ): Promise<GetSmartWalletRootSignerResponse> {
        const params = {
            omni_account: omniAccount,
            chain_type: chainType,
            wallet_index: walletIndex,
        };
        
        const response = await this.client.call(JsonRpcMethods.OmniGetSmartWalletRootSigner, params);
        return response;
    }

    async submitUserOpTest(params: SubmitUserOpTestParams): Promise<SubmitUserOpTestResponse> {
        const response = await this.client.call(JsonRpcMethods.OmniSubmitUserOpTest, params);

        return response;
    }
}

export const omniApi = OmniApi.getInstance();
