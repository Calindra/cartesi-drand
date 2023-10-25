import nock from "nock";


export default class Helper {

    static nockUrl = new URL("/inspect", process.env.INSPECT_ENDPOINT ?? "http://localhost:8080");

    static nockInspectEndpointRandomIsNeeded() {
        return nock(Helper.nockUrl)
            .get(/\/inspect\/pendingdrandbeacon/)
            .reply(200, {
                "status": "Accepted",
                "reports": [
                    {
                        "payload": "0x01"
                    }
                ],
                "processed_input_count": 0
            })
    }
    static nockInspectEndpointRandomIsntNeeded() {
        return nock(Helper.nockUrl)
            .get(/\/inspect\/pendingdrandbeacon/)
            .reply(200, {
                "status": "Accepted",
                "reports": [
                    {
                        "payload": "0x00"
                    }
                ],
                "processed_input_count": 0
            })
    }
}