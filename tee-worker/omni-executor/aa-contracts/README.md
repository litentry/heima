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
```forge create --from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --unlocked --broadcast EntryPoint```