import { ChainOptions, HttpCachingChain, HttpChainClient, fetchBeacon } from "drand-client"
import Axios, { AxiosInstance } from "axios";
import InputSender from "./cartesi/InputSender";
import { CartesiConfig, DrandConfig, InputSenderConfig } from "./configs";
import { setTimeout } from 'node:timers/promises'

interface PendingDrandBeacon {
    reports?: Array<{
        payload?: string
    }>
 }

export class DrandProvider {

    desiredState: 'RUNNING' | 'STOPPED' = 'RUNNING'
    inspectAxiosInstance: AxiosInstance;

    cartesiConfig: CartesiConfig = {
        inspectEndpoint: "http://localhost:5005/inspect"
    }

    drandConfig: DrandConfig = {
        chainHash: "dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493",
        publicKey: 'a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e', // (hex encoded)
        secondsToWait: 3,
    }

    inputSenderConfig: InputSenderConfig = {
        dappAddress: '0x142105FC8dA71191b3a13C738Ba0cF4BC33325e2',
        mnemonic: 'test test test test test test test test test test test junk',
        rpc: 'http://localhost:8545',
        accountIndex: 0,
    }

    lastPendingTime = 0
    secondsToWait: number = 3;
    private drandClient: HttpChainClient
    inputSender: InputSender

    constructor() {
        this.inspectAxiosInstance = Axios.create({ baseURL: this.cartesiConfig.inspectEndpoint })
        this.drandClient = this.createDrandClient()
        this.inputSender = new InputSender(this.inputSenderConfig)
    }

    async pendingDrandBeacon() {
        try {
            // url = "http://localhost:5005/inspect/pending_drand_beacon"
            console.log('Fetching pending drand beacon')
            const res = await this.inspectAxiosInstance.get<PendingDrandBeacon>('/pending_drand_beacon')

            if (Array.isArray(res.data.reports) && res.data.reports.length > 0) {
                const firstReport = res.data.reports.at(0);
                if (firstReport?.payload && firstReport?.payload !== '0x00') {
                    return { inputTime: Number(firstReport.payload) }
                }
            }
        } catch (error) {

            if (Axios.isAxiosError(error)) {
                console.error(
                    "No connection to cartesi machine", error.code
                );
            } else {
                console.error('Error on pending drand beacon', error);
            }

        }

        return null;
    }

    private createDrandClient() {
        const options: ChainOptions = {
            chainVerificationParams: { chainHash: this.drandConfig.chainHash, publicKey: this.drandConfig.publicKey },
            disableBeaconVerification: false,
            noCache: false
        }
        const chain = new HttpCachingChain(`https://api.drand.sh/${this.drandConfig.chainHash}`, options)
        return new HttpChainClient(chain, options)
    }

    private configureInputSender() {
        this.inputSender.config = this.inputSenderConfig
    }

    async run() {
        this.desiredState = 'RUNNING'
        this.configureInputSender()
        while (this.desiredState === 'RUNNING') {
            let pending = await this.pendingDrandBeacon()
            if (pending && this.lastPendingTime !== pending.inputTime && pending.inputTime < (Date.now() / 1000 - this.secondsToWait)) {
                const beacon = await fetchBeacon(this.drandClient)
                console.log('sending beacon', beacon.round)
                this.inputSender.sendInput({ payload: JSON.stringify({ beacon }) })
                this.lastPendingTime = pending.inputTime
            }
            await this.someTime()
        }
    }

    async someTime() {
        return setTimeout(Math.round(this.secondsToWait * 1000))
        // return new Promise(resolve => globalThis.setTimeout(resolve, Math.round(this.secondsToWait * 1000)))
    }

    stop() {
        this.desiredState = 'STOPPED'
    }
}