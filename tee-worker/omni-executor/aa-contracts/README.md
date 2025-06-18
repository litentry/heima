# aa-contracts

Contains Solidity code for ERC-4337 Account Abstraction.

This project includes code from https://github.com/eth-infinitism/account-abstraction git commit sha: cc3893bcaf2272c163ce89d5eb9eadb8e6b52db7
licensed under the GNU General Public License v3.0.

## Compiling 

`forge compile`

## Running tests

`forge test`

## Deploying locally

Run `anvil`, then contracts can be deployed using one of the node unlocked accounts, for example:
```
forge create --from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --unlocked --broadcast EntryPoint
[⠊] Compiling...
No files changed, compilation skipped
Deployer: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Deployed to: 0x5FbDB2315678afecb367f032d93F642f64180aa3
Transaction hash: 0x99a0f81df2d5c4a0dbf2ea7b266dccf8cb8f33506aca5ef5c1aae2a64382a8a9
```

Then deploy `SmartAccountFactory` contract providing entrypoint's address as constructor argument:
```
forge create --from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --unlocked --broadcast SmartAccountFactory --constructor-args 0x5FbDB2315678afecb367f032d93F642f64180aa3
[⠊] Compiling...
No files changed, compilation skipped
Deployer: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Deployed to: 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
Transaction hash: 0xb3ef2976e8fd4e51bd0157336322d7c814d998ed33fa126c448aa193c4e247a1
```