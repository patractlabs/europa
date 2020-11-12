# Europa
Europa is a sandbox framework of [substrate](https://github.com/paritytech/substrate/) runtime environment, which would be used to simplify the developing, debugging, and integration test when developers develop substrate runtime pallets and test pallet-contracts. 
The sandbox framework will remove WASM executor, p2p, consensus functions and other unnecessary parts, just remaining the native execution environment and RPC interface. 

We also provide a local database, a detailed log print function, a concept of workspaces which isolates different developing environments, and maybe become an electron wrapper app in future.

Riot Group for disscusion: https://app.element.io/#/room/#PatractLabsDev:matrix.org

## Features
In details, current europa provide:
1. Europa sandbox framework is another implementation for [substrate client](https://github.com/paritytech/substrate/tree/master/client).

    For substrate client crate is named `sc-*`, europa client crate is named `ec-*`. Thus, europa sandbox framework could also be used by any blockchain projects which based on substrate.
    
    The directory `bin/europa` is the implementation example for europa like [`bin/node`](https://github.com/paritytech/substrate/tree/master/bin/node), [`bin/node-template`](https://github.com/paritytech/substrate/tree/master/bin/node-template) in substrate. Substrate blockchain could integrate europa to provide following features.  

2. Produce a block only when receive new extrinsic.

    Europa sandbox framework uses `sc-consensus-manual-seal` to produce blocks. So that when debugging contracts or runtime pallet developers do not need to wait the interval time for consensus. Currently once receiving an extrinsic would produce a new block. In the future, we would provide a rpc to allow developers to submit a batch of extrinsics and produce one block to contains this batch of extrinsics. 

3. Remove all WASM related part.

    We think runtime `no-std` could not easily to debug, like set breaking point for gdb debug and add log print for any formatted printing (though substrate provide a log in runtime, it's limited). And when someone what to integrate experimental features in runtime to verify them which do not support WASM yet, europa provide a friendly environment.
    
    e.g. We would fork `wasmi` and change it a lot to provide more features. Those features are just used for testing contracts, not used in production blockchain. 
           
4. Provide another database called `state-kv` to records every block modified state.

    The sandbox framework could export every block modified state kvs, include state kvs and child state kvs. Currently europa just provide a way to export all state for a specified block state, but for debugging, we just need to know the changed state after executing a block.
    
    [`substrate-archive`](https://github.com/paritytech/substrate-archive) provides a way to store the modified state kvs by constructing an outside executing environment to do this thing. In europa, we directly store the modified state kvs for every block in `state-kv` db, so developer could easily lookup those changed state kvs.
    
    ```bash
    # print the modified state kvs for block 1
    $ ./target/debug/europa state-kv 1
    Nov 12 15:53:27.699  INFO modified state for block:0x6c119a8f7de42e330aca8b9d3587937aacbbc203cc21650b60644c2f2d33e7fb    
    Nov 12 15:53:27.699  INFO       key:26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac|value:[DELETED]    
    Nov 12 15:53:27.699  INFO       key:26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850|value:05000000
    # ...    
    ```
   
5. Provide some particular rpc to operate this sandbox.

    Currently europa framework is compatible with all rpc in substrate except the rpc related to network (like `system_networkState` and others). Thus the node which is integrated with europa framework could use apps: [https://polkadot.js.org/apps/](https://polkadot.js.org/apps/) to operate directly.
    
    Besides, europa provides following rpc:
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
    
    Europa sandbox framework provide the concept of workspace to isolate node environment. In substrate, developer could use command `-d/--base-path` to isolate different data environment. 
    
    We think `-d/--base-path` should be used for assigning the workspace directory for europa, and using `-w/--workspace` command to specify a separate environment. And on the other hand, all existed workspaces would be recorded, developer could switch between different workspace.  

## Build and run
### Build
The building for this project is same as [substrate](https://github.com/paritytech/substrate/).

When building finish, current executable file in `target` directory is named `europa`.

### Run
#### Run europa
*Building in debug mode. If build with release mode, using `release` replace `debug` in following commands.*
```bash
$ ./target/debug/europa 
# if what to specify a directory, add `-d` or `--base-path`
$ ./target/debug/europa -d database
```
then, the europa sandbox is starting:
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
#### Access europa
now, you could use apps([https://polkadot.js.org/apps/](https://polkadot.js.org/apps/)) to access europa:
* click left tab to switch `DEVELOPMENT` - `Local Node`.
* click `Settings` - `Developer`, and parse:
    ```json
    {
      "Address": "AccountId",
      "LookupSource": "AccountId"
    }
    ```
* click "save"
then, you could do a transfer normally and could see the europa log like:
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
stop the europa, the execute:
```bash
# another-workspace is the workspace name which we what to set as default.
$ ./target/debug/europa workspace default another-workspace
Nov 12 17:28:41.980  INFO Current default workspace:    
Nov 12 17:28:41.981  INFO       default    
Nov 12 17:28:41.981  INFO     
Nov 12 17:28:41.981  INFO Set [another-workspace] as default workspace.  
```
then start europa, would use "another-workspace" as default workspace.
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
