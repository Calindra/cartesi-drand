import { describe } from "node:test";
import { DrandProvider } from "../src/DrandProvider"
import nock from "nock";

describe('DrandProvider', () => {

    beforeEach(async () => {
        nock('http://localhost:5005')
            .get(/\/inspect\/randomNeeded/)
            .reply(200, {
                "status": "Accepted",
                // "exception_payload": "string",
                "reports": [
                    {
                        "payload": "0x01"
                    }
                ],
                "processed_input_count": 0
            })
            .persist()
    })

    it("should do the polling to see the need of the Drand's beacon", async () => {
        const provider = new DrandProvider()
        const runPromise = provider.run()
        setTimeout(() => {
            provider.stop()
        }, 3500)
        await runPromise
    })
})

