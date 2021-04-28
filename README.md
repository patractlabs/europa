# Europa
Europa is a sandbox environment that runs [`FRAME Contracts pallet`](https://substrate.dev/docs/en/knowledgebase/smart-contracts/contracts-pallet), 
and it is also a framework that provides a sandbox environment for [substrate](https://github.com/paritytech/substrate/) runtime. 
Europa could be used to simplify the developing, debugging, and integration test when developers develop Substrate 
runtime pallets and test smart contract (like [ink!](https://github.com/paritytech/ink), [ask!](https://github.com/patractlabs/ask) and [Solang](https://github.com/hyperledger-labs/solang)) for `FRAME Contracts pallet`. 

* **As the framework for Substrate runtime**

    When regarding this project as a lib framework, this sandbox framework already removes WASM executor, p2p, 
    consensus functions and other unnecessary parts, **just remaining the native execution environment and RPC interface**. 

    Europa sandbox framework also provides a local database, a concept of workspaces which isolates different developing environments. 

* **As the sandbox for `FRAME Contracts pallet` module for debugging and testing contracts**

    When regarding this project as an executable file which is used for running contracts, Europa provides more detail and richer
    information and wasm panic backtrace for executing contracts. Those information is very useful to help developers 
    to locate the bugs and errors in contracts, especially for the cases which multiple contracts call each other in a
    complex way, like defi or else.
    And in future, We may build it as an electron app to allow developers to download and run directly.

Riot Group for disscusion: https://app.element.io/#/room/#PatractLabsDev:matrix.org

**Note: Currently, `FRAME Contracts pallet(pallet-contract)` is under developing, which may contain some breaking changes. 
Thus we use different branch to distinguish different `FRAME Contracts pallet` version.**

*Note: We name `FRAME Contracts pallet` as `pallet-contract` in following doc.*

We provide tow branches now:

* `master`: run newest `pallet-contracts` on v2.0.0 substrate dependencies now.
* `substrate-v2.0.0`: run v2.0.0 `pallet-contracts` based on v2.0.0 substrate dependencies.

We may keep it in this way until `pallet-contracts` release v3.0.0

* `master` branch is our default branch, which provides our forked `pallet-contracts` crate that tracks the newest substrate `pallet-contracts` module.

    In `master` branch, Europa use `vender/substrate`'s `pallet-contracts` as dependency. This forked `pallet-contracts` is from 
    the branch `europa-contracts` in our `vendor/substrate` repo. In this forked `pallet-contracts` Europa provides 
    many self test features.
    
    More information about this forked substrate refers to [this repo](https://github.com/patractlabs/substrate)
    
    Currently, the tracked substrate commit is [505a8d6ae1e36b81220b7ac6999f99d14d2708af](https://github.com/paritytech/substrate/commit/505a8d6ae1e36b81220b7ac6999f99d14d2708af)
    
    For substrate change log:
    - [x] [contracts: Add missing instruction to the `Schedule`](https://github.com/paritytech/substrate/pull/7527)
    - [x] [contracts: Add `salt` argument to contract instantiation #7482](https://github.com/paritytech/substrate/pull/7482)
    - [x] [contracts: No longer put delta keys back after a failed restoration #7747](https://github.com/paritytech/substrate/pull/7747)
    - [x] [contracts: Allow runtime authors to define a chain extension #7548](https://github.com/paritytech/substrate/pull/7548)
    - [x] [contracts: Lazy storage removal #7740](https://github.com/paritytech/substrate/pull/7740)
    - [x] [contracts: Change `define_env!` to expect a `Result<T, DispatchError>` for every function #7762](https://github.com/paritytech/substrate/pull/7762)  
    - [x] [contracts: Prevent contracts from allocating a too large buffer #7818](https://github.com/paritytech/substrate/pull/7818)
    - [x] [contracts: Add configurable per-storage item cost #7819](https://github.com/paritytech/substrate/pull/7819)
    
* `substrate-v2.0.0` branch is fixed in v2.0.0 substrate, both for `pallet-contracts` module and all substrate dependencies.

    If you just need v2.0.0 contract test, do not need to clone git submodule in vendor, just switch to this branch.

Europa is tracking [newest substrate (505a8d6a)](https://github.com/paritytech/substrate/commit/505a8d6ae1e36b81220b7ac6999f99d14d2708af) now. 
Thus `pallet-contracts` could use newest features.

## Extending types
When using [Substrate Portal](https://polkadot.js.org/apps), [@polkadot/api](https://github.com/polkadot-js/api) and [Redspot](https://github.com/patractlabs/redspot) or other 3rd parties clients to connect Europa `pallet-contracts` node, please remember to add ["extending types"](https://polkadot.js.org/docs/api/start/types.extend/) for Europa requirements.

Europa **current** "extending types" is (This may be changed for different Europa version):
```json
{
  "LookupSource": "MultiAddress",
  "Address": "MultiAddress",
  "AliveContractInfo": {
    "trieId": "TrieId",
    "storageSize": "u32",
    "pairCount": "u32",
    "codeHash": "CodeHash",
    "rentAllowance": "Balance",
    "rentPayed": "Balance",
    "deductBlock": "BlockNumber",
    "lastWrite": "Option<BlockNumber>",
    "_reserved": "Option<Null>"
  }
}
```

## Features
In details, current Europa sandbox framework provides:
1. This framework is another implementation for [substrate client](https://github.com/paritytech/substrate/tree/master/client).

    Europa client crates are named `ec-*`, for Substrate client crates are named `sc-*`. Thus, Europa sandbox framework could also be used by any blockchain projects which are based on Substrate.
    
    The directory `bin/europa` is the implementation example for Europa like [`bin/node`](https://github.com/paritytech/substrate/tree/master/bin/node), [`bin/node-template`](https://github.com/paritytech/substrate/tree/master/bin/node-template) in Substrate. Substrate blockchain could integrate Europa framework for following features.  

2. Producing a block only when receive new extrinsic.
3. Removing all WASM related parts.
4. Providing another database called `state-kv` to records every block modified state.
5. Providing some particular rpc to operate this sandbox.
    * `europa_forwardToHeight`: developer can call this rpc to auto generate empty blocks to pointed height. 
    * `europa_backwardToHeight`: developer could revert block height and states to pointed height.
    * ...
    
6. Use workspace to isolate different node environment.

More information about sandbox framework detailed features refers to the doc: [basic.md](./doc/basic.md)

And for Europa `pallet-contracts` sandbox, we split into two parts:

Europa self modifications:

- [x] Using `ep-sandbox` instead of `sp-sandbox` in `pallet-contracts`.
    - [x] Using [`forked wasmi`](https://github.com/patractlabs/wasmi) to support **WASM panic backtrace**.
    - [ ] Using `wasmtime` as WASM JIT-executor and support gdb/lldb debug. (developing)
    - [ ] Using `wasm3` as a more faster WASM interpreter. (not in plan)
- [x] Supporting `NestedRuntime` event track feature to record all useful thing in `pallet-contracts`.
    When instantiate or call a contract (This contract needs to be compiled by [PatractLabs's `cargo-contract`](https://github.com/patractlabs/cargo-contract/) now), Europa would print:
    
    ```bash
    1: NestedRuntime {
        ext_result: [success] ExecReturnValue { flags: 0, data: 01 },
        caller: 0000000000000000000000000000000000000000000000000000000000000000 (5C4hrfjw...),
        self_account: 3790ddf4d8c63d559b3b46b96ca9b7b5f07b772c9ad4587eca6c0738e5d48422 (5DKZXRQN...),
        selector: 0x1e5ca456,
        args: None,
        value: 0,
        gas_limit: 4999999999999,
        gas_left: 4998334662707,
        env_trace: [
            seal_value_transferred(Some(0x00000000000000000000000000000000)),
            seal_input(Some(0x1e5ca456)),
            seal_get_storage((Some(0x0000000000000000000000000000000000000000000000000000000000000000), Some(0x01))),
            seal_return((0, Some(0x01))),
        ],
        trap_reason: TrapReason::Return(ReturnData { flags: 0, data: 01 }),
        nest: [],
    }
    ```

ChainExtensions features:

- [x] Contract Logger support, refer to this link [PIP-102](https://github.com/patractlabs/PIPs/blob/main/PIPs/pip-102.md)
- [x] Zero-Knowledge support, refer to this link [PIP-101](https://github.com/patractlabs/PIPs/blob/main/PIPs/pip-101.md)

  *Currently we use a simple static way to charge weight for ZKP, we would change this part with benchmarks result in future.*

More information about Europa `pallet-contracts` sandbox detailed features refers to the doc: [europa.md](./doc/europa.md)

## Build and run
### Build
#### clone this repo
```bash
> git clone --recurse-submodules https://github.com/patractlabs/europa.git
## or do following commands
> git clone https://github.com/patractlabs/europa.git
> cd europa/vendor
> git submodule update --init --recursive
```
If you want to use `substrate-v2.0.0` branch, do following commands:
```bash
> git clone --branch substrate-v2.0.0 https://github.com/patractlabs/europa.git
## or do following commands:
> git clone https://github.com/patractlabs/europa.git
> git checkout -t origin/substrate-v2.0.0
```

#### compile
The building for this project is same as [substrate](https://github.com/paritytech/substrate/).

When building finish, current executable file is named `europa` in `target` directory.

### Run
#### Run Europa
*Following example are built in debug mode. If you build with release mode, using `release` replace `debug` in following commands.*
```bash
$ ./target/debug/europa 
# if you what to specify a directory, add `-d` or `--base-path`
$ ./target/debug/europa -d database
# if you just want to test in tmp, add `--tmp`
$ ./target/debug/europa --tmp
```
then, the Europa sandbox is starting:
```bash
Nov 12 17:10:14.524  INFO Europa Dev Node    
Nov 12 17:10:14.524  INFO ✌️  version 0.1.0-7b4463c-x86_64-linux-gnu    
Nov 12 17:10:14.524  INFO ❤️  by patract labs <https://github.com/patractlabs>, 2020-2020    
Nov 12 17:10:14.524  INFO 📋 Chain specification: Development    
Nov 12 17:10:14.524  INFO 💾 Database: RocksDb at .sub/default/chains/dev/db    
Nov 12 17:10:14.524  INFO 📖 Workspace: default | Current workspace list: ["default"]    
Nov 12 17:10:14.524  INFO ⛓  Native runtime: europa-1 (europa-1.tx1.au1)    
Nov 12 17:10:14.986  INFO 🔨 Initializing Genesis block/state (state: 0x8fc7…d968, header-hash: 0xc7e1…7529)
Nov 12 17:10:14.988  INFO 📦 Highest known block at #0    
Nov 12 17:10:14.991  INFO Listening for new connections on 127.0.0.1:9944.   
```
#### Access Europa
now, you could use apps([https://polkadot.js.org/apps/](https://polkadot.js.org/apps/)) to access Europa:
* click left tab to switch `DEVELOPMENT` - `Local Node`.
* click `Settings` - `Developer`, and paste "extending types"(see [above](#extending-types)) to here:
* click "save"

then, you could do transfer call as normal and could see the Europa log like:
```bash
Nov 12 17:21:23.544  INFO Accepted a new tcp connection from 127.0.0.1:44210.    
Nov 12 17:21:32.238  INFO 🙌 Starting consensus session on top of parent 0xc7e1ce585807b34b7fecabe1242cafb2628c958b984ec0aee7727cdd34117529    
Nov 12 17:21:32.252  INFO 🎁 Prepared block for proposing at 1 [hash: 0x0109608217316a298c88135cf39a87cc31c37729fbe567b4a1a9f8dcdb81ebeb; parent_hash: 0xc7e1…7529; extrinsics (2): [0x2194…baf8, 0x0931…58bb]]    
Nov 12 17:21:32.267  INFO Instant Seal success: CreatedBlock { hash: 0x0109608217316a298c88135cf39a87cc31c37729fbe567b4a1a9f8dcdb81ebeb, aux: ImportedAux { header_only: false, clear_justification_requests: false, needs_justification: false, bad_justification: false, needs_finality_proof: false, is_new_best: true } }    
```

More operations please refers to the doc [basic.md](./doc/basic.md)

## Plan
1. v0.1: Have an independent runtime environment to facilitate more subsequent expansion directions. (finish)

    The independent runtime environment of excluded nodes can be expanded more without the constraints of the node environment and WASM compilation, and can be easily integrated with other components. In this version, it is more like simulating the Ganache project in Ethereum ecosystem, enabling contract developers to develop without having to build a contract blockchain. Developers can quickly fire up a personal Substrate chain, which can be used to run tests, execute commands, and inspect state while controlling how the chain operates.

2. v0.2: Modify at contract module level to provide more information. (finish)

    In this version, we will fork the pallet-contracts module for secondary development. We will strengthen the part of the error notification for contract developers, such as providing:
    * WASM stack traces, the function call stack during WASM contract execution;
    * Contracts stack traces, the call stack of a contract calling another contract;
    * Console.log, provides libraries and methods to print command lines during contract development;
    * Strengthen the error type and error display of the contract module;
    * Simple integration with Redspot; (not yet)
    
3. v0.3: Improve the development experience, strengthen collaboration with other tools, and extend the sandbox to be compatible with other runtime modules. (in future)

    * Strengthen the integration with Redspot
    * Strengthen the integration with polkadot.js.org/apps to achieve complete RPC support
    * Support status data query
