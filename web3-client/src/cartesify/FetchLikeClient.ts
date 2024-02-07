import { ContractTransactionResponse, ethers } from "ethers";
import { CartesiClient } from "../main";
import { Utils } from "../utils";
import { AxiosWrappedPromise } from "./AxiosWrappedPromise";
import { InputAddedListener } from "./InputAddedListener";

interface FetchOptions {
    method: 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE'
    body?: string
    headers?: Record<string, string>
}

let cartesiClient: CartesiClient

async function _fetch(url: string, options?: FetchOptions) {
    if (options?.method === 'GET' || options?.method === undefined) {
        return doRequestWithInspect(url, options)
    } else if (options?.method === 'POST' || options?.method === 'PUT' || options?.method === 'PATCH' || options?.method === 'DELETE') {
        return doRequestWithAdvance(url, options)
    }
    throw new Error(`Method ${options?.method} not implemented.`);
}

async function doRequestWithAdvance(url: string, options?: FetchOptions) {
    if (!cartesiClient) {
        throw new Error('You need to configure the Cartesi client')
    }
    const { logger } = cartesiClient.config;
    try {
        new InputAddedListener(cartesiClient).addListener()
        const inputContract = await cartesiClient.getInputContract();
        const requestId = `${Date.now()}:${Math.random()}`
        const wPromise = InputAddedListener.requests[requestId] = new AxiosWrappedPromise()
        // convert string to input bytes (if it's not already bytes-like)
        const inputBytes = ethers.toUtf8Bytes(
            JSON.stringify({
                requestId,
                cartesify: {
                    fetch: {
                        url,
                        options,
                    },
                },
            })
        );
        const dappAddress = await cartesiClient.getDappAddress();

        // send transaction
        const tx = await inputContract.addInput(dappAddress, inputBytes) as ContractTransactionResponse;
        await tx.wait(1);
        const resp = (await wPromise.promise) as any
        const res = new Response(resp.success)
        return res
    } catch (e) {
        logger.error(`Error ${options?.method ?? 'GET'} ${url}`, e)
        throw e
    }
}

async function doRequestWithInspect(url: string, options?: FetchOptions) {
    if (!cartesiClient) {
        throw new Error('You need to configure the Cartesi client')
    }
    const that = cartesiClient as any;
    const { logger } = that.config;

    try {
        const inputJSON = JSON.stringify({
            cartesify: {
                fetch: {
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
                const successOrError = JSON.parse(payload)
                if (successOrError.success) {
                    const response = new Response(successOrError.success)
                    return response
                } else if (successOrError.error) {
                    if (successOrError.error?.constructorName === "TypeError") {
                        throw new TypeError(successOrError.error.message)
                    } else {
                        throw successOrError.error
                    }
                }
            }
        }
        throw new Error(`Wrong inspect response format.`)
    } catch (e) {
        logger.error(e);
        throw e;
    }
    
}

export function setup(cClient: CartesiClient) {
    cartesiClient = cClient
}

export { _fetch as fetch }

class Response {

    ok: boolean = false
    status: number = 0
    type: string = ""
    headers = new Map<string, string>()
    private rawData: string
    constructor(params: any) {
        this.ok = params.ok
        this.status = params.status
        this.type = params.type
        this.rawData = params.text
        if (params.headers) {
            this.headers = new Map(params.headers)
        }
    }

    async json() {
        return JSON.parse(this.rawData)
    }

    async text() {
        return this.rawData
    }
}

