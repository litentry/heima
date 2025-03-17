import { Base64 } from 'js-base64';
const bs58 = require('bs58');

// We can get the vc data here. For details, please refer to the request vc step of vc.test.ts
/* await requestVCs(
    context,
    context.defaultSigner[0],
    aesKey,
    true,
    context.mrEnclave,
    assertion）
) */
const vc = {
    '@context': ['https://www.w3.org/2018/credentials/v1', 'https://w3id.org/security/suites/ed25519-2020/v1'],
    id: '0x5dc6e20ae1cde237390c54484ceeb65d937672b6df384c587f5aea7a4f0e2c75',
    type: ['VerifiableCredential'],
    credentialSubject: {
        id: 'd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d',
        description: 'User has over X number of transactions',
        type: 'Total EVM and Substrate Transactions',
        tag: ['IDHub'],
        assertions: [
            {
                and: [
                    {
                        src: '$total_txs',
                        op: '>',
                        dst: '0',
                    },
                    {
                        src: '$total_txs',
                        op: '<=',
                        dst: '1',
                    },
                    {
                        or: [
                            {
                                src: '$network',
                                op: '==',
                                dst: 'litentry',
                            },
                        ],
                    },
                ],
            },
        ],
        values: [true],
        endpoint: 'https://www.heima.network/parachain/extrinsic',
    },
    issuer: {
        id: '21e2bf9b4637cee6be6fc3f68412212c6bdf47f895edbb4f44b937b7bb1d6a64',
        name: 'Litentry TEE Worker',
        mrenclave: '2X19fA2iU37raQSSvvducKxXZFZ3tbFSWYUZUTCQtWP4',
    },
    issuanceBlockNumber: 21,
    proof: {
        createdBlockNumber: 21,
        type: 'Ed25519Signature2020',
        proofPurpose: 'assertionMethod',
        proofValue:
            'da6bb5c68766f1718afb4755e81edc92b0fe2bef7b39c268a0ae164aefe46927a77ea3261d4dd97873339bd311a86b5856af059de61e2f2e816a2374dfb81b0a',
        verificationMethod: '21e2bf9b4637cee6be6fc3f68412212c6bdf47f895edbb4f44b937b7bb1d6a64',
    },
};

// You can also get data from here. For details, please refer to the request vc step of vc.test.ts
/** 
    const registry = (await context.api.query.vcManagement.vcRegistry(res[k].index)) as any;
 * 
*/
const enclaveRegistry = {
    mrenclave: '0x168b47aceff04e8cd20f4606a7eb255ffc1981cd3b8ba1d44face858f9a4c25b',
    lastSeen: '1,677,164,874,078',
    url: 'wss://localhost:2000',
    shieldingPubkey:
        '{n:[189,64,222,165,185,105,241,193,170,87,19,231,76,95,247,110,231,7,196,65,135,231,55,75,60,58,158,23,77,230,154,23,203,167,163,219,18,113,83,23,172,131,29,222,200,73,217,159,155,120,217,194,74,33,79,99,88,227,2,242,225,141,116,231,89,68,119,109,183,56,135,70,151,177,245,199,196,222,193,33,28,47,252,83,240,120,238,81,99,154,219,75,84,108,96,199,108,42,64,70,217,164,89,81,156,188,168,181,169,228,21,140,90,18,126,77,50,31,19,149,26,86,160,108,197,78,134,19,54,25,89,80,239,106,95,226,42,109,202,54,158,128,224,243,1,197,209,131,48,1,208,207,48,197,66,44,203,76,113,150,100,73,81,17,94,153,217,11,14,193,230,43,207,24,236,200,207,15,63,16,75,173,191,245,127,191,186,18,111,111,90,24,177,167,177,7,61,94,60,161,130,242,31,210,158,152,31,35,202,80,179,138,219,244,139,19,60,134,108,94,151,228,224,22,29,139,21,241,71,221,65,145,210,108,80,0,76,137,98,128,107,224,16,32,135,232,168,150,9,225,30,120,17,176,26,2,8,100,185,121,158,67,89,110,130,126,122,113,248,169,73,27,52,90,109,66,249,255,161,105,174,129,163,7,14,180,63,178,218,81,86,108,116,118,81,185,248,231,84,150,13,140,49,239,103,44,119,97,37,30,13,230,100,73,24,229,178,3,89,14,26,155,245,254,12,152,7,72,209,209,24,224,5,131,144,124,254,204,209,138,57,196,176,244,231,185,190,187,118,215,46,45,57,81,238,163,11,152,73,217,252,9,77,95,86,4,201,34,68,88,235,103,15,120,159,134,5,182,83,122,128,111,160,141],e:[1,0,0,1]}',
    vcPubkey: '0xde17d6daedb66ec9f5e096cc0317bd6cbf881c0d8273e54105ee7c22a2e48648',
    sgxMode: 'Debug',
    sgxMetadata: {
        quote: 'eyJpZCI6IjE0Mjk5ODY4MDk0NzI3NDMxNzkyNjc5NzczNTE3NDIxNjE5MjA0MSIsInRpbWVzdGFtcCI6IjIwMjMtMDItMjNUMTQ6NTM6NTYuNjY3ODAwIiwidmVyc2lvbiI6NCwiYWR2aXNvcnlVUkwiOiJodHRwczovL3NlY3VyaXR5LWNlbnRlci5pbnRlbC5jb20iLCJhZHZpc29yeUlEcyI6WyJJTlRFTC1TQS0wMDE2MSIsIklOVEVMLVNBLTAwMjE5IiwiSU5URUwtU0EtMDAyODkiLCJJTlRFTC1TQS0wMDMzNCIsIklOVEVMLVNBLTAwNjE1Il0sImlzdkVuY2xhdmVRdW90ZVN0YXR1cyI6IkNPTkZJR1VSQVRJT05fQU5EX1NXX0hBUkRFTklOR19ORUVERUQiLCJwbGF0Zm9ybUluZm9CbG9iIjoiMTUwMjAwNjUwMDAwMDgwMDAwMTMxMzAyMDQwMTAxMDcwMDAwMDAwMDAwMDAwMDAwMDAwRDAwMDAwQzAwMDAwMDAyMDAwMDAwMDAwMDAwMEM1ODZGN0Y2NzlDMzY1N0ZBQThDRjM3OUJGRjQ0MjUxMzI0ODg2MTBBMkQ1MUZGRENDNTgyQTVDRjQwRTcxNjM1MDU4MzJBNDcyOTk5N0NFODRDRTc3NkUzRDkzQTgwQzBENEJBQTdCNTUzOTdFRjk4MjkxNkQyODlEQUQwQzQ3QzI0IiwiaXN2RW5jbGF2ZVF1b3RlQm9keSI6IkFnQUFBRmdNQUFBTkFBMEFBQUFBQU54bVJSSmtrcHp5ZGE0cTdLUWhBeW55eWo0QzdYT0N0aFFiNnV3Y3JoVVpFeFAvL3dFQ0FBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQndBQUFBQUFBQUFIQUFBQUFBQUFBRE9OUDZKTkIybmY4UHUzaEpTeVMwNjJxYUUzQVhwczBNUHBCQktBT0NkK0FBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUNEMXhubmZlcktGSEQydXZZcVRYZERBOGlaMjJrQ0Q1eHc3aDM4Q01mT25nQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFCbzhPbFlUa2JSaFJMRllXMUZCRWdQdmhvNmRVVkdrQUVUVGlwclhoZDBqd0FBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQSJ9',
        quoteSig:
            '0x98130ec1410a857be38eb2d1855f5c7aad7ca30955825793c10dd95a41eeaece1a07c09e1b923aef38b3ee73c2567534fdf30d9bd9ecbc765c3883c154780807cb34d88f374ac855462e788cec60e002e0b3aef80b8cf577edbd70b6127fa0fd7bd203fe35a5f81ec11df83bc382b1476429e070ea97ae9e9186b808158b1299e116e36c4c6ac1812c5ee3a606252cc9c7c56c13d6c1af47522304faed75806bf19de3e4ea8fcbf86ac3b37dabc6a7d0b652898c7c56fe9b24c03244bebc4baf0c9bfffc249f431bb564d994841e06c445469a95a80739b48fd4e14190d44fc67fd917ae1cc5c1999906c5ae5faf89b0521b4c3cc8a5638a251031afd72dffef',
        quoteCert:
            '0x308204a130820309a003020102020900d107765d32a3b096300d06092a864886f70d01010b0500307e310b3009060355040613025553310b300906035504080c0243413114301206035504070c0b53616e746120436c617261311a3018060355040a0c11496e74656c20436f72706f726174696f6e3130302e06035504030c27496e74656c20534758204174746573746174696f6e205265706f7274205369676e696e67204341301e170d3136313132323039333635385a170d3236313132303039333635385a307b310b3009060355040613025553310b300906035504080c0243413114301206035504070c0b53616e746120436c617261311a3018060355040a0c11496e74656c20436f72706f726174696f6e312d302b06035504030c24496e74656c20534758204174746573746174696f6e205265706f7274205369676e696e6730820122300d06092a864886f70d01010105000382010f003082010a0282010100a97a2de0e66ea6147c9ee745ac0162686c7192099afc4b3f040fad6de093511d74e802f510d716038157dcaf84f4104bd3fed7e6b8f99c8817fd1ff5b9b864296c3d81fa8f1b729e02d21d72ffee4ced725efe74bea68fbc4d4244286fcdd4bf64406a439a15bcb4cf67754489c423972b4a80df5c2e7c5bc2dbaf2d42bb7b244f7c95bf92c75d3b33fc5410678a89589d1083da3acc459f2704cd99598c275e7c1878e00757e5bdb4e840226c11c0a17ff79c80b15c1ddb5af21cc2417061fbd2a2da819ed3b72b7efaa3bfebe2805c9b8ac19aa346512d484cfc81941e15f55881cc127e8f7aa12300cd5afb5742fa1d20cb467a5beb1c666cf76a368978b50203010001a381a43081a1301f0603551d2304183016801478437b76a67ebcd0af7e4237eb357c3b8701513c300e0603551d0f0101ff0404030206c0300c0603551d130101ff0402300030600603551d1f045930573055a053a051864f687474703a2f2f7472757374656473657276696365732e696e74656c2e636f6d2f636f6e74656e742f43524c2f5347582f4174746573746174696f6e5265706f72745369676e696e6743412e63726c300d06092a864886f70d01010b050003820181006708b61b5c2bd215473e2b46af99284fbb939d3f3b152c996f1a6af3b329bd220b1d3b610f6bce2e6753bded304db21912f385256216cfcba456bd96940be892f5690c260d1ef84f1606040222e5fe08e5326808212a447cfdd64a46e94bf29f6b4b9a721d25b3c4e2f62f58baed5d77c505248f0f801f9fbfb7fd752080095cee80938b339f6dbb4e165600e20e4a718812d49d9901e310a9b51d66c79909c6996599fae6d76a79ef145d9943bf1d3e35d3b42d1fb9a45cbe8ee334c166eee7d32fcdc9935db8ec8bb1d8eb3779dd8ab92b6e387f0147450f1e381d08581fb83df33b15e000a59be57ea94a3a52dc64bdaec959b3464c91e725bbdaea3d99e857e380a23c9d9fb1ef58e9e42d71f12130f9261d7234d6c37e2b03dba40dfdfb13ac4ad8e13fd3756356b6b50015a3ec9580b815d87c2cef715cd28df00bbf2a3c403ebf6691b3f05edd9143803ca085cff57e053eec2f8fea46ea778a68c9be885bc28225bc5f309be4a2b74d3a03945319dd3c7122fed6ff53bb8b8cb3a03c',
    },
};

/**
 * The process of check issuer attestation.
 */
function checkIssuerAttestation() {
    // decode mrenclave from vc object
    const mrEnclaveFromVc = '0x' + Buffer.from(bs58.decode(vc.issuer.mrenclave)).toString('hex');
    console.log('   [IssuerAttestation] mrEnclaveFromVc: ', mrEnclaveFromVc);

    // Fetch mrEnclave from parachain
    const mrEnclaveFromParachain = enclaveRegistry.mrenclave;
    console.log('   [IssuerAttestation] mrEnclaveFromParachain: ', mrEnclaveFromParachain);

    // >>>0. Check mrEnclave
    if (mrEnclaveFromVc !== mrEnclaveFromParachain) {
        console.log('   [IssuerAttestation] mrEnclave must be equal!');
        return;
    }

    const sgxMetadata = enclaveRegistry.sgxMetadata;
    if (sgxMetadata != null) {
        const quoteFromData = sgxMetadata.quote;
        console.log('   [IssuerAttestation] quoteFromData: ', quoteFromData);
        const quote = JSON.parse(Base64.decode(quoteFromData));

        // >>>1. Verify quote status (mandatory field)
        const status = quote!['isvEnclaveQuoteStatus'];
        console.log('   [IssuerAttestation] ISV Enclave Quote Status: ', status);
        switch (status) {
            case 'OK':
                console.log('OK!');
                break;
            case 'GROUP_OUT_OF_DATE':
            case 'GROUP_REVOKED':
            case 'CONFIGURATION_NEEDED': {
                // Verify platformInfoBlob for further info if status not OK
                if (quote!['platformInfoBlob'] !== undefined) {
                    return;
                } else {
                    console.log('   [IssuerAttestation] Failed to fetch platformInfoBlob from attestation report');
                }

                break;
            }
            default: {
                console.log('   [IssuerAttestation] Unexpected status in attestation report: ', status);
                break;
            }
        }

        // >>>2. Verify quote body
        const quoteBody = quote!['isvEnclaveQuoteBody'];
        if (quoteBody !== undefined) {
            console.log('   [IssuerAttestation] sgxQuote: ', quoteBody);
        } else {
            console.log('   [IssuerAttestation] Failed to fetch isvEnclaveQuoteBody from attestation report');
        }

        // >>>3. Check timestamp is within 24H (90day is recommended by Intel)
        if (quote!['timestamp'] !== undefined) {
            const timestamp = Date.parse(quote!['timestamp']);
            const now = Date.now();
            const dt = now - timestamp;
            console.log('   [IssuerAttestation] ISV Enclave Quote Delta Time: ', dt);
        } else {
            console.log('   [IssuerAttestation] Failed to fetch timestamp from attestation report');
        }
    }

    console.log('   [IssuerAttestation] check passed.');
}

checkIssuerAttestation();
