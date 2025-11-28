This folder contains the dev keystore: signer key

How is bin file generated:

```bash
# subkey inspect '//Alice'
echo -n "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a" | xxd -r -p > substrate_key.bin
```

```bash
cast w n
echo -n "3b42bd2398687699729f168ecb5e29010a5fd2f5b6fc5195dd44eb1c8019715a" | xxd -r -p > accounting_ecdsa_signer_key.bin
```