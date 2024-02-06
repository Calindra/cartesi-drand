import { ContractTransactionResponse, ethers } from "ethers";
import { CartesiClient } from "../main";
import { Utils } from "../utils";
import { AxiosWrappedPromise } from "./AxiosWrappedPromise";
import { AxiosLikeClient } from "./AxiosLikeClient";

interface FetchOptions {
    method: 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE'
    body?: string
    headers?: Record<string, string>
}

let cartesiClient: CartesiClient

async function _fetch(url: string, options?: FetchOptions) {
    if (options?.method === 'GET' || options?.method === undefined) {
        return doGet(url, options)
    } else if (options?.method === 'POST' || options?.method === 'PUT' || options?.method === 'PATCH' || options?.method === 'DELETE') {
        return doRequestWithAdvance(url, options)
    }
    throw new Error("Function not implemented.");
}

async function doRequestWithAdvance(url: string, options?: FetchOptions) {
    if (!cartesiClient) {
        throw new Error('You need to configure the Cartesi client')
    }
    const { logger } = cartesiClient.config;
    try {
        new AxiosLikeClient(cartesiClient).addListener()
        const inputContract = await cartesiClient.getInputContract();
        const data = options?.body
        const requestId = `${Date.now()}:${Math.random()}`
        const wPromise = AxiosLikeClient.requests[requestId] = new AxiosWrappedPromise()
        // convert string to input bytes (if it's not already bytes-like)
        const inputBytes = ethers.toUtf8Bytes(
            JSON.stringify({
                requestId,
                cartesify: {
                    axios: {
                        data: data ? JSON.parse(data) : undefined,
                        url,
                        method: options?.method
                    },
                },
            })
        );
        const dappAddress = await cartesiClient.getDappAddress();

        // send transaction
        const tx = await inputContract.addInput(dappAddress, inputBytes) as ContractTransactionResponse;
        await tx.wait(1);
        const resp = await wPromise.promise
        const res = new Response(JSON.stringify({ success: { data: resp.data } }))
        res.ok = true
        return res
    } catch (e) {
        logger.error(e);
        if (e instanceof Error) {
            throw e;
        }
        const res = new Response('')
        res.ok = false
        res.status = (e as any).status
        return res
    }
}

async function doGet(url: string, options?: FetchOptions) {
    if (!cartesiClient) {
        throw new Error('You need to configure the Cartesi client')
    }
    const that = cartesiClient as any;
    const { logger } = that.config;

    try {
        const inputJSON = JSON.stringify({
            cartesify: {
                axios: {
                    url,
                    method: "GET"
                },
            },
        });
        const jsonEncoded = encodeURIComponent(inputJSON);
        const urlInner = new URL(that.config.endpoint);
        urlInner.pathname += `/${jsonEncoded}`;
        const response = await fetch(urlInner.href, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
            },
        });
        const result: unknown = await response.json();

        if (Utils.isObject(result) && "reports" in result && Utils.isArrayNonNullable(result.reports)) {
            const firstReport = result.reports.at(0);
            if (Utils.isObject(firstReport) && "payload" in firstReport && typeof firstReport.payload === "string") {
                const payload = Utils.hex2str(firstReport.payload.replace(/^0x/, ""));
                const response = new Response(payload)
                response.ok = true
                return response
            }
        }
    } catch (e) {
        logger.error(e);
    }
    return new Response('')
}

export function setup(cClient: CartesiClient) {
    cartesiClient = cClient
}

export { _fetch as fetch }

class Response {

    ok: boolean = false
    status: number = 0
    constructor(private rawData: string) {
    }

    async json() {
        const resp = JSON.parse(this.rawData)
        return resp.success.data
    }
}

