import { CartesiClient } from "../main";
import { Utils } from "../utils";
import { AxiosWrappedPromise } from "./AxiosWrappedPromise";


export class InputAddedListener {
    static listenerAdded = false
    static requests: Record<string, AxiosWrappedPromise> = {}

    constructor(private cartesiClient: CartesiClient) {

    }

    async addListener() {
        const MAX_RETRY = 20
        const cartesiClient = this.cartesiClient;
        if (!cartesiClient) {
            throw new Error('You need to configure the Cartesi client')
        }
        if (InputAddedListener.listenerAdded) {
            return
        }
        InputAddedListener.listenerAdded = true
        const { logger } = cartesiClient.config;
        const contract = await cartesiClient.getInputContract()
        contract.on("InputAdded", async (_dapp, inboxInputIndex, _sender, input) => {
            const start = Date.now()
            let attempt = 0;
            try {
                const str = Utils.hex2str(input.replace(/0x/, ''))
                const payload = JSON.parse(str)
                const wPromise = InputAddedListener.requests[payload.requestId]
                if (!wPromise) {
                    return
                }
                while (attempt < MAX_RETRY) {
                    try {
                        attempt++;
                        const req = await fetch("http://localhost:8080/graphql", {
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
                            "referrer": "http://localhost:8080/graphql",
                            "referrerPolicy": "strict-origin-when-cross-origin",
                            "body": `{\"operationName\":null,\"variables\":{},\"query\":\"{\\n  input(index: ${inboxInputIndex}) {\\n    reports(first: 10) {\\n      edges {\\n        node {\\n          payload\\n        }\\n      }\\n    }\\n  }\\n}\\n\"}`,
                            "method": "POST",
                            "mode": "cors",
                            "credentials": "omit"
                        });
                        const json = await req.json()
                        if (json.data?.input.reports.edges.length > 0) {
                            const hex = json.data.input.reports.edges[0].node.payload.replace(/0x/, '')
                            const strRes = Utils.hex2str(hex)
                            const successOrError = JSON.parse(strRes)
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
                        await new Promise((resolve) => setTimeout(resolve, 1000))
                    } catch (e) {
                        logger.error(e)
                    }
                }
            } catch (e) {
                logger.error(e)
            } finally {
                logger.info(`InputAdded: ${Date.now() - start}ms; attempts = ${attempt}`)
            }
        })
    }
}
