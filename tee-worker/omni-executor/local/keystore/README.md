This folder contains the dev keystore: signer key

How is bin file generated:

```bash
# subkey inspect '//Alice'
echo -n "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a" | xxd -r -p > substrate_alice.bin
```