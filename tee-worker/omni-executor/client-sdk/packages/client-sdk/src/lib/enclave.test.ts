import { enclave } from './enclave';

describe('enclave', () => {
  it('get shielding key', async () => {
    const shieldingKey = await enclave.getShieldingKey();

    expect(shieldingKey).toBeDefined();
  });
});
