import { CartesiClient } from "../main";
import { AxiosLikeClient } from "./AxiosLikeClient";


export class Cartesify {
    axios: AxiosLikeClient

    constructor(cartesiClient: CartesiClient) {
        this.axios = new AxiosLikeClient(cartesiClient)
    }
}
