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

   [`substrate-archive`](https://github.com/paritytech/substrate-archive) provides a way to store the modified state kvs by constructing an outside executing environment to do this thing. In Europa, we directly store the modified state kvs for every block in `state-kv` db, so developers could easily lookup those changed state kvs.

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

   * `europa_extrinsicStateChanges` (params: \[`number_or_hash: NumberOrHash<B>`, `index: u32` \])
   
      The rpc can return the changed state for an extrinsic. The return type shows by "Action" and "Value".
   
      All is in 6 type now:
   
      ```rust
      pub enum Event {
         Put(Put),  // modify or delete a value.
         PutChild(PutChild), // modify or delete a child value.
         KillChild(KillChild), // remove all storage for a child.
         ClearPrefix(ClearPrefix), // remove all matched value for a prefix.
         ClearChildPrefix(ClearChildPrefix), // remove all matched value for a prefix in a child storage.
         Append(Append), // appended value for a key (e.g. for System::Events)
      }
      ```
   
      The `Put`, `PutChild` and others are "Action". Different Actions contain different type value. 
      The definitions for the Value type can be found in `client/basic-authorship/src/block_tracing/mod.rs`
     
      The return example is following. The value part is "data", and the action part is "type":
     
      ```json
      [
         {
            "data": {
               "key": "0x32366161333934656561353633306530376334386165306339353538636566376138366461356139333236383466313939353339383336666362386338383666",
               "value": "0x3635643432623030"
            },
            "type": "Put"
         },
        // ...
      ] 
      ```

6. Use workspace to isolate different node environment.

   Europa sandbox framework provides the concept of workspace to isolate node environment. In Substrate, developers could use command `-d/--base-path` to isolate different data environment.

   We think `-d/--base-path` should be used for assigning the workspace directory for Europa, and using `-w/--workspace` command to specify a separate environment. And on the other hand, all existed workspaces would be recorded, developers could switch between different workspace.  

### Europa framework operations
#### Run

Running the following command could run the europa node directly.

```bash
$ ./target/debug/europa
```

Same to substrate, the node data is stored in:
* Linux: `$XDG_DATA_HOME/europa`or`~/.local/share/europa`
* macOS: `$HOMELibrary/Application Support/europa`
* Windows: `{FOLDERID_LocalAppData}\europa\data`

(In your project, europa would be replaced by your project name.) 

We advice developers use command `-d` or `--base-path` to assign a specific path for data

```bash
$ ./target/debug/europa -d <data_path>
```

Or developers aware that he just need temp store data and drop later, he could use `--tmp` instead of `-d/--base-path`.
**But notice if specify `--tmp`, the command `--workspace` is useless**. (for the data directory would be dropped when node shutdown):

```bash
$ ./target/debug/europa --tmp
```

#### Export modified state kvs
The exported k/v state could help developers to analyse the final changed data for this block, judging whether the execution
result is matching with expectation, counting changed data for a specified key or other situations.

Currently Europa just could export the raw hex k/v, in the future, we could design other tools to debug the raw k/v to specific value.

If developers want to export the k/v state for a block, he could do the following command.
```bash
# block number is 1, export state for height 1
$ ./target/debug/europa state-kv 1
# if you have specified a directory, add `-d` or `--base-path`
$ ./target/debug/europa state-kv -d database 1
# or use block hash instead of block number
$ ./target/debug/europa state-kv 0x6c119a8f7de42e...
```
**This command could run while the node is running**. The output log is like:
```bash
$ ./target/debug/europa state-kv 0x6c119a8f7de42e330aca8b9d3587937aacbbc203cc21650b60644c2f2d33e7fb
2021-01-12 14:56:07  modified state for block:0x6c119a8f7de42e330aca8b9d3587937aacbbc203cc21650b60644c2f2d33e7fb    
2021-01-12 14:56:07  	key:26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac|value:[DELETED]    
2021-01-12 14:56:07  	key:26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850|value:05000000
# ...  
```
`key` means the hex-like value for state key and `value` means the hex-like state value for this key. If this value is 
deleted in this block, the value is marked as `[DELETED]` to distinguish the encode value of type `()`.

#### Use another workspace

Workspace is used for isolating different spaces to store data in same directory. This is useful to test different
contracts or use Europa as the backend for contract integration testing.

Though developers can use `-d/--base-path` to accomplish same thing, but they need to pay more to manage the different paths.
Workspace concept provides a simplified method to do this thing.

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
