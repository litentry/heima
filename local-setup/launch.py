#!/usr/bin/env python3
"""
Launch handily a local dev setup consisting of the parachain network and some workers.

Example usage: `./local-setup/launch.py --parachain local-binary --worker identity`
Standalone + 3 workers: `./local-setup/launch.py --worker identity -wn 3`

The worker log is piped to `./log/worker*.log` etc. folder in the current-working dir.

"""
import argparse
import json
import signal
from subprocess import run
import os
import shutil
import sys
from time import sleep
from typing import Union, IO
from dotenv import load_dotenv

import pycurl
from io import BytesIO

from py.worker import Worker
from py.helpers import GracefulKiller, mkdir_p

import socket
import toml
import datetime

log_dir = "log"
mkdir_p(log_dir)

OFFSET = 100
PORTS = [
    "AliceWSPort",
    "AlicePort",
    "BobWSPort",
    "BobPort",
    "TrustedWorkerPort",
    "UntrustedWorkerPort",
    "MuRaPort",
    "UntrustedHttpPort",
]


def setup_worker(work_dir: str, source_dir: str, worker_bin: str, std_err: Union[None, int, IO], log_config_path):
    print(f"Setting up worker in {work_dir}")
    print(f"Copying files from {source_dir}")

    log_level_dic = setup_worker_log_level(log_config_path)
    worker = Worker(cwd=work_dir, source_dir=source_dir, worker_bin=worker_bin, std_err=std_err, log_level_dic=log_level_dic)
    worker.init_clean()
    print("Initialized worker.")
    return worker


def run_worker(id, worker_dir, worker_bin, flags, subcommand_flags, log_config_path):
    log = open(f"{log_dir}/worker-{id}.log", "w+")

    w = setup_worker(f"tmp/w-{id}", worker_dir + "/bin", worker_bin, log, log_config_path)

    print(f"Starting worker {id} in background")
    return w.run_in_background(
        log_file=log, flags=flags, subcommand_flags=subcommand_flags
    )


# Function to check if a port is open
def is_port_open(port):
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.bind(("127.0.0.1", int(port)))
        sock.close()
        return True
    except OSError:
        return False

# Function to reallocate port if it is not available
def reallocate_ports(env_name, port):
    # Offset the original port by 100
    new_port = int(port) + int(OFFSET)
    while not is_port_open(str(new_port)):
        new_port = int(port) + int(OFFSET)

    # Set the new port value in the environment variable
    os.environ[env_name] = str(new_port)
    print("Port for {} changed to: {}".format(env_name, os.environ.get(env_name)))


# Function to iterate over all ports and automatically reallocate
def check_all_ports_and_reallocate():
    for x in PORTS:
        if is_port_open(os.environ.get(x)):
            continue
        else:
            reallocate_ports(x, os.environ.get(x))

    print("All preliminary port checks completed")


# Generate `config.local.json` used by parachain ts utils
def generate_config_local_json(parachain_dir):
    data = {
        "eth_endpoint": "http://127.0.0.1:8545",
        "eth_address": "[0x4d88dc5d528a33e4b8be579e9476715f60060582]",
        "private_key": "0xe82c0c4259710bb0d6cf9f9e8d0ad73419c1278a14d375e5ca691e7618103011",
        "ocw_account": "5FEYX9NES9mAJt1Xg4WebmHWywxyeGQK8G3oEBXtyfZrRePX",
        "genesis_state_path": parachain_dir+"/genesis-state",
        "genesis_wasm_path": parachain_dir+"/genesis-wasm",
        "parachain_ws": "ws://localhost:" + os.environ.get("CollatorWSPort", "9944"),
        "relaychain_ws": "ws://localhost:" + os.environ.get("AliceWSPort", "9946"),
        "bridge_path": "/tmp/parachain_dev/chainbridge",
        "parachain_fast_runtime": "true",
    }
    config_file = "./parachain/ts-tests/config.local.json"

    with open(config_file, "w") as f:
        json.dump(data, f, indent=4)

    print("Successfully written ", config_file)


# Generate `.env.local` used by local enclave ts-tests
def generate_env_local(worker_dir):
    if worker_dir == "tee-worker":
        env_local_example_file = "./" + worker_dir + "/ts-tests/integration-tests/.env.local.example"
        env_local_file = env_local_example_file[: -len(".example")]

        with open(env_local_example_file, "r") as f:
            data = f.read()
            data = data.replace(":2000", ":" + os.environ.get("TrustedWorkerPort", "2000"))
            data = data.replace(":9944", ":" + os.environ.get("CollatorWSPort", "9944"))

        with open(env_local_file, "w") as f:
            f.write(data)

        print("Successfully written ", env_local_file)


def offset_port(offset):
    for x in PORTS:
        port = os.environ.get(x)
        new_port = int(port) + int(offset)
        os.environ[x] = str(new_port)


def setup_environment(offset, parachain_dir, worker_dir):
    if not os.path.isfile("./local-setup/.env"):
        shutil.copy("./local-setup/.env.dev", "./local-setup/.env")

    load_dotenv("./local-setup/.env")
    offset_port(offset)
    check_all_ports_and_reallocate()

    if parachain_dir != "":
        generate_config_local_json(parachain_dir)

    generate_env_local(worker_dir)

def setup_worker_log_level(log_config_path):
    log_level_dic = {}
    with open(log_config_path) as f:
        log_data = toml.load(f)

        # Section
        for (section, item) in log_data.items():
            log_level_string = ""
            indx = 0

            for (k, v) in item.items():
                if indx == 0:
                    log_level_string += v+","
                else:
                    log_level_string += k+"="+v+","

                indx += 1

            log_level_dic[section] = log_level_string

    return log_level_dic


def get_flags(index, worker):
    woker_offset = index * 10
    port_with_offset = lambda env_name: str(int(os.environ.get(env_name)) + woker_offset)
    ports = {
        'trusted_worker_port': port_with_offset("TrustedWorkerPort"),
        'untrusted_worker_port': port_with_offset("UntrustedWorkerPort"),
        'mura_port': port_with_offset("MuRaPort"),
        'untrusted_http_port': port_with_offset("UntrustedHttpPort"),
        'collator_ws_port': os.environ.get("CollatorWSPort"),
    }

    return list(filter(None, [
        "--clean-reset",
        "-T", "ws://localhost",
        "-P", ports['trusted_worker_port'],
        "-w", ports['untrusted_worker_port'],
        "-r", ports['mura_port'],
        "-h", ports['untrusted_http_port'],
        "-p", ports['collator_ws_port'],
        "--enable-mock-server",
        "--parentchain-start-block", "0",
        "--enable-metrics" if index == 0 else None
    ]))

def get_subcommand_flags(index):
    return list(filter(None, [
        "--skip-ra",
        "--dev"
    ]))

def add_collator_ports():
    PORTS.extend(
        [
            "CollatorWSPort",
            "CollatorPort",
        ]
    )    

def main(processes, worker, workers_number, parachain_type, log_config_path, offset, parachain_dir):
    # Litentry
    worker_dir = "tee-worker/identity"
    worker_bin = "litentry-worker"

    print("Starting litentry parachain in background ...")
    if parachain_type == "standalone":
        add_collator_ports()        
        os.environ['HEIMA_DIR'] = parachain_dir
        setup_environment(offset, parachain_dir, worker_dir)
        run(["./parachain/scripts/launch-standalone.sh"], check=True)
    elif parachain_type == "network":
        add_collator_ports()        
        os.environ['HEIMA_DIR'] = parachain_dir
        setup_environment(offset, parachain_dir, worker_dir)
        run(["./parachain/scripts/launch-network.sh", "litentry"], check=True)
    elif parachain_type == "remote":
        setup_environment(offset, "", worker_dir)
        print("Litentry parachain should be started remotely")
    else:
        sys.exit("Unsupported parachain_type")

    print("Litentry parachain is running")
    print("------------------------------------------------------------")

    c = pycurl.Curl()

    for i in range(workers_number):
        flags = get_flags(i, worker)
        subcommand_flags = get_subcommand_flags(i)
        id = "dev" if workers_number == 1 else i

        processes.append(run_worker(id, worker_dir, worker_bin, flags, subcommand_flags, log_config_path))

        print()
        # Wait a bit for worker to start up.
        sleep(5)

        idx = 0
        if "-h" in flags:
            idx = flags.index("-h") + 1
        elif "--untrusted-http-port" in flags:
            idx = flags.index("--untrusted-http-port") + 1
        else:
            print('No "--untrusted-http-port" provided in config file')
            return 0
        untrusted_http_port = flags[idx]
        url = "http://localhost:" + str(untrusted_http_port) + "/is_initialized"
        c.setopt(pycurl.URL, url)

        counter = 0
        while True:
            sleep(5)
            buffer = BytesIO()
            c.setopt(c.WRITEDATA, buffer)
            try:
                c.perform()
            except Exception as e:
                print("Try to connect to worker error: " + str(e))
                return 0

            if "I am initialized." == buffer.getvalue().decode("iso-8859-1"):
                break
            if counter >= 600:
                print("Worker initialization timeout (3000s). Exit")
                return 0
            counter += 1


    c.close()
    print("Worker(s) started!")

    # keep script alive until terminated
    signal.pause()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Run a setup consisting of a node and some workers"
    )
    parser.add_argument("-w", "--worker", type=str, default="identity", help="Worker to run: identity")
    parser.add_argument("-wn", "--workers-number", type=int, default=1, help="Number of workers to run")
    parser.add_argument(
        "-p",
        "--parachain",
        nargs="?",
        default="standalone",
        type=str,
        help="Config for parachain selection: standalone / network / remote",
    )
    parser.add_argument(
        "-l",
        "--log-config-path",
        nargs="?",
        default="./local-setup/worker-log-level-config.toml",
        type=str,
        help="log level config file path"
    )
    parser.add_argument(
        "-o", "--offset", nargs="?", default="0", type=int, help="offset for port"
    )
    args = parser.parse_args()

    today = datetime.datetime.now()
    formatted_date = today.strftime('%d_%m_%Y_%H%M')
    directory_name = f"parachain_dev_{formatted_date}"
    parachain_dir = os.path.join('/tmp', directory_name)
    print("Directory has been assigned to:", parachain_dir)

    process_list = []
    killer = GracefulKiller(process_list, args.parachain)
    if main(process_list, args.worker, args.workers_number, args.parachain, args.log_config_path, args.offset, parachain_dir) == 0:
        killer.exit_gracefully()
