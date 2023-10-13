# Ethereum RPC &nbsp;[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/internet-computer-protocol/ic-eth-rpc/issues)

> #### Interact with [EVM blockchains](https://chainlist.org/?testnets=true) from the [Internet Computer](https://internetcomputer.org/).

---

This project is currently under active development. The canister interface may change unexpectedly, and the codebase has not yet undergone a comprehensive security audit. We will continue updating this message to reflect the current status of this repository. 

Thank you for your interest! Please feel free to [submit an issue](https://github.com/internet-computer-protocol/ic-eth-rpc/issues) if you would like to request a feature, report a bug, or start a conversation about the Ethereum RPC canister.

## Overview

**Ethereum RPC** is an Internet Computer canister smart contract for communicating with [Ethereum](https://ethereum.org/en/) and other [EVM blockchains](https://chainlist.org/?testnets=true) using an [on-chain API](./API.md). 

This canister facilitates API requests to JSON-RPC services such as [CloudFlare](https://www.cloudflare.com/en-gb/web3/), [Alchemy](https://www.alchemy.com/), or [Gateway.fm](https://gateway.fm/) using [HTTPS outcalls](https://internetcomputer.org/docs/current/developer-docs/integrations/http_requests/). This enables functionality similar to traditional Ethereum dApps, including querying Ethereum smart contract states and submitting raw transactions.

Beyond the Ethereum blockchain, this canister also supports Polygon, Avalanche, and other popular EVM networks. Check out [this webpage](https://chainlist.org/?testnets=true) for a list of all supported networks and RPC providers.

## Canisters

* Low-cost testing: [6yxaq-riaaa-aaaap-abkpa-cai](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.ic0.app/?id=6yxaq-riaaa-aaaap-abkpa-cai)

## Quick Start

Add the following to your `dfx.json` config file (replace `remote.id.ic` with any option from the list of available canisters above):

```json
{
  "canisters": {
    "eth_rpc": {
      "type": "custom",
      "candid": "https://github.com/internet-computer-protocol/ic-eth-rpc/releases/latest/download/eth_rpc.did",
      "wasm": "https://github.com/internet-computer-protocol/ic-eth-rpc/releases/latest/download/eth_rpc_dev.wasm.gz",
      "remote": {
        "id": {
          "ic": "6yxaq-riaaa-aaaap-abkpa-cai"
        }
      }
    }
  }
}
```

Run the following commands to run the canister in your local environment:

```sh
# Start the local replica
dfx start --background

# Deploy the `eth_rpc` canister
dfx deploy eth_rpc

# Call the `eth_gasPrice` JSON-RPC method
dfx canister call eth_rpc request '(variant {Url="https://cloudflare-eth.com/v1/mainnet"}, "{\"jsonrpc\":\"2.0\",\"method\":\"eth_gasPrice\",\"params\":[],\"id\":1}", 1000)' --wallet $(dfx identity get-wallet) --with-cycles 600000000
```

## Examples

### Ethereum RPC (IC mainnet)
```bash
dfx canister call eth_rpc --network ic --wallet $(dfx identity --network ic get-wallet) --with-cycles 600000000 request '(variant {Chain=0x1},"{\"jsonrpc\":\"2.0\",\"method\":\"eth_gasPrice\",\"params\":[],\"id\":1}",1000)'
```

### Ethereum RPC (local replica)
```bash
# Use a custom provider
dfx canister call eth_rpc --wallet $(dfx identity get-wallet) --with-cycles 600000000 request '(variant {Url="https://cloudflare-eth.com"},"{\"jsonrpc\":\"2.0\",\"method\":\"eth_gasPrice\",\"params\":[],\"id\":1}",1000)'
dfx canister call eth_rpc --wallet $(dfx identity get-wallet) --with-cycles 600000000 request '(variant {Url="https://ethereum.publicnode.com"},"{\"jsonrpc\":\"2.0\",\"method\":\"eth_gasPrice\",\"params\":[],\"id\":1}",1000)'

# Register your own provider
dfx canister call eth_rpc register_provider '(record { chain_id=1; base_url="https://cloudflare-eth.com"; credential_path="/v1/mainnet"; cycles_per_call=10; cycles_per_message_byte=1; })'

# Use a specific EVM chain
dfx canister call eth_rpc --wallet $(dfx identity get-wallet) --with-cycles 600000000 request '(variant {Chain=0x1},"{\"jsonrpc\":\"2.0\",\"method\":\"eth_gasPrice\",\"params\":[],\"id\":1}",1000)'
```

### Authorization (local replica)

```bash
PRINCIPAL=$(dfx identity get-principal)
dfx canister call eth_rpc authorize "(principal \"$PRINCIPAL\", variant { RegisterProvider })"
dfx canister call eth_rpc get_authorized '(variant { RegisterProvider })'
dfx canister call eth_rpc deauthorize "(principal \"$PRINCIPAL\", variant { RegisterProvider })"
```

## Contributing

Contributions are welcome! Please check out the [contributor guidelines](https://github.com/internet-computer-protocol/ic-eth-rpc/blob/main/.github/CONTRIBUTING.md) for more information.

Run the following commands to set up a local development environment:

```bash
# Clone the repository
git clone https://github.com/internet-computer-protocol/ic-eth-rpc
cd ic-eth-rpc

# Deploy to the local replica
dfx start --background
dfx deploy
```

After making source code changes, regenerate language bindings with the `generate` [npm script](https://docs.npmjs.com/cli/v10/using-npm/scripts):

```bash
npm run generate
```

## Learn More

* [How this canister works behind the scenes](https://github.com/internet-computer-protocol/ic-eth-rpc/blob/main/DeepDive.md)
* [Candid interface](https://github.com/internet-computer-protocol/ic-eth-rpc/blob/main/candid/eth_rpc.did)
* [Detailed API documentation](https://github.com/internet-computer-protocol/ic-eth-rpc/blob/main/API.md)

## Related Projects

* [IC 🔗 ETH](https://github.com/dfinity/ic-eth-starter): a full-stack starter project for calling Ethereum smart contracts from an IC dApp.
* [Bitcoin canister](https://github.com/dfinity/bitcoin-canister): interact with the Bitcoin blockchain from the Internet Computer.
