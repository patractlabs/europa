# europa
Europa is a sandbox of [Substrate](https://github.com/paritytech/substrate/) runtime environment, which would be used to simplify the developing, debugging, and integration test when developers develop Substrate runtime applications and test pallet-contracts. 
The sandbox will remove p2p and consensus functions, just remaining the execution environment and RPC interface. We will wrap the core modules, then provide a local database, a detailed log print function, a concept of workspaces which isolates different developing environments, and a front-end UI to interact with this sandbox and so on. 

In the first phase, Europa would just integrate the pallet-contracts module. In the second phase, we would refactor the sandbox to be more general that could let developers to add their runtime module and test.

Riot Group for disscusion: https://app.element.io/#/room/#PatractLabsDev:matrix.org
