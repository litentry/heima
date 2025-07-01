export interface BaseResponse {
  jsonrpc: '2.0';
  id: string | number;
}

export interface RequestEmailVerificationCodeResponse extends BaseResponse {
  result: null;
}

// todo: add other types