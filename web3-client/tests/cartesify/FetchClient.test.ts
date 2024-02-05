import { expect, it, describe, beforeAll } from "@jest/globals";
import { fetch as cFetch, setup } from "../../src/cartesify/FetchLikeClient";
import { CartesiClientBuilder } from "../../src/main";
import { ethers } from "ethers";

describe("fetch", () => {
    const tFetch = cFetch
    // const tFetch = fetch

    beforeAll(() => {
        const endpoint = new URL("http://localhost:8080/inspect");
        const provider = ethers.getDefaultProvider("http://localhost:8545");
        const cartesiClient = new CartesiClientBuilder()
            .withDappAddress('0x70ac08179605AF2D9e75782b8DEcDD3c22aA4D0C')
            .withEndpoint(endpoint)
            .withProvider(provider)
            .build();
        const privateKey = '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'
        let walletWithProvider = new ethers.Wallet(privateKey, provider);
        cartesiClient.setSigner(walletWithProvider)
        setup(cartesiClient)
    })

    it("should works with GET", async () => {
        const response = await tFetch("http://127.0.0.1:8383/health")
        const json = await response.json();
        expect(json.some).toEqual('response')
    })

    it("should works with POST", async () => {
        const response = await tFetch("http://127.0.0.1:8383/echo", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ any: 'body' })
        })
        const json = await response.json();
        expect(json).toEqual({ myPost: { any: "body" } })
    }, 30000)
})
