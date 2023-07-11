import { DrandProvider } from "../src/DrandProvider"
import nock from "nock";
import Helper from "./Helper";

describe('DrandProvider', () => {

    beforeEach(async () => {
        nock.cleanAll()
    })

    describe('.pendingDrandBeacon()', () => {
        it("should inform the inputTime when there is some random seed pending", async () => {
            Helper.nockInspectEndpointRandomIsNeeded()
            const provider = new DrandProvider()
            const resp = await provider.pendingDrandBeacon()
            expect(resp?.inputTime).toBeDefined()
        })
        it("should respond null when inspect response is 0x00, aka no need for beacon", async () => {
            Helper.nockInspectEndpointRandomIsntNeeded()
            const provider = new DrandProvider()
            const resp = await provider.pendingDrandBeacon()
            expect(resp).toBe(null)
        })
    })

    describe('.run()', () => {
        it("should do the polling to see the need of the Drand's beacon", async () => {
            Helper.nockInspectEndpointRandomIsNeeded().persist()
            const provider = new DrandProvider()
            let inputSent: any
            jest.spyOn(provider.inputSender, 'sendInput').mockImplementation(async (args: any) => {
                inputSent = args
            })
            provider.drandConfig.secondsToWait = .3
            const runPromise = provider.run()
            setTimeout(() => {
                provider.stop()
            }, 1000)
            await runPromise
            expect(inputSent.payload).toBeDefined()
            const payload = JSON.parse(inputSent.payload)
            expect(payload.beacon).toBeDefined()
        })
    })
})

