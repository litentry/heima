#![allow(dead_code, unused_imports, const_item_mutation)]

use super::{
    collateral::{EnclaveIdentitySigned, TcbInfoSigned},
    *,
};
use frame_support::assert_err;
use hex_literal::hex;
use parity_scale_codec::Decode;

// reproduce with "litentry-worker dump_ra"
const TEST1_CERT: &[u8] = include_bytes!("./test/test_ra_cert_MRSIGNER1_MRENCLAVE1.der");
const TEST2_CERT: &[u8] = include_bytes!("./test/test_ra_cert_MRSIGNER2_MRENCLAVE2.der");
const TEST3_CERT: &[u8] = include_bytes!("./test/test_ra_cert_MRSIGNER3_MRENCLAVE2.der");
const TEST4_CERT: &[u8] = include_bytes!("./test/ra_dump_cert_TEST4.der");
const TEST5_CERT: &[u8] = include_bytes!("./test/ra_dump_cert_TEST5.der");
const TEST6_CERT: &[u8] = include_bytes!("./test/ra_dump_cert_TEST6.der");
const TEST7_CERT: &[u8] = include_bytes!("./test/ra_dump_cert_TEST7.der");
const TEST8_CERT: &[u8] = include_bytes!("./test/ra_dump_cert_TEST8_PRODUCTION.der");

const TEST1_SIGNER_ATTN: &[u8] =
    include_bytes!("./test/test_ra_signer_attn_MRSIGNER1_MRENCLAVE1.bin");
const TEST2_SIGNER_ATTN: &[u8] =
    include_bytes!("./test/test_ra_signer_attn_MRSIGNER2_MRENCLAVE2.bin");
const TEST3_SIGNER_ATTN: &[u8] =
    include_bytes!("./test/test_ra_signer_attn_MRSIGNER3_MRENCLAVE2.bin");

// reproduce with "litentry-worker signing-key"
const TEST1_SIGNER_PUB: &[u8] =
    include_bytes!("./test/test_ra_signer_pubkey_MRSIGNER1_MRENCLAVE1.bin");
const TEST2_SIGNER_PUB: &[u8] =
    include_bytes!("./test/test_ra_signer_pubkey_MRSIGNER2_MRENCLAVE2.bin");
const TEST3_SIGNER_PUB: &[u8] =
    include_bytes!("./test/test_ra_signer_pubkey_MRSIGNER3_MRENCLAVE2.bin");
const TEST4_SIGNER_PUB: &[u8] = include_bytes!("./test/enclave-signing-pubkey-TEST4.bin");
// equal to TEST4!
const TEST5_SIGNER_PUB: &[u8] = include_bytes!("./test/enclave-signing-pubkey-TEST5.bin");
const TEST6_SIGNER_PUB: &[u8] = include_bytes!("./test/enclave-signing-pubkey-TEST6.bin");
const TEST7_SIGNER_PUB: &[u8] = include_bytes!("./test/enclave-signing-pubkey-TEST7.bin");
const QE_IDENTITY_CERT: &str = include_str!("./test/dcap/qe_identity_cert.pem");
const DCAP_QUOTE_CERT: &str = include_str!("./test/dcap/dcap_quote_cert.der");
const PCK_CRL: &[u8] = include_bytes!("./test/dcap/pck_crl.der");

// reproduce with "make mrenclave" in worker repo root
const TEST1_MRENCLAVE: &[u8] = &[
    62, 252, 187, 232, 60, 135, 108, 204, 87, 78, 35, 169, 241, 237, 106, 217, 251, 241, 99, 189,
    138, 157, 86, 136, 77, 91, 93, 23, 192, 104, 140, 167,
];
const TEST2_MRENCLAVE: &[u8] = &[
    4, 190, 230, 132, 211, 129, 59, 237, 101, 78, 55, 174, 144, 177, 91, 134, 1, 240, 27, 174, 81,
    139, 8, 22, 32, 241, 228, 103, 189, 43, 44, 102,
];
const TEST3_MRENCLAVE: &[u8] = &[
    4, 190, 230, 132, 211, 129, 59, 237, 101, 78, 55, 174, 144, 177, 91, 134, 1, 240, 27, 174, 81,
    139, 8, 22, 32, 241, 228, 103, 189, 43, 44, 102,
];

// MRSIGNER is 83d719e77deaca1470f6baf62a4d774303c899db69020f9c70ee1dfc08c7ce9e
const TEST4_MRENCLAVE: MrEnclave =
    hex!("7a3454ec8f42e265cb5be7dfd111e1d95ac6076ed82a0948b2e2a45cf17b62a0");
const TEST5_MRENCLAVE: MrEnclave =
    hex!("f4dedfc9e5fcc48443332bc9b23161c34a3c3f5a692eaffdb228db27b704d9d1");
// equal to TEST5!
const TEST6_MRENCLAVE: MrEnclave =
    hex!("f4dedfc9e5fcc48443332bc9b23161c34a3c3f5a692eaffdb228db27b704d9d1");
// equal to TEST6!
const TEST7_MRENCLAVE: MrEnclave =
    hex!("f4dedfc9e5fcc48443332bc9b23161c34a3c3f5a692eaffdb228db27b704d9d1");

// production mode
// MRSIGNER is 117f95f65f06afb5764b572156b8b525c6230db7d6b1c94e8ebdb7fba068f4e8
const TEST8_MRENCLAVE: MrEnclave =
    hex!("bcf66abfc6b3ef259e9ecfe4cf8df667a7f5a546525dee16822741b38f6e6050");

// unix epoch. must be later than this
const TEST1_TIMESTAMP: i64 = 1580587262i64;
/// Collateral test data mus be valid at this time (2022-10-11 14:01:02) for the tests to work
const COLLATERAL_VERIFICATION_TIMESTAMP: u64 = 1665489662000;

//const CERT: &[u8] =
// b"0\x82\x0c\x8c0\x82\x0c2\xa0\x03\x02\x01\x02\x02\x01\x010\n\x06\x08*\x86H\xce=\x04\x03\x020\
// x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0\x1e\x17\r190617124609Z\x17\r190915124609Z0\x121\
// x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0Y0\x13\x06\x07*\x86H\xce=\x02\x01\x06\x08*\x86H\xce=\
// x03\x01\x07\x03B\0\x04RT\x16\x16
// \xef_\xd8\xe7\xc3\xb7\x03\x1d\xd6:\x1fF\xe3\xf2b!\xa9/\x8b\xd4\x82\x8f\xd1\xff[\x9c\x97\xbc\xf27\
// xb8,L\x8a\x01\xb0r;;\xa9\x83\xdc\x86\x9f\x1d%y\xf4;I\xe4Y\xc80'$K[\xd6\xa3\x82\x0bw0\x82\x0bs0\
// x82\x0bo\x06\t`\x86H\x01\x86\xf8B\x01\r\x04\x82\x0b`{\"id\":\"
// 117077750682263877593646412006783680848\",\"timestamp\":\"2019-06-17T12:46:04.002066\",\"version\
// ":3,\"isvEnclaveQuoteStatus\":\"GROUP_OUT_OF_DATE\",\"platformInfoBlob\":\"
// 1502006504000900000909020401800000000000000000000008000009000000020000000000000B401A355B313FC939B4F48A54349C914A32A3AE2C4871BFABF22E960C55635869FC66293A3D9B2D58ED96CA620B65D669A444C80291314EF691E896F664317CF80C\
// ",\"isvEnclaveQuoteBody\":\"AgAAAEALAAAIAAcAAAAAAOE6wgoHKsZsnVWSrsWX9kky0kWt9K4xcan0fQ996Ct+CAj//
// wGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAHAAAAAAAAAFJJYIbPVot9NzRCjW2z9+k+9K8BsHQKzVMEHOR14hNbAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACD1xnnferKFHD2uvYqTXdDA8iZ22kCD5xw7h38CMfOngAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSVBYWIO9f2OfDtwMd1jofRuPyYiGpL4vUgo/
// R/1ucl7zyN7gsTIoBsHI7O6mD3IafHSV59DtJ5FnIMCckS1vW\"}|EbPFH/ThUaS/
// dMZoDKC5EgmdUXUORFtQzF49Umi1P55oeESreJaUvmA0sg/
// ATSTn5t2e+e6ZoBQIUbLHjcWLMLzK4pJJUeHhok7EfVgoQ378i+eGR9v7ICNDGX7a1rroOe0s1OKxwo/
// 0hid2KWvtAUBvf1BDkqlHy025IOiXWhXFLkb/
// qQwUZDWzrV4dooMfX5hfqJPi1q9s18SsdLPmhrGBheh9keazeCR9hiLhRO9TbnVgR9zJk43SPXW+pHkbNigW+2STpVAi5ugWaSwBOdK11ZjaEU1paVIpxQnlW1D6dj1Zc3LibMH+ly9ZGrbYtuJks4eRnjPhroPXxlJWpQ==|MIIEoTCCAwmgAwIBAgIJANEHdl0yo7CWMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwHhcNMTYxMTIyMDkzNjU4WhcNMjYxMTIwMDkzNjU4WjB7MQswCQYDVQQGEwJVUzELMAkGA1UECAwCQ0ExFDASBgNVBAcMC1NhbnRhIENsYXJhMRowGAYDVQQKDBFJbnRlbCBDb3Jwb3JhdGlvbjEtMCsGA1UEAwwkSW50ZWwgU0dYIEF0dGVzdGF0aW9uIFJlcG9ydCBTaWduaW5nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAqXot4OZuphR8nudFrAFiaGxxkgma/
// Es/BA+tbeCTUR106AL1ENcWA4FX3K+E9BBL0/7X5rj5nIgX/R/1ubhkKWw9gfqPG3KeAtIdcv/
// uTO1yXv50vqaPvE1CRChvzdS/ZEBqQ5oVvLTPZ3VEicQjlytKgN9cLnxbwtuvLUK7eyRPfJW/
// ksddOzP8VBBniolYnRCD2jrMRZ8nBM2ZWYwnXnwYeOAHV+W9tOhAImwRwKF/95yAsVwd21ryHMJBcGH70qLagZ7Ttyt++qO/
// 6+KAXJuKwZqjRlEtSEz8gZQeFfVYgcwSfo96oSMAzVr7V0L6HSDLRnpb6xxmbPdqNol4tQIDAQABo4GkMIGhMB8GA1UdIwQYMBaAFHhDe3amfrzQr35CN+s1fDuHAVE8MA4GA1UdDwEB/
// wQEAwIGwDAMBgNVHRMBAf8EAjAAMGAGA1UdHwRZMFcwVaBToFGGT2h0dHA6Ly90cnVzdGVkc2VydmljZXMuaW50ZWwuY29tL2NvbnRlbnQvQ1JML1NHWC9BdHRlc3RhdGlvblJlcG9ydFNpZ25pbmdDQS5jcmwwDQYJKoZIhvcNAQELBQADggGBAGcIthtcK9IVRz4rRq+ZKE+7k50/
// OxUsmW8aavOzKb0iCx07YQ9rzi5nU73tME2yGRLzhSViFs/
// LpFa9lpQL6JL1aQwmDR74TxYGBAIi5f4I5TJoCCEqRHz91kpG6Uvyn2tLmnIdJbPE4vYvWLrtXXfFBSSPD4Afn7+3/
// XUggAlc7oCTizOfbbtOFlYA4g5KcYgS1J2ZAeMQqbUdZseZCcaZZZn65tdqee8UXZlDvx0+NdO0LR+5pFy+juM0wWbu59MvzcmTXbjsi7HY6zd53Yq5K244fwFHRQ8eOB0IWB+4PfM7FeAApZvlfqlKOlLcZL2uyVmzRkyR5yW72uo9mehX44CiPJ2fse9Y6eQtcfEhMPkmHXI01sN+KwPbpA39+xOsStjhP9N1Y1a2tQAVo+yVgLgV2Hws73Fc0o3wC78qPEA+v2aRs/
// Be3ZFDgDyghc/1fgU+7C+P6kbqd4poyb6IW8KCJbxfMJvkordNOgOUUxndPHEi/tb/U7uLjLOgPA==0\n\x06\x08*\x86H\
// xce=\x04\x03\x02\x03H\00E\x02!\0\xae6\x06\t@Sy\x8f\x8ec\x9d\xdci^Ex*\x92}\xdcG\x15A\x97\xd7\xd7\
// xd1\xccx\xe0\x1e\x08\x02
// \x15Q\xa0BT\xde'~\xec\xbd\x027\xd3\xd8\x83\xf7\xe6Z\xc5H\xb4D\xf7\xe2\r\xa7\xe4^f\x10\x85p";
const CERT_FAKE_QUOTE_STATUS: &[u8] = b"0\x82\x0c\x8c0\x82\x0c2\xa0\x03\x02\x01\x02\x02\x01\x010\n\x06\x08*\x86H\xce=\x04\x03\x020\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0\x1e\x17\r190617124609Z\x17\r190915124609Z0\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0Y0\x13\x06\x07*\x86H\xce=\x02\x01\x06\x08*\x86H\xce=\x03\x01\x07\x03B\0\x04RT\x16\x16 \xef_\xd8\xe7\xc3\xb7\x03\x1d\xd6:\x1fF\xe3\xf2b!\xa9/\x8b\xd4\x82\x8f\xd1\xff[\x9c\x97\xbc\xf27\xb8,L\x8a\x01\xb0r;;\xa9\x83\xdc\x86\x9f\x1d%y\xf4;I\xe4Y\xc80'$K[\xd6\xa3\x82\x0bw0\x82\x0bs0\x82\x0bo\x06\t`\x86H\x01\x86\xf8B\x01\r\x04\x82\x0b`{\"id\":\"117077750682263877593646412006783680848\",\"timestamp\":\"2019-06-17T12:46:04.002066\",\"version\":3,\"isvEnclaveQuoteStatus\":\"OK\",\"platformInfoBlob\":\"1602006504000900000909020401800000000000000000000008000009000000020000000000000B401A355B313FC939B4F48A54349C914A32A3AE2C4871BFABF22E960C55635869FC66293A3D9B2D58ED96CA620B65D669A444C80291314EF691E896F664317CF80C\",\"isvEnclaveQuoteBody\":\"AgAAAEALAAAIAAcAAAAAAOE6wgoHKsZsnVWSrsWX9kky0kWt9K4xcan0fQ996Ct+CAj//wGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAHAAAAAAAAAFJJYIbPVot9NzRCjW2z9+k+9K8BsHQKzVMEHOR14hNbAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACD1xnnferKFHD2uvYqTXdDA8iZ22kCD5xw7h38CMfOngAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSVBYWIO9f2OfDtwMd1jofRuPyYiGpL4vUgo/R/1ucl7zyN7gsTIoBsHI7O6mD3IafHSV59DtJ5FnIMCckS1vW\"}|EbPFH/ThUaS/dMZoDKC5EgmdUXUORFtQzF49Umi1P55oeESreJaUvmA0sg/ATSTn5t2e+e6ZoBQIUbLHjcWLMLzK4pJJUeHhok7EfVgoQ378i+eGR9v7ICNDGX7a1rroOe0s1OKxwo/0hid2KWvtAUBvf1BDkqlHy025IOiXWhXFLkb/qQwUZDWzrV4dooMfX5hfqJPi1q9s18SsdLPmhrGBheh9keazeCR9hiLhRO9TbnVgR9zJk43SPXW+pHkbNigW+2STpVAi5ugWaSwBOdK11ZjaEU1paVIpxQnlW1D6dj1Zc3LibMH+ly9ZGrbYtuJks4eRnjPhroPXxlJWpQ==|MIIEoTCCAwmgAwIBAgIJANEHdl0yo7CWMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwHhcNMTYxMTIyMDkzNjU4WhcNMjYxMTIwMDkzNjU4WjB7MQswCQYDVQQGEwJVUzELMAkGA1UECAwCQ0ExFDASBgNVBAcMC1NhbnRhIENsYXJhMRowGAYDVQQKDBFJbnRlbCBDb3Jwb3JhdGlvbjEtMCsGA1UEAwwkSW50ZWwgU0dYIEF0dGVzdGF0aW9uIFJlcG9ydCBTaWduaW5nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAqXot4OZuphR8nudFrAFiaGxxkgma/Es/BA+tbeCTUR106AL1ENcWA4FX3K+E9BBL0/7X5rj5nIgX/R/1ubhkKWw9gfqPG3KeAtIdcv/uTO1yXv50vqaPvE1CRChvzdS/ZEBqQ5oVvLTPZ3VEicQjlytKgN9cLnxbwtuvLUK7eyRPfJW/ksddOzP8VBBniolYnRCD2jrMRZ8nBM2ZWYwnXnwYeOAHV+W9tOhAImwRwKF/95yAsVwd21ryHMJBcGH70qLagZ7Ttyt++qO/6+KAXJuKwZqjRlEtSEz8gZQeFfVYgcwSfo96oSMAzVr7V0L6HSDLRnpb6xxmbPdqNol4tQIDAQABo4GkMIGhMB8GA1UdIwQYMBaAFHhDe3amfrzQr35CN+s1fDuHAVE8MA4GA1UdDwEB/wQEAwIGwDAMBgNVHRMBAf8EAjAAMGAGA1UdHwRZMFcwVaBToFGGT2h0dHA6Ly90cnVzdGVkc2VydmljZXMuaW50ZWwuY29tL2NvbnRlbnQvQ1JML1NHWC9BdHRlc3RhdGlvblJlcG9ydFNpZ25pbmdDQS5jcmwwDQYJKoZIhvcNAQELBQADggGBAGcIthtcK9IVRz4rRq+ZKE+7k50/OxUsmW8aavOzKb0iCx07YQ9rzi5nU73tME2yGRLzhSViFs/LpFa9lpQL6JL1aQwmDR74TxYGBAIi5f4I5TJoCCEqRHz91kpG6Uvyn2tLmnIdJbPE4vYvWLrtXXfFBSSPD4Afn7+3/XUggAlc7oCTizOfbbtOFlYA4g5KcYgS1J2ZAeMQqbUdZseZCcaZZZn65tdqee8UXZlDvx0+NdO0LR+5pFy+juM0wWbu59MvzcmTXbjsi7HY6zd53Yq5K244fwFHRQ8eOB0IWB+4PfM7FeAApZvlfqlKOlLcZL2uyVmzRkyR5yW72uo9mehX44CiPJ2fse9Y6eQtcfEhMPkmHXI01sN+KwPbpA39+xOsStjhP9N1Y1a2tQAVo+yVgLgV2Hws73Fc0o3wC78qPEA+v2aRs/Be3ZFDgDyghc/1fgU+7C+P6kbqd4poyb6IW8KCJbxfMJvkordNOgOUUxndPHEi/tb/U7uLjLOgPA==0\n\x06\x08*\x86H\xce=\x04\x03\x02\x03H\x000E\x02!\0\xae6\x06\t@Sy\x8f\x8ec\x9d\xdci^Ex*\x92}\xdcG\x15A\x97\xd7\xd7\xd1\xccx\xe0\x1e\x08\x02 \x15Q\xa0BT\xde'~\xec\xbd\x027\xd3\xd8\x83\xf7\xe6Z\xc5H\xb4D\xf7\xe2\r\xa7\xe4^f\x10\x85p";
const CERT_WRONG_PLATFORM_BLOB: &[u8] = b"0\x82\x0c\x8c0\x82\x0c2\xa0\x03\x02\x01\x02\x02\x01\x010\n\x06\x08*\x86H\xce=\x04\x03\x020\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0\x1e\x17\r190617124609Z\x17\r190915124609Z0\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0Y0\x13\x06\x07*\x86H\xce=\x02\x01\x06\x08*\x86H\xce=\x03\x01\x07\x03B\0\x04RT\x16\x16 \xef_\xd8\xe7\xc3\xb7\x03\x1d\xd6:\x1fF\xe3\xf2b!\xa9/\x8b\xd4\x82\x8f\xd1\xff[\x9c\x97\xbc\xf27\xb8,L\x8a\x01\xb0r;;\xa9\x83\xdc\x86\x9f\x1d%y\xf4;I\xe4Y\xc80'$K[\xd6\xa3\x82\x0bw0\x82\x0bs0\x82\x0bo\x06\t`\x86H\x01\x86\xf8B\x01\r\x04\x82\x0b`{\"id\":\"117077750682263877593646412006783680848\",\"timestamp\":\"2019-06-17T12:46:04.002066\",\"version\":3,\"isvEnclaveQuoteStatus\":\"GROUP_OUT_OF_DATE\",\"platformInfoBlob\":\"1602006504000900000909020401800000000000000000000008000009000000020000000000000B401A355B313FC939B4F48A54349C914A32A3AE2C4871BFABF22E960C55635869FC66293A3D9B2D58ED96CA620B65D669A444C80291314EF691E896F664317CF80C\",\"isvEnclaveQuoteBody\":\"AgAAAEALAAAIAAcAAAAAAOE6wgoHKsZsnVWSrsWX9kky0kWt9K4xcan0fQ996Ct+CAj//wGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAHAAAAAAAAAFJJYIbPVot9NzRCjW2z9+k+9K8BsHQKzVMEHOR14hNbAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACD1xnnferKFHD2uvYqTXdDA8iZ22kCD5xw7h38CMfOngAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSVBYWIO9f2OfDtwMd1jofRuPyYiGpL4vUgo/R/1ucl7zyN7gsTIoBsHI7O6mD3IafHSV59DtJ5FnIMCckS1vW\"}|EbPFH/ThUaS/dMZoDKC5EgmdUXUORFtQzF49Umi1P55oeESreJaUvmA0sg/ATSTn5t2e+e6ZoBQIUbLHjcWLMLzK4pJJUeHhok7EfVgoQ378i+eGR9v7ICNDGX7a1rroOe0s1OKxwo/0hid2KWvtAUBvf1BDkqlHy025IOiXWhXFLkb/qQwUZDWzrV4dooMfX5hfqJPi1q9s18SsdLPmhrGBheh9keazeCR9hiLhRO9TbnVgR9zJk43SPXW+pHkbNigW+2STpVAi5ugWaSwBOdK11ZjaEU1paVIpxQnlW1D6dj1Zc3LibMH+ly9ZGrbYtuJks4eRnjPhroPXxlJWpQ==|MIIEoTCCAwmgAwIBAgIJANEHdl0yo7CWMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwHhcNMTYxMTIyMDkzNjU4WhcNMjYxMTIwMDkzNjU4WjB7MQswCQYDVQQGEwJVUzELMAkGA1UECAwCQ0ExFDASBgNVBAcMC1NhbnRhIENsYXJhMRowGAYDVQQKDBFJbnRlbCBDb3Jwb3JhdGlvbjEtMCsGA1UEAwwkSW50ZWwgU0dYIEF0dGVzdGF0aW9uIFJlcG9ydCBTaWduaW5nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAqXot4OZuphR8nudFrAFiaGxxkgma/Es/BA+tbeCTUR106AL1ENcWA4FX3K+E9BBL0/7X5rj5nIgX/R/1ubhkKWw9gfqPG3KeAtIdcv/uTO1yXv50vqaPvE1CRChvzdS/ZEBqQ5oVvLTPZ3VEicQjlytKgN9cLnxbwtuvLUK7eyRPfJW/ksddOzP8VBBniolYnRCD2jrMRZ8nBM2ZWYwnXnwYeOAHV+W9tOhAImwRwKF/95yAsVwd21ryHMJBcGH70qLagZ7Ttyt++qO/6+KAXJuKwZqjRlEtSEz8gZQeFfVYgcwSfo96oSMAzVr7V0L6HSDLRnpb6xxmbPdqNol4tQIDAQABo4GkMIGhMB8GA1UdIwQYMBaAFHhDe3amfrzQr35CN+s1fDuHAVE8MA4GA1UdDwEB/wQEAwIGwDAMBgNVHRMBAf8EAjAAMGAGA1UdHwRZMFcwVaBToFGGT2h0dHA6Ly90cnVzdGVkc2VydmljZXMuaW50ZWwuY29tL2NvbnRlbnQvQ1JML1NHWC9BdHRlc3RhdGlvblJlcG9ydFNpZ25pbmdDQS5jcmwwDQYJKoZIhvcNAQELBQADggGBAGcIthtcK9IVRz4rRq+ZKE+7k50/OxUsmW8aavOzKb0iCx07YQ9rzi5nU73tME2yGRLzhSViFs/LpFa9lpQL6JL1aQwmDR74TxYGBAIi5f4I5TJoCCEqRHz91kpG6Uvyn2tLmnIdJbPE4vYvWLrtXXfFBSSPD4Afn7+3/XUggAlc7oCTizOfbbtOFlYA4g5KcYgS1J2ZAeMQqbUdZseZCcaZZZn65tdqee8UXZlDvx0+NdO0LR+5pFy+juM0wWbu59MvzcmTXbjsi7HY6zd53Yq5K244fwFHRQ8eOB0IWB+4PfM7FeAApZvlfqlKOlLcZL2uyVmzRkyR5yW72uo9mehX44CiPJ2fse9Y6eQtcfEhMPkmHXI01sN+KwPbpA39+xOsStjhP9N1Y1a2tQAVo+yVgLgV2Hws73Fc0o3wC78qPEA+v2aRs/Be3ZFDgDyghc/1fgU+7C+P6kbqd4poyb6IW8KCJbxfMJvkordNOgOUUxndPHEi/tb/U7uLjLOgPA==0\n\x06\x08*\x86H\xce=\x04\x03\x02\x03H\x000E\x02!\0\xae6\x06\t@Sy\x8f\x8ec\x9d\xdci^Ex*\x92}\xdcG\x15A\x97\xd7\xd7\xd1\xccx\xe0\x1e\x08\x02 \x15Q\xa0BT\xde'~\xec\xbd\x027\xd3\xd8\x83\xf7\xe6Z\xc5H\xb4D\xf7\xe2\r\xa7\xe4^f\x10\x85p";
const CERT_WRONG_SIG: &[u8] = b"0\x82\x0c\x8c0\x82\x0c2\xa0\x03\x02\x01\x02\x02\x01\x010\n\x06\x08*\x86H\xce=\x04\x03\x020\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0\x1e\x17\r190617124609Z\x17\r190915124609Z0\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0Y0\x13\x06\x07*\x86H\xce=\x02\x01\x06\x08*\x86H\xce=\x03\x01\x07\x03B\0\x04RT\x16\x16 \xef_\xd8\xe7\xc3\xb7\x03\x1d\xd6:\x1fF\xe3\xf2b!\xa9/\x8b\xd4\x82\x8f\xd1\xff[\x9c\x97\xbc\xf27\xb8,L\x8a\x01\xb0r;;\xa9\x83\xdc\x86\x9f\x1d%y\xf4;I\xe4Y\xc80'$K[\xd6\xa3\x82\x0bw0\x82\x0bs0\x82\x0bo\x06\t`\x86H\x01\x86\xf8B\x01\r\x04\x82\x0b`{\"id\":\"117077750682263877593646412006783680848\",\"timestamp\":\"2019-06-17T12:46:04.002066\",\"version\":3,\"isvEnclaveQuoteStatus\":\"GROUP_OUT_OF_DATE\",\"platformInfoBlob\":\"1602006504000900000909020401800000000000000000000008000009000000020000000000000B401A355B313FC939B4F48A54349C914A32A3AE2C4871BFABF22E960C55635869FC66293A3D9B2D58ED96CA620B65D669A444C80291314EF691E896F664317CF80C\",\"isvEnclaveQuoteBody\":\"AgAAAEALAAAIAAcAAAAAAOE6wgoHKsZsnVWSrsWX9kky0kWt9K4xcan0fQ996Ct+CAj//wGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAHAAAAAAAAAFJJYIbPVot9NzRCjW2z9+k+9K8BsHQKzVMEHOR14hNbAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACD1xnnferKFHD2uvYqTXdDA8iZ22kCD5xw7h38CMfOngAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSVBYWIO9f2OfDtwMd1jofRuPyYiGpL4vUgo/R/1ucl7zyN7gsTIoBsHI7O6mD3IafHSV59DtJ5FnIMCckS1vW\"}|EbPFH/ThUaS/dMZoDKC5EgmdUXUORFtQzF49Umi1P55oeESreJaUvmA0sg/ATSTn5t2e+e6ZoBQIUbLHjcWLMLzK4pJJUeHhok7EfVgoQ378i+eGR9v7ICNDGX7a1rroOe0s1OKxwo/0hid2KWvtAUBvf1BDkqlHy025IOiXWhXFLkb/qQwUZDWzrV4dooMfX5hfqJPi1q9s18SsdLPmhrGBheh9keazeCR9hiLhRO9TbnVgR9zJk43SPXW+pHkbNigW+2STpVAi5ugWaSwBOdK11ZjaEU1paVIpxQnlW1D6dj1Zc3LibMH+ly9ZGrbYtuJks4eRnjPhroPXxlJWpQ==|MIIEoTCCAwmgAwIBAgIJANEHdl0yo7CWMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwHhcNMTYxMTIyMDkzNjU4WhcNMjYxMTIwMDkzNjU4WjB7MQswCQYDVQQGEwJVUzELMAkGA1UECAwCQ0ExFDASBgNVBAcMC1NhbnRhIENsYXJhMRowGAYDVQQKDBFJbnRlbCBDb3Jwb3JhdGlvbjEtMCsGA1UEAwwkSW50ZWwgU0dYIEF0dGVzdGF0aW9uIFJlcG9ydCBTaWduaW5nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAqXot4OZuphR8nudFrAFiaGxxkgma/Es/BA+tbeCTUR106AL1ENcWA4FX3K+E9BBL0/7X5rj5nIgX/R/1ubhkKWw9gfqPG3KeAtIdcv/uTO1yXv50vqaPvE1CRChvzdS/ZEBqQ5oVvLTPZ3VEicQjlytKgN9cLnxbwtuvLUK7eyRPfJW/ksddOzP8VBBniolYnRCD2jrMRZ8nBM2ZWYwnXnwYeOAHV+W9tOhAImwRwKF/95yAsVwd21ryHMJBcGH70qLagZ7Ttyt++qO/6+KAXJuKwZqjRlEtSEz8gZQeFfVYgcwSfo96oSMAzVr7V0L6HSDLRnpb6xxmbPdqNol4tQIDAQABo4GkMIGhMB8GA1UdIwQYMBaAFHhDe3amfrzQr35CN+s1fDuHAVE8MA4GA1UdDwEB/wQEAwIGwDAMBgNVHRMBAf8EAjAAMGAGA1UdHwRZMFcwVaBToFGGT2h0dHA6Ly90cnVzdGVkc2VydmljZXMuaW50ZWwuY29tL2NvbnRlbnQvQ1JML1NHWC9BdHRlc3RhdGlvblJlcG9ydFNpZ25pbmdDQS5jcmwwDQYJKoZIhvcNAQELBQADggGBAGcIthtcK9IVRz4rRq+ZKE+7k50/OxUsmW8aavOzKb0iCx07YQ9rzi5nU73tME2yGRLzhSViFs/LpFa9lpQL6JL1aQwmDR74TxYGBAIi5f4I5TJoCCEaRHz91kpG6Uvyn2tLmnIdJbPE4vYvWLrtXXfFBSSPD4Afn7+3/XUggAlc7oCTizOfbbtOFlYA4g5KcYgS1J2ZAeMQqbUdZseZCcaZZZn65tdqee8UXZlDvx0+NdO0LR+5pFy+juM0wWbu59MvzcmTXbjsi7HY6zd53Yq5K244fwFHRQ8eOB0IWB+4PfM7FeAApZvlfqlKOlLcZL2uyVmzRkyR5yW72uo9mehX44CiPJ2fse9Y6eQtcfEhMPkmHXI01sN+KwPbpA39+xOsStjhP9N1Y1a2tQAVo+yVgLgV2Hws73Fc0o3wC78qPEA+v2aRs/Be3ZFDgDyghc/1fgU+7C+P6kbqd4poyb6IW8KCJbxfMJvkordNOgOUUxndPHEi/tb/U7uLjLOgPA==0\n\x06\x08*\x86H\xce=\x04\x03\x02\x03H\x000E\x02!\0\xae6\x06\t@Sy\x8f\x8ec\x9d\xdci^Ex*\x92}\xdcG\x15A\x97\xd7\xd7\xd1\xccx\xe0\x1e\x08\x02 \x15Q\xa0BT\xde'~\xec\xbd\x027\xd3\xd8\x83\xf7\xe6Z\xc5H\xb4D\xf7\xe2\r\xa7\xe4^f\x10\x85p";
const CERT_TOO_SHORT1: &[u8] = b"0\x82\x0c\x8c0\x82\x0c2\xa0\x03\x02\x01\x02\x02\x01\x010\n\x06\x08*\x86H\xce=\x04\x03\x020\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0\x1e\x17\r190617124609Z\x17\r190915124609Z0\x121\x100\x0e\x06\x03U\x04\x03\x0c\x07MesaTEE0Y0\x13\x06\x07*\x86H\xce=\x02\x01\x06\x08*\x86H\xce=\x03\x01\x07\x03B\0\x04RT\x16\x16 \xef_\xd8\xe7\xc3\xb7\x03\x1d\xd6:\x1fF\xe3\xf2b!\xa9/\x8b\xd4\x82\x8f\xd1\xff[\x9c\x97\xbc\xf27\xb8,L\x8a\x01\xb0r;;\xa9\x83\xdc\x86\x9f\x1d%y\xf4;I\xe4Y\xc80'$K[\xd6\xa3\x82\x0bw0\x82\x0bs0\x82\x0bo\x06\t`\x86H\x01\x86\xf8B\x01\r\x04\x82\x0b`{\"id\":\"117077750682263877593646412006783680848\",\"timestamp\":\"2019-06-17T12:46:04.002066\",\"version\":3,\"isvEnclaveQuoteStatus\":\"GROUP_OUT_OF_DATE\",\"platformInfoBlob\":\"1602006504000900000909020401800000000000000000000008000009000000020000000000000B401A355B313FC939B4F48A54349C91\x03\x02\x03H\x000E\x02!\0\xae6\x06\t@Sy\x8f\x8ec\x9d\xdci^Ex*\x92}\xdcG\x15A\x97\xd7\xd7\xd1\xccx\xe0\x1e\x08\x02 \x15Q\xa0BT\xde'~\xec\xbd\x027\xd3\xd8\x83\xf7\xe6Z\xc5H\xb4D\xf7\xe2\r\xa7\xe4^f\x10\x85p";
const CERT_TOO_SHORT2: &[u8] = b"0\x82\x0c\x8c0";

#[test]
fn verify_ias_report_should_work() {
    let _signer_attn: [u32; 16] = Decode::decode(&mut TEST1_SIGNER_ATTN).unwrap();
    let report = verify_ias_report(TEST4_CERT);
    let report = report.unwrap();
    assert_eq!(report.mr_enclave, TEST4_MRENCLAVE);
    assert!(report.timestamp >= TEST1_TIMESTAMP.try_into().unwrap());
    assert_eq!(report.pubkey, TEST4_SIGNER_PUB);
    //assert_eq!(report.status, SgxStatus::GroupOutOfDate);
    assert_eq!(report.status, SgxStatus::ConfigurationNeeded);
    assert_eq!(report.build_mode, SgxBuildMode::Debug);
}

#[test]
fn verify_zero_length_cert_returns_err() {
    // CERT empty, argument 2 and 3 are wrong too!
    let _signer_attn: [u32; 16] = Decode::decode(&mut TEST1_SIGNER_ATTN).unwrap();
    assert!(verify_ias_report(&Vec::new()[..]).is_err())
}

#[test]
fn verify_wrong_cert_is_err() {
    // CERT wrong, argument 2 and 3 are wrong too!
    let _signer_attn: [u32; 16] = Decode::decode(&mut TEST1_SIGNER_ATTN).unwrap();
    assert!(verify_ias_report(CERT_WRONG_PLATFORM_BLOB).is_err())
}

#[test]
fn verify_wrong_fake_enclave_quote_is_err() {
    // quote wrong, argument 2 and 3 are wrong too!
    let _signer_attn: [u32; 16] = Decode::decode(&mut TEST1_SIGNER_ATTN).unwrap();
    assert!(verify_ias_report(CERT_FAKE_QUOTE_STATUS).is_err())
}

#[test]
fn verify_wrong_sig_is_err() {
    // sig wrong, argument 2 and 3 are wrong too!
    let _signer_attn: [u32; 16] = Decode::decode(&mut TEST1_SIGNER_ATTN).unwrap();
    assert!(verify_ias_report(CERT_WRONG_SIG).is_err())
}

#[test]
fn verify_short_cert_is_err() {
    let _signer_attn: [u32; 16] = Decode::decode(&mut TEST1_SIGNER_ATTN).unwrap();
    assert!(verify_ias_report(CERT_TOO_SHORT1).is_err());
    assert!(verify_ias_report(CERT_TOO_SHORT2).is_err());
}

#[test]
fn fix_incorrect_handling_of_iterator() {
    // In `verify_ias_report` we called `iter.next()` with unwrap three times, which could fail
    // for certain invalid reports as the one in this test. This test verifies that the issue
    // has been fixed.
    //
    // For context, see: https://github.com/integritee-network/pallet-teerex/issues/35

    let report: [u8; 56] = [
        224, 224, 224, 224, 224, 224, 224, 224, 235, 2, 0, 1, 5, 40, 0, 8, 255, 6, 8, 42, 134, 72,
        206, 61, 3, 1, 7, 0, 2, 183, 64, 48, 48, 0, 1, 10, 23, 3, 6, 9, 96, 134, 72, 1, 134, 248,
        66, 1, 13, 0, 0, 0, 13, 1, 14, 177,
    ];

    assert_err!(verify_ias_report(&report), "Invalid netscape payload");
}

#[test]
fn verify_sgx_build_mode_works() {
    //verify report from enclave in debug mode
    let report = verify_ias_report(TEST4_CERT);
    let report = report.unwrap();
    assert_eq!(report.build_mode, SgxBuildMode::Debug);
    //verify report from enclave in production mode
    let report = verify_ias_report(TEST8_CERT);
    let report = report.unwrap();
    assert_eq!(report.build_mode, SgxBuildMode::Production);
}

#[test]
fn decode_qe_authentication_data() {
    assert!(QeAuthenticationData::decode(&mut &[0u8][..]).is_err());
    assert!(QeAuthenticationData::decode(&mut &[1u8][..]).is_err());
    assert_eq!(
        0,
        QeAuthenticationData::decode(&mut &[0u8, 0][..])
            .unwrap()
            .size
    );
    let d = QeAuthenticationData::decode(&mut &[1u8, 0, 5][..]).unwrap();
    assert_eq!(1, d.size);
    assert_eq!(5, d.certification_data[0]);
}

#[test]
fn decode_qe_certification_data() {
    assert!(QeCertificationData::decode(&mut &[0u8][..]).is_err());
    assert!(QeCertificationData::decode(&mut &[1u8, 0, 0, 0, 0][..]).is_err());
    assert_eq!(
        0,
        QeCertificationData::decode(&mut &[0u8, 0, 0, 0, 0, 0][..])
            .unwrap()
            .size
    );
    let d = QeCertificationData::decode(&mut &[0u8, 0, 1, 0, 0, 0, 5][..]).unwrap();
    assert_eq!(1, d.size);
    assert_eq!(5, d.certification_data[0]);
    assert!(QeCertificationData::decode(&mut &[0u8, 0, 2, 0, 0, 0, 5][..]).is_err());
}

#[test]
fn deserialize_qe_identity_works() {
    let certs = extract_certs(include_bytes!("./test/dcap/qe_identity_issuer_chain.pem"));
    let intermediate_slices: Vec<webpki::types::CertificateDer> =
        certs[1..].iter().map(|c| c.as_slice().into()).collect();
    let leaf_cert_der = webpki::types::CertificateDer::from(certs[0].as_slice());
    let leaf_cert = webpki::EndEntityCert::try_from(&leaf_cert_der).unwrap();
    verify_certificate_chain(
        &leaf_cert,
        &intermediate_slices,
        COLLATERAL_VERIFICATION_TIMESTAMP,
    )
    .unwrap();
    let json: EnclaveIdentitySigned =
        serde_json::from_slice(include_bytes!("./test/dcap/qe_identity.json")).unwrap();
    let json_data = serde_json::to_vec(&json.enclave_identity).unwrap();
    let signature = hex::decode(json.signature).unwrap();

    let e = deserialize_enclave_identity(&json_data, &signature, &leaf_cert).unwrap();
    assert_eq!(1, e.isvprodid);
    assert_eq!(5, e.tcb_levels.len());
}

#[test]
fn deserialize_tcb_info_works() {
    let certs = extract_certs(include_bytes!("./test/dcap/tcb_info_issuer_chain.pem"));
    let intermediate_slices: Vec<webpki::types::CertificateDer> =
        certs[1..].iter().map(|c| c.as_slice().into()).collect();
    let leaf_cert_der = webpki::types::CertificateDer::from(certs[0].as_slice());
    let leaf_cert = webpki::EndEntityCert::try_from(&leaf_cert_der).unwrap();
    verify_certificate_chain(
        &leaf_cert,
        &intermediate_slices,
        COLLATERAL_VERIFICATION_TIMESTAMP,
    )
    .unwrap();
    let json: TcbInfoSigned =
        serde_json::from_slice(include_bytes!("./test/dcap/tcb_info.json")).unwrap();

    let json_data = serde_json::to_vec(&json.tcb_info).unwrap();
    let signature = hex::decode(json.signature).unwrap();

    let _e = deserialize_tcb_info(&json_data, &signature, &leaf_cert).unwrap();
    assert_eq!(hex!("00906EA10000"), json.tcb_info.fmspc);
}

#[test]
fn verify_tcb_info_signature() {
    let cert = QE_IDENTITY_CERT.replace('\n', "");
    let leaf_cert = base64::decode(cert).unwrap();
    let leaf_cert_der = webpki::types::CertificateDer::from(leaf_cert.as_slice());
    let leaf_cert = webpki::EndEntityCert::try_from(&leaf_cert_der).unwrap();
    let data = br#"{"version":2,"issueDate":"2022-10-18T21:45:02Z","nextUpdate":"2022-11-17T21:45:02Z","fmspc":"00906EA10000","pceId":"0000","tcbType":0,"tcbEvaluationDataNumber":12,"tcbLevels":[{"tcb":{"sgxtcbcomp01svn":17,"sgxtcbcomp02svn":17,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":7,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":11},"tcbDate":"2021-11-10T00:00:00Z","tcbStatus":"SWHardeningNeeded"},{"tcb":{"sgxtcbcomp01svn":17,"sgxtcbcomp02svn":17,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":7,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":10},"tcbDate":"2020-11-11T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"sgxtcbcomp01svn":17,"sgxtcbcomp02svn":17,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":11},"tcbDate":"2021-11-10T00:00:00Z","tcbStatus":"ConfigurationAndSWHardeningNeeded"},{"tcb":{"sgxtcbcomp01svn":17,"sgxtcbcomp02svn":17,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":10},"tcbDate":"2020-11-11T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded"},{"tcb":{"sgxtcbcomp01svn":15,"sgxtcbcomp02svn":15,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":7,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":10},"tcbDate":"2020-06-10T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"sgxtcbcomp01svn":15,"sgxtcbcomp02svn":15,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":10},"tcbDate":"2020-06-10T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded"},{"tcb":{"sgxtcbcomp01svn":14,"sgxtcbcomp02svn":14,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":7,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":10},"tcbDate":"2019-12-11T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"sgxtcbcomp01svn":14,"sgxtcbcomp02svn":14,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":10},"tcbDate":"2019-12-11T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded"},{"tcb":{"sgxtcbcomp01svn":13,"sgxtcbcomp02svn":13,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":3,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":9},"tcbDate":"2019-11-13T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"sgxtcbcomp01svn":13,"sgxtcbcomp02svn":13,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":9},"tcbDate":"2019-11-13T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded"},{"tcb":{"sgxtcbcomp01svn":6,"sgxtcbcomp02svn":6,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":1,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":7},"tcbDate":"2019-05-15T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"sgxtcbcomp01svn":6,"sgxtcbcomp02svn":6,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":7},"tcbDate":"2019-05-15T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded"},{"tcb":{"sgxtcbcomp01svn":5,"sgxtcbcomp02svn":5,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":1,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":7},"tcbDate":"2019-01-09T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"sgxtcbcomp01svn":5,"sgxtcbcomp02svn":5,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":1,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":6},"tcbDate":"2018-08-15T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"sgxtcbcomp01svn":5,"sgxtcbcomp02svn":5,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":7},"tcbDate":"2019-01-09T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded"},{"tcb":{"sgxtcbcomp01svn":5,"sgxtcbcomp02svn":5,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":6},"tcbDate":"2018-08-15T00:00:00Z","tcbStatus":"OutOfDateConfigurationNeeded"},{"tcb":{"sgxtcbcomp01svn":4,"sgxtcbcomp02svn":4,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":5},"tcbDate":"2018-01-04T00:00:00Z","tcbStatus":"OutOfDate"},{"tcb":{"sgxtcbcomp01svn":2,"sgxtcbcomp02svn":2,"sgxtcbcomp03svn":2,"sgxtcbcomp04svn":4,"sgxtcbcomp05svn":1,"sgxtcbcomp06svn":128,"sgxtcbcomp07svn":0,"sgxtcbcomp08svn":0,"sgxtcbcomp09svn":0,"sgxtcbcomp10svn":0,"sgxtcbcomp11svn":0,"sgxtcbcomp12svn":0,"sgxtcbcomp13svn":0,"sgxtcbcomp14svn":0,"sgxtcbcomp15svn":0,"sgxtcbcomp16svn":0,"pcesvn":4},"tcbDate":"2017-07-26T00:00:00Z","tcbStatus":"OutOfDate"}]}"#;
    let signature = hex!("e0cc3102e9ffdb21cf156ba30f13d027210ab11f3bff349e670e4c49b2f0cb6889c7eeb436149c7efe53e15c97e6ec3fc9f34c3440e732a4c760f8eb91834a36");
    let signature = encode_as_der(&signature).unwrap();
    verify_signature(
        &leaf_cert,
        data,
        &signature,
        webpki::ring::ECDSA_P256_SHA256,
    )
    .unwrap();
}

/// This is demo code of how a CRL certificate can be parsed and how the revoked serials can be
/// extracted The part that is missing/open is how to verify the certificate chain of the CRL
/// TODO: Implement CRL handling
#[test]
fn parse_pck_crl() {
    let crl_decoded = hex::decode(PCK_CRL).unwrap();
    let crl: x509_cert::crl::CertificateList = der::Decode::from_der(&crl_decoded).unwrap();

    let mut serials = vec![];
    if let Some(certs) = crl.tbs_cert_list.revoked_certificates {
        for c in certs {
            let serial = c.serial_number.as_bytes().to_vec();
            serials.push(serial);
        }
    }
    assert_eq!(3, serials.len());
}

#[test]
fn parse_pck_certificate() {
    let der = DCAP_QUOTE_CERT.replace('\n', "");
    let der = base64::decode(der).unwrap();

    let ext = get_intel_extension(&der).unwrap();
    assert_eq!(453, ext.len());

    let fmspc = get_fmspc(&ext).unwrap();
    assert_eq!(hex!("00906EA10000"), fmspc);

    let cpusvn = get_cpusvn(&ext).unwrap();
    assert_eq!(hex!("11110204018007000000000000000000"), cpusvn);

    let pcesvn = get_pcesvn(&ext).unwrap();
    assert_eq!(u16::from_be_bytes(hex!("000B")), pcesvn);
}
