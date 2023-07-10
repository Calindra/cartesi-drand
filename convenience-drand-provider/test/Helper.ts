import nock from "nock";


export default class Helper {
    static nockInspectEndpointRandomIsNeeded() {
        return nock('http://localhost:5005')
            .get(/\/inspect\/randomNeeded/)
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
        return nock('http://localhost:5005')
            .get(/\/inspect\/randomNeeded/)
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