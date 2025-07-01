import { JsonRpcClient } from './client';
import {JsonRpcMethods} from './methods';
import { RequestEmailVerificationCodeParams } from './request-types';
import { RequestEmailVerificationCodeResponse } from './response-types';
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

  async requestEmailVerificationCode(params: RequestEmailVerificationCodeParams): Promise<RequestEmailVerificationCodeResponse> {
    const response = await this.client.call(JsonRpcMethods.OmniRequestEmailVerificationCode, params);

    return response;
  }
    
  // todo: add other methods

}

export const omniApi = OmniApi.getInstance(); 