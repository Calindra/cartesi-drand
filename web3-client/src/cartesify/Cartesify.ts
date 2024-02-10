import { AddressLike, Provider, Signer } from "ethers";
import { CartesiClient, CartesiClientBuilder } from "../main";
import { AxiosLikeClient } from "./AxiosLikeClient";
import { FetchFun, fetch as _fetch } from "./FetchLikeClient";

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

    static createFetch(options: SetupOptions): FetchFun {
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
        return (input: string | URL | globalThis.Request, init?: RequestInit) => {
            return _fetch(input, { ...init, cartesiClient })
        }
    }
}
