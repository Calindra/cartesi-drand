import { AddressLike, Provider, Signer } from "ethers";
import { CartesiClient, CartesiClientBuilder } from "../main";
import { AxiosLikeClient } from "./AxiosLikeClient";
import { fetch as _fetch, setup as fetchSetup } from "./FetchLikeClient";

interface SetupOptions {
    endpoints: {
        graphQL: URL;
        inspect: URL;
    };
    provider?: Provider;
    signer?: Signer;
    dappAddress: AddressLike
}

export class Cartesify {

    axios: AxiosLikeClient

    constructor(cartesiClient: CartesiClient) {
        this.axios = new AxiosLikeClient(cartesiClient)
    }

    static setup(options: SetupOptions) {
        const builder = new CartesiClientBuilder()
            .withDappAddress(options.dappAddress)
            .withEndpoint(options.endpoints.inspect)
            .withEndpointGraphQL(options.endpoints.graphQL)
        if (options.provider) {
            builder.withProvider(options.provider)
        }
        const cartesiClient = builder.build()
        if (options.signer) {
            cartesiClient.setSigner(options.signer)
        }
        fetchSetup(cartesiClient)
    }

    static fetch = _fetch
}
