## Europa sandbox framework
### Europa framework features
In details, current Europa sandbox framework provides:
1. This framework is another implementation for [substrate client](https://github.com/paritytech/substrate/tree/master/client).

   Europa client crates are named `ec-*`, for Substrate client crates are named `sc-*`. Thus, Europa sandbox framework could also be used by any blockchain projects which are based on Substrate.

   The directory `bin/europa` is the implementation example for Europa like [`bin/node`](https://github.com/paritytech/substrate/tree/master/bin/node), [`bin/node-template`](https://github.com/paritytech/substrate/tree/master/bin/node-template) in Substrate. Substrate blockchain could integrate Europa framework for following features.

2. Producing a block only when receive new extrinsic.

   Europa sandbox framework uses `sc-consensus-manual-seal` to produce blocks. So that when debugging contracts or runtime pallet,
   developers do not need to wait the interval time for consensus. Currently once receiving an extrinsic would produce a new block. In the future, we would provide a rpc to allow developers to submit a batch of extrinsics and produce one block to contains this batch of extrinsics.
   In the feature, framework would provide a rpc to submit a batch of extrinsics at once.

3. Removing all WASM related parts.

   We think runtime `no-std` could not debug easily, like set breaking point for gdb debug and add log print for any formatted printing (Substrate provide a log in runtime, but it's limited). And when someone wants to integrate experimental features in runtime to verify them which do not support WASM yet, Europa provides a friendly environment.

   e.g. We would fork `wasmi` and change it a lot to provide more features. Those features are just used for testing contracts, not used in production blockchain.

4. Providing another database called `state-kv` to records every block modified state.

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

5. Providing some particular rpc to operate this sandbox.

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
