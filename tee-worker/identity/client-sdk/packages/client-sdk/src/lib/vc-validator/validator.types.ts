/**
 * Defines the details of the validation result for each component of the Verifiable Credential (VC).
 */
export type ValidationResultDetail = {
  /**
   * Represents the validation result for the VC signature.
   *
   * If validation succeeds, it's true; if validation fails, it's an error message.
   *
   * Use issuer.id in VC JSON as vcPubkey, proof.proofValue in VC JSON as signature to verify VC JSON.
   */
  vcSignature?: true | string;
  /**
   * Represents the validation result (vcPubkey and mrEnclave) for the Enclave registry.
   *
   * If validation succeeds, it's true; if validation fails, it's an error message.
   *
   * The vcPubkey from Enclave registry must be same as issuer.id in VC JSON.
   *
   * The mrEnclave from Enclave registry must be same as issuer.mrenclave in VC JSON.
   */
  enclaveRegistry?: true | string;
};

/**
 * Represents the overall validation result for a Verifiable Credential (VC).
 */
export type ValidationResult = {
  /**
   * Represents the whole validation result status.
   *
   * If is true, means all fields of the detail are true, otherwise any one of it is not true.
   *
   * The caller should use this field to determine whether the VC is valid.
   */
  isValid: boolean;
  /**
   * Represents the whole validation result detail.
   */
  detail: ValidationResultDetail;
};
