# Web3 Client

## Introduction

Web3 Client is a powerful tool that allows you to interact with the Cartesi Machine. It enables you to send transactions, query data, and interact with your backend seamlessly.

## Getting Started

### Installation

To use our web3 client, follow these simple installation steps:

```shell
npm install @calindra/web3-client
```

### Usage

1. Import and configure the Web3 library into your project:

   ```ts
   import { type CartesiClient, CartesiClientBuilder } from "@calindra/web3-client";

   const CARTESI_INSPECT_ENDPOINT="http://localhost:8080/inspect";

   // replace with the content of your dapp address (it could be found on dapp.json)
   const DAPP_ADDRESS="0x70ac08179605AF2D9e75782b8DEcDD3c22aA4D0C";

   const cartesiClient: CartesiClient = new CartesiClientBuilder()
    .withEndpoint(CARTESI_INSPECT_ENDPOINT)
    .withLogger({
      info: console.log,
      error: console.error,
    })
    .withDappAddress(DAPP_ADDRESS)
    .build();
    ```
2. When available set the provider and the signer
    ```ts
    cartesiClient.setProvider(provider);
    cartesiClient.setSigner(signer);
   ```

3. Start interacting with the Cartesi Machine:

   ```ts
   const payload = { foo: "bar" };

   // send an advance command
   cartesiClient.advance(payload);

   // send an inspect command
   cartesiClient.inspect(payload);
   ```

## Examples

Here are some examples to get started:

### Sending a Transaction

Example: Sending an advance input
```ts
const payload = { "action": "start_game", "game_id": "123" };
await cartesiClient.advance(payload);
```

### Querying Data

Example: Getting the player
```ts
const payload = { "action": "show_player", "address": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266" };
const player = await cartesiClient.inspect(payload);
```
