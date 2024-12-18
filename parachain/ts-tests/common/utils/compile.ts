//@ts-ignore
import solc from 'solc';
const solcWrapper: any = solc;
const source: string = `
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

contract Hello {
    string public message;

    constructor() {
        message = "Hello World";
    }

    function sayMessage() public view returns (string memory) {
        return message;
    }

    function setMessage(string memory newMessage) public {
        message = newMessage;
    }
}
`;

const input = {
    language: 'Solidity',
    sources: {
        'hello.sol': {
            content: source,
        },
    },
    settings: {
        outputSelection: {
            '*': {
                '*': ['*'],
            },
        },
        // evmVersion: "byzantium",
    },
};
const result = JSON.parse(solcWrapper.compile(JSON.stringify(input)));
if (!result.contracts) {
    console.log(result.errors);
}
export const compiled: any = result.contracts['hello.sol']['Hello'];
