import { ContractTransactionResponse, ethers } from "ethers";
import { CartesiClient } from "../main";
import { Utils } from "../utils";
import { AxiosWrappedPromise } from "./AxiosWrappedPromise";
import { AxiosLikeClient } from "./AxiosLikeClient";

interface FetchOptions {
    method: 'GET' | 'POST'
    body?: string
    headers?: Record<string, string>
}

let cartesiClient: CartesiClient

async function _fetch(url: string, options?: FetchOptions) {
    if (options?.method === 'GET' || options?.method === undefined) {
        return doGet(url, options)
    } else if (options?.method === 'POST') {
        console.log('Doing post')
        return doPost(url, options)
    }
    throw new Error("Function not implemented.");
}

async function doPost(url: string, options?: FetchOptions) {
    if (!cartesiClient) {
        throw new Error('You need to configure the Cartesi client')
    }
    const { logger } = cartesiClient.config;

    try {
        new AxiosLikeClient(cartesiClient).addListener()
        const { provider, signer } = cartesiClient.config;
        logger.info("getting network", provider);
        const network = await provider.getNetwork();
        logger.info("getting signer address", signer);
        const signerAddress = await signer.getAddress();

        logger.info(`connected to chain ${network.chainId}`);
        logger.info(`using account "${signerAddress}"`);

        // connect to rollup,
        const inputContract = await cartesiClient.getInputContract();

        const data = options?.body || ''
        // use message from command line option, or from user prompt
        logger.info(`sending "${JSON.stringify(data)}"`);

        const requestId = `${Date.now()}:${Math.random()}`
        const wPromise = AxiosLikeClient.requests[requestId] = new AxiosWrappedPromise()
        // convert string to input bytes (if it's not already bytes-like)
        const inputBytes = ethers.toUtf8Bytes(
            JSON.stringify({
                requestId,
                cartesify: {
                    axios: {
                        data: JSON.parse(data),
                        url,
                        method: "POST"
                    },
                },
            })
        );

        const dappAddress = await cartesiClient.getDappAddress();
        logger.info(`dappAddress: ${dappAddress} typeof ${typeof dappAddress}`);

        // send transaction
        const tx = await inputContract.addInput(dappAddress, inputBytes) as ContractTransactionResponse;
        logger.info(`transaction: ${tx.hash}`);
        logger.info("waiting for confirmation...");
        const receipt = await tx.wait(1);
        logger.info(JSON.stringify(receipt));
        const resp = await wPromise.promise
        return new Response(JSON.stringify({ success: { data: resp.data } }))
    } catch (e) {
        logger.error(e);
        if (e instanceof Error) {
            throw e;
        }
        throw new Error("Error on advance");
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

        logger.info("Inspecting endpoint: ", urlInner.href);

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
                return new Response(payload)
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

    constructor(private rawData: string) {

    }

    async json() {
        const resp = JSON.parse(this.rawData)
        return resp.success.data
    }
}

