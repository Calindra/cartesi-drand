import { expect, it, describe, beforeAll } from "@jest/globals";
import { fetch as cartesifyFetch, setup } from "../../src/cartesify/FetchLikeClient";
import { CartesiClientBuilder } from "../../src/main";
import { ethers } from "ethers";

describe("fetch", () => {
    const TEST_TIMEOUT = 30000
    const fetch2test = cartesifyFetch
    // const fetch2test = fetch

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

    it("should work with GET", async () => {
        const response = await fetch2test("http://127.0.0.1:8383/health")
        expect(response.ok).toBe(true)
        const json = await response.json();
        expect(json.some).toEqual('response')
    })

    it("should work with POST", async () => {
        const response = await fetch2test("http://127.0.0.1:8383/echo", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ any: 'body' })
        })
        expect(response.ok).toBe(true)
        const json = await response.json();
        expect(json).toEqual({ myPost: { any: "body" } })
    }, TEST_TIMEOUT)

    it("should work with PUT", async () => {
        const response = await fetch2test("http://127.0.0.1:8383/update", {
            method: "PUT",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ any: 'body' })
        })
        expect(response.ok).toBe(true)
        const json = await response.json();
        expect(json).toEqual({ updateBody: { any: "body" } })
    }, TEST_TIMEOUT)

    it("should work with PATCH", async () => {
        const response = await fetch2test("http://127.0.0.1:8383/patch", {
            method: "PATCH",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ any: 'body' })
        })
        expect(response.ok).toBe(true)
        const json = await response.json();
        expect(json).toEqual({ patchBody: { any: "body" } })
    }, TEST_TIMEOUT)

    it("should work with DELETE", async () => {
        const response = await fetch2test("http://127.0.0.1:8383/delete?foo=bar", {
            method: "DELETE",
        })
        expect(response.ok).toBe(true)
        const json = await response.json();
        expect(json).toEqual({ query: { foo: "bar" } })
    }, TEST_TIMEOUT)

    it("should handle 404 doing POST", async () => {
        const response = await fetch2test("http://127.0.0.1:8383/echoNotFound", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ any: 'body' })
        })
        expect(response.ok).toBe(false)
        expect(response.status).toBe(404)
        expect(await response.text()).toContain('<pre>Cannot POST /echoNotFound</pre')
    }, TEST_TIMEOUT)
})
