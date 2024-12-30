
# Release package


## Step 0: Preparation

This package is generated from [litentry-parachain](https://github.com/litentry/heima)
From the root folder ~/litentry-parachain/tee-worker/:
```
make release-pkg
```
A release package will be generated, within which there are:

- enclave.sign.so
- litentry-worker
- config.json.eg
- prepare.sh

<br>

## Step 1: Deploy on production

Before starting the workers, please make sure the target parachain is already up and accessable. As well as the following directory/files:

| Name | Value | Comment |
|-----|------|---|
| WORKER_DIR | /opt/worker | Working directory of workers |
| CONFIG_DIR | /opt/configs | Config directory which contains the following 4 secret files |
|
| CONFIG | config.json | Configs for twitter/discord/data provider/etc. url/keys. Take reference from config.json.eg |
| ACCOUNT | account.json | Substrate account exported json file |
| INTEL_KEY | key_production.txt | Intel SGX production key. Need to apply from Intel |
| INTEL_SPI | spid_production.txt | Intel SGX production spid. Need to apply from Intel |

<br>

1. Extract the release package to one target location. Worker will be executed from there. Then execute `prepare.sh`:
    ```
    ./prepare.sh
    ```
    This script will generate out `MRENCLAVE` hex value (mrenclave.txt) and `Enclave Account` info (account.txt). They will be used later by ts scripts to setup enclave account.
    <br>

2. Startup options.

    The service will start up like this example:
    ```
    RUST_LOG=info,litentry_worker=debug,ws=warn,sp_io=error,substrate_api_client=warn,itc_parentchain_light_client=info,jsonrpsee_ws_client=warn,jsonrpsee_ws_server=warn,enclave_runtime=debug,ita_stf=debug,its_rpc_handler=warn,itc_rpc_client=warn,its_consensus_common=debug,its_state=warn,its_consensus_aura=warn,aura*=warn,its_consensus_slots=warn,itp_attestation_handler=debug,http_req=debug,lc_mock_server=warn,itc_rest_client=debug,lc_credentials=debug,lc_identity_verification=debug,lc_stf_task_receiver=debug,lc_stf_task_sender=debug,lc_data_providers=debug,itp_top_pool=debug,itc_parentchain_indirect_calls_executor=debug ./litentry-worker --clean-reset --ws-external --mu-ra-external-address localhost --mu-ra-port 3443 --node-port 9944 --node-url ws://127.0.0.1 --trusted-external-address wss://localhost --trusted-worker-port 2000 --untrusted-external-address ws://localhost --untrusted-http-port 4545 --untrusted-worker-port 3000 --enable-mock-server run --skip-ra --dev
    ```
    The first part is RUST_LOG info. In production env, most of them will be disabled. Or `RUST_LOG=info` is enough.

    Starting from `./litentry-worker`, the following is the real startup options:

    ```
    USAGE:
    litentry-worker [FLAGS] [OPTIONS] <SUBCOMMAND>

    FLAGS:
        -c, --clean-reset           Cleans and purges any previous state and key files and generates them anew before starting.
            --enable-metrics        Enable the metrics HTTP server to serve metrics
            --enable-mock-server    Set this flag to enable starting the mock server.
            --help                  Prints help information
        -V, --version               Prints version information
            --ws-external           Set this flag in case the worker should listen to external requests.

    OPTIONS:
        -i, --metrics-port <metrics-port>
                Set the port on which the metrics are served. [default: 8787]

            --mock-server-port <mock-server-port>
                Set the port for the mock-server HTTP server [default: 9527]

        -M, --mu-ra-external-address <mu-ra-external-address>
                Set the mutual remote attestation worker address to be retrieved by a trusted rpc call. If no port is given, the same as in `mu-ra-port` will be used.
        -r, --mu-ra-port <mu-ra-port>
                Set the websocket port to listen for mu-ra requests [default: 3443]

        -p, --node-port <node-port>
                Set the websocket port to listen for substrate events [default: 9944]

        -u, --node-url <node-server>
                Set the node server protocol and IP address [default: ws://127.0.0.1]

        -T, --trusted-external-address <trusted-external-address>
                Set the trusted worker address to be advertised on the parentchain. If no port is given, the same as in
                `trusted-worker-port` will be used.
        -P, --trusted-worker-port <trusted-worker-port>
                Set the trusted websocket port of the worker, running directly in the enclave. [default: 2000]

        -U, --untrusted-external-address <untrusted-external-address>
                Set the untrusted worker address to be retrieved by a trusted rpc call. If no port is given, the same as in
                `untrusted-worker-port` will be used.
        -h, --untrusted-http-port <untrusted-http-port>                  Set the port for the untrusted HTTP server
        -w, --untrusted-worker-port <untrusted-worker-port>
                Set the untrusted websocket port of the worker [default: 2001]

    SUBCOMMANDS:
    dump-ra          Perform RA and dump cert to disk
    help             Prints this message or the help of the given subcommand(s)
    init-shard       Initialize new shard (do this only if you run the first worker for that shard). if shard is not
                     specified, the MRENCLAVE is used instead
    migrate-shard    Migrate shard
    mrenclave        Dump mrenclave to stdout. base58 encoded.
    request-state    join a shard by requesting key provisioning from another worker
    run              Start the litentry-worker
    shielding-key    Get the public RSA3072 key from the TEE to be used to encrypt requests
    signing-key      Get the public ed25519 key the TEE uses to sign messages and extrinsics
    test             Run tests involving the enclave
    ```

