import { CartesiClient } from "../main";
import { Utils } from "../utils";
import { AxiosWrappedPromise } from "./AxiosWrappedPromise";
import debug from "debug";

/**
 * to see the logs run on terminal:
 * ```
 * export DEBUG=cartesify:*
 * ```
 */
const debugs = debug('cartesify:InputAddedListener')

let listenerAdded = false

const query = `query Report($index: Int!) {
    input(index: $index) {
        reports(last: 1) {
            edges {
                node {
                    payload
                }
            }
        }
    }
}`

const defaultOptions: RequestInit = {
    "headers": {
        "accept": "*/*",
        "accept-language": "en-US,en;q=0.9,pt;q=0.8",
        "content-type": "application/json",
        "sec-ch-ua": "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Microsoft Edge\";v=\"120\"",
        "sec-ch-ua-mobile": "?0",
        "sec-ch-ua-platform": "\"macOS\"",
        "sec-fetch-dest": "empty",
        "sec-fetch-mode": "cors",
        "sec-fetch-site": "same-origin"
    },
    "referrerPolicy": "strict-origin-when-cross-origin",
    "method": "POST",
    "mode": "cors",
    "credentials": "omit",
}



export class InputAddedListener {

    static requests: Record<string, AxiosWrappedPromise> = {}

    endpointGraphQL: URL

    constructor(private cartesiClient: CartesiClient) {
        this.endpointGraphQL = cartesiClient.config.endpointGraphQL
    }

    async addListener() {
        const MAX_RETRY = 30
        const cartesiClient = this.cartesiClient;
        if (!cartesiClient) {
            throw new Error('You need to configure the Cartesi client')
        }
        if (listenerAdded) {
            return
        }
        listenerAdded = true
        const contract = await cartesiClient.getInputContract()
        contract.on("InputAdded", async (_dapp, inboxInputIndex, _sender, input) => {
            const start = Date.now()
            let attempt = 0;
            try {
                const payload = Utils.hex2str2json(input)
                const wPromise = InputAddedListener.requests[payload.requestId]
                if (!wPromise) {
                    return
                }
                while (attempt < MAX_RETRY) {
                    try {
                        attempt++;
                        if (attempt > 1) {
                            debugs(`waiting 1s to do the ${attempt} attempt.`)
                            await new Promise((resolve) => setTimeout(resolve, 1000))
                        }
                        const req = await fetch(this.endpointGraphQL, {
                            ...defaultOptions,
                            referrer: `${this.endpointGraphQL.toString()}`,
                            body: JSON.stringify({
                                query,
                                operationName: null,
                                variables: { index: inboxInputIndex.toString() }
                            }),
                        });
                        const json = await req.json()
                        if (json.data?.input.reports.edges.length > 0) {
                            const lastEdge = json.data.input.reports.edges.length - 1
                            const hex = json.data.input.reports.edges[lastEdge].node.payload
                            const successOrError = Utils.hex2str2json(hex)
                            if (successOrError.success) {
                                wPromise.resolve!(successOrError)
                            } else {
                                if (successOrError.error?.constructorName === "TypeError") {
                                    const typeError = new TypeError(successOrError.error.message)
                                    wPromise.reject!(typeError)
                                } else {
                                    wPromise.reject!(successOrError.error)
                                }
                            }
                            break;
                        }
                    } catch (e) {
                        debugs('%O', e)
                    }
                }
            } catch (e) {
                debugs(e)
            } finally {
                debugs(`InputAdded: ${Date.now() - start}ms; attempts = ${attempt}`)
            }
        })
    }
}
