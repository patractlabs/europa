# Europa
Europa is a sandbox environment that runs [`FRAME Contracts pallet`](https://substrate.dev/docs/en/knowledgebase/smart-contracts/contracts-pallet), and it is also a framework that provides a sandbox environment for [substrate](https://github.com/paritytech/substrate/) runtime. Europa could be used to simplify the developing, debugging, and integration test when developers develop Substrate runtime pallets and test smart contract for `FRAME Contracts pallet`. 

The sandbox framework already removes WASM executor, p2p, consensus functions and other unnecessary parts, just remaining the native execution environment and RPC interface. 

Europa sandbox framework also provides a local database, a detailed log print function, a concept of workspaces which isolates different developing environments. 

Regarding Europa as a sandbox with `FRAME Contracts pallet`, We would provide more debugging  features in v0.2 Europa, refers to [Patract Hub's treasury proposal for Europa (sandbox) v0.2](https://polkadot.polkassembly.io/post/189), and we may build it as an electron app to allow developers to download and run directly in future.

Riot Group for disscusion: https://app.element.io/#/room/#PatractLabsDev:matrix.org

**Note: Because currently `FRAME Contracts pallet(pallet-contract)` is under developing, may contains some breaking changes. Thus we use branch to distinguish different features.**

*Note: We name `FRAME Contracts pallet` as `pallet-contract` in following doc.*

We provide tow branches now:

* `master`: run newest `pallet-contracts` on v2.0.0 substrate dependencies now.
* `substrate-v2.0.0`: run v2.0.0 `pallet-contracts` based on v2.0.0 substrate dependencies.

We may keep it in this way until `pallet-contracts` release v3.0.0
* `master` branch is our default branch, which provides our forked `pallet-contracts` crate that tracks the newest substrate `pallet-contracts` module.
    We provide our forked `pallet-contracts` in `vendor` directory which tracks a pointed version for substrate. This forked `pallet-contracts` is from 
    the branch `europa-contracts` in our substrate repo. In this forked `pallet-contracts` we would provide many self test features.
    
    More information about this forked substrate refers to [this repo](https://github.com/patractlabs/substrate)
    
    Currently, the tracked substrate commit is [b27503591d019b94a0eea7510578dadc5ad3196c](https://github.com/paritytech/substrate/commit/b27503591d019b94a0eea7510578dadc5ad3196c)
    
    For substrate change log:
    - [x] [contracts: Add missing instruction to the `Schedule`](https://github.com/paritytech/substrate/pull/7527)
    - [x] [contracts: Add `salt` argument to contract instantiation #7482](https://github.com/paritytech/substrate/pull/7482)
    - [x] [contracts: No longer put delta keys back after a failed restoration #7747](https://github.com/paritytech/substrate/pull/7747)
    - [x] [contracts: Allow runtime authors to define a chain extension #7548](https://github.com/paritytech/substrate/pull/7548)
    - [x] [contracts: Lazy storage removal #7740](https://github.com/paritytech/substrate/pull/7740)
    - [x] [contracts: Change `define_env!` to expect a `Result<T, DispatchError>` for every function #7762](https://github.com/paritytech/substrate/pull/7762)  
    - [x] [contracts: Prevent contracts from allocating a too large buffer #7818](https://github.com/paritytech/substrate/pull/7818)
    - [x] [contracts: Add configurable per-storage item cost #7819](https://github.com/paritytech/substrate/pull/7819)

    For our change log:
    
    (Not yet now)

* `substrate-v2.0.0` branch is fixed in v2.0.0 substrate, both for `pallet-contracts` module and all substrate dependencies.

    If you just need v2.0.0 contract test, do not need to clone git submodule in vendor, just switch to this branch.

Europa is tracking [newest substrate (b27503)](https://github.com/paritytech/substrate/commit/b27503591d019b94a0eea7510578dadc5ad3196c) now. Thus `pallet-contracts` could use newest features.

## Extending types
When using [Substrate Portal](https://polkadot.js.org/apps), [@polkadot/api](https://github.com/polkadot-js/api) and [Redspot](https://github.com/patractlabs/redspot) or other 3rd parties client to connect Europa `pallet-contracts` node, please remember to add ["extending types"](https://polkadot.js.org/docs/api/start/types.extend/) for Europa requirements.

Europa **current** "extending types" is (This may be changed for different Europa version):
```json
{
  "LookupSource": "MultiAddress"
}
```
    
## Features
In details, current Europa provide:
1. Europa sandbox framework is another implementation for [substrate client](https://github.com/paritytech/substrate/tree/master/client).

    Europa client crate is named `ec-*`, for Substrate client crate is named `sc-*`. Thus, Europa sandbox framework could also be used by any blockchain projects which are based on Substrate.
    
    The directory `bin/europa` is the implementation example for Europa like [`bin/node`](https://github.com/paritytech/substrate/tree/master/bin/node), [`bin/node-template`](https://github.com/paritytech/substrate/tree/master/bin/node-template) in Substrate. Substrate blockchain could integrate Europa to provide following features.  

2. Produce a block only when receive new extrinsic.

    Europa sandbox framework uses `sc-consensus-manual-seal` to produce blocks. So that when debugging contracts or runtime pallet developers do not need to wait the interval time for consensus. Currently once receiving an extrinsic would produce a new block. In the future, we would provide a rpc to allow developers to submit a batch of extrinsics and produce one block to contains this batch of extrinsics. 

3. Remove all WASM related part.

    We think runtime `no-std` could not debug easily, like set breaking point for gdb debug and add log print for any formatted printing (Substrate provide a log in runtime, but it's limited). And when someone wants to integrate experimental features in runtime to verify them which do not support WASM yet, Europa provides a friendly environment.
    
    e.g. We would fork `wasmi` and change it a lot to provide more features. Those features are just used for testing contracts, not used in production blockchain. 
    
4. Provide another database called `state-kv` to records every block modified state.

    The sandbox framework could export modified state kvs for every block, including state kvs and child state kvs. Currently Europa just provides a way to export all state for a specified block state, but for debugging, we just need to know the changed state after executing a block.
    
    [`substrate-archive`](https://github.com/paritytech/substrate-archive) provides a way to store the modified state kvs by constructing an outside executing environment to do this thing. In Europa, we directly store the modified state kvs for every block in `state-kv` db, so developer could easily lookup those changed state kvs.
    
    ```bash
    # print the modified state kvs for block 1
    $ ./target/debug/europa state-kv 1
    Nov 12 15:53:27.699  INFO modified state for block:0x6c119a8f7de42e330aca8b9d3587937aacbbc203cc21650b60644c2f2d33e7fb    
    Nov 12 15:53:27.699  INFO       key:26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac|value:[DELETED]    
    Nov 12 15:53:27.699  INFO       key:26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850|value:05000000
    # ...    
    ```
   
5. Provide some particular rpc to operate this sandbox.

    Currently Europa framework is compatible with all rpc in Substrate except the rpc related to network (like `system_networkState` and others). Thus the node which is integrated with Europa framework could use Substrate Protal: [https://polkadot.js.org/apps/](https://polkadot.js.org/apps/) to operate directly.
    
    Besides, Europa provides following rpc:
    * `europa_forwardToHeight` (params: \[`height: NumberOf<B>`\])
    
        The rpc provide a way to produce a batch of empty block to reach target block height. This rpc is very useful when testing a feature related to block height like staking locking time, the rent calculating in `pallet-contracts` and so on. 
    
        e.g. Currently best height is 100, and when call this rpc and pass 199, the best height would reach to 199.  
    
    * `europa_backwardToHeight` (params: \[`height: NumberOf<B>`\])
    
        The rpc could revert current best height to the specified height which is less than current best height. This rpc provide a way to back to an old state to do testing repeatedly. So that when debugging, there is no need to construct the testing state environment every time, just using this rpc to revert to an old state.
        
        e.g. Currently best height is 100, and when call this rpc and pass 88, the best height would back to 88 and the state is reverted to 88 block state.
    
    * `europa_modifiedStateKvs` (params: \[`number_or_hash: NumberOrHash<B>`\])
    
        The rpc could print the modified state kvs for a specified block height or hash. This rpc is same to the feature 4, just using rpc to return the information.
        
        ```json
        {
            "jsonrpc": "2.0",
            "result": {
                "0x26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac": null,
                "0x26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850": "0x05000000",
                // ...
            }
        }
        ```
    
6. Use workspace to isolate different node environment.
   
    Europa sandbox framework provides the concept of workspace to isolate node environment. In Substrate, developer could use command `-d/--base-path` to isolate different data environment. 
    
    We think `-d/--base-path` should be used for assigning the workspace directory for Europa, and using `-w/--workspace` command to specify a separate environment. And on the other hand, all existed workspaces would be recorded, developer could switch between different workspace.  

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

#### Export modified state kvs
```bash
$ ./target/debug/europa state-kv 1
# if you have specified a directory, add `-d` or `--base-path`
$ ./target/debug/europa state-kv -d database 1
```

#### Use another workspace
##### Specify another workspace
```bash
$ ./target/debug/europa -w another-workspace
# if you have specified a directory, add `-d` or `--base-path`
$ ./target/debug/europa -d database -w another-workspace
```
the log would like:
```bash
Nov 12 17:25:47.428  INFO 💾 Database: RocksDb at .sub/another-workspace/chains/dev/db    
Nov 12 17:25:47.428  INFO 📖 Workspace: another-workspace | Current workspace list: ["default", "another-workspace"]    
Nov 12 17:25:47.428  INFO ⛓  Native runtime: europa-1 (europa-1.tx1.au1)  
```

##### Set default workspace
stop the Europa, than execute:
```bash
# another-workspace is the workspace name which we what to set as default.
$ ./target/debug/europa workspace default another-workspace
Nov 12 17:28:41.980  INFO Current default workspace:    
Nov 12 17:28:41.981  INFO       default    
Nov 12 17:28:41.981  INFO     
Nov 12 17:28:41.981  INFO Set [another-workspace] as default workspace.  
```
then start Europa, Europa would use "another-workspace" as default workspace.
```bash
$ ./target/debug/europa
# ...
Nov 12 17:29:33.862  INFO 💾 Database: RocksDb at .sub/another-workspace/chains/dev/db    
Nov 12 17:29:33.862  INFO 📖 Workspace: another-workspace | Current workspace list: ["default", "another-workspace"]    
Nov 12 17:29:33.862  INFO ⛓  Native runtime: europa-1 (europa-1.tx1.au1)    
```

##### Delete workspace
```bash
$ ./target/debug/europa workspace delete another-workspace
Nov 12 17:30:49.549  INFO Current default workspace:    
Nov 12 17:30:49.549  INFO       another-workspace    
Nov 12 17:30:49.549  INFO     
Nov 12 17:30:49.550  INFO Delete workspace [another-workspace].    
Nov 12 17:30:49.550  INFO       delete default record: [another-workspace]    
Nov 12 17:30:49.550  INFO       delete workspace:[another-workspace] from workspace list
```

## Plan
1. v0.1: Have an independent runtime environment to facilitate more subsequent expansion directions.

    The independent runtime environment of excluded nodes can be expanded more without the constraints of the node environment and WASM compilation, and can be easily integrated with other components. In this version, it is more like simulating the Ganache project in Ethereum ecosystem, enabling contract developers to develop without having to build a contract blockchain. Developers can quickly fire up a personal Substrate chain, which can be used to run tests, execute commands, and inspect state while controlling how the chain operates.

2. v0.2: Modify at contract module level to provide more information

    In this version, we will fork the pallet-contracts module for secondary development. We will strengthen the part of the error notification for contract developers, such as providing:
    * WASM stack traces, the function call stack during WASM contract execution;
    * Contracts stack traces, the call stack of a contract calling another contract;
    * Console.log, provides libraries and methods to print command lines during contract development;
    * Strengthen the error type and error display of the contract module;
    * Simple integration with Redspot;
    
3. v0.3: Improve the developement experience, strengthen collaboration with other tools, and extend the sandbox to be compatible with other runtime modules

    * Strengthen the integration with Redspot
    * Strengthen the integration with polkadot.js.org/apps to achieve complete RPC support
    * Support status data query
