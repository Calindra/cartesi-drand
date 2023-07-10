import { ChainOptions, HttpCachingChain, HttpChainClient, fetchBeacon } from "drand-client"
import Axios, { AxiosInstance } from "axios";

export class DrandProvider {
    delaySeconds = 3
    chainHash = "dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493"
    publicKey = 'a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e' // (hex encoded)
    desiredState: 'RUNNING' | 'STOPPED' = 'RUNNING'
    inspectEndpoint = 'http://localhost:5005/inspect'
    inspectAxiosInstance: AxiosInstance;

    lastPendingTime = 0
    secodsToWait: number = 3;
    private drandClient: HttpChainClient

    constructor() {
        this.inspectAxiosInstance = Axios.create({ baseURL: this.inspectEndpoint })
        this.drandClient = this.createDrandClient()
    }

    async pendingDrandBeacon() {
        const res = await this.inspectAxiosInstance.get('/pending_drand_beacon')
        const firstReport = res.data.reports[0]
        if (firstReport?.payload && firstReport?.payload !== '0x00') {
            return { inputTime: Number(firstReport.payload) }
        } else {
            return null
        }
    }

    private createDrandClient() {
        const options: ChainOptions = {
            chainVerificationParams: { chainHash: this.chainHash, publicKey: this.publicKey },
            disableBeaconVerification: false,
            noCache: false
        }
        const chain = new HttpCachingChain(`https://api.drand.sh/${this.chainHash}`, options)
        return new HttpChainClient(chain, options)
    }

    async run() {
        this.desiredState = 'RUNNING'
        while (this.desiredState === 'RUNNING') {
            let pending = await this.pendingDrandBeacon()
            if (pending && this.lastPendingTime !== pending.inputTime && pending.inputTime < (Date.now() / 1000 - this.secodsToWait)) {
                const beacon = await fetchBeacon(this.drandClient)
                console.log('sending', beacon)
                this.lastPendingTime = pending.inputTime
            }
            await new Promise(resolve => setTimeout(resolve, Math.round(this.delaySeconds * 1000)))
        }
    }

    stop() {
        this.desiredState = 'STOPPED'
    }
}