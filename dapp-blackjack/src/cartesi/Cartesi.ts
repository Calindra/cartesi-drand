import { type CartesiClient, CartesiClientBuilder } from "web3-client";
import type { Provider, Signer } from "ethers";
import { address as DAppAddress } from "../deployments/dapp.json";

const CARTESI_INSPECT_ENDPOINT = new URL(
  process.env.CARTESI_INSPECT_ENDPOINT ?? "https://5005-cartesi-rollupsexamples-mk3ozp0tglt.ws-us104.gitpod.io/inspect",
);

console.debug("ENDPOINT", CARTESI_INSPECT_ENDPOINT);

/**
 * Lightweight wrapper around the CartesiClient to provide a more convenient interface.
 */
export class Cartesi {
  private static readonly cartesiClient: CartesiClient = new CartesiClientBuilder()
    .withEndpoint(CARTESI_INSPECT_ENDPOINT)
    .withLogger({
      info: console.log,
      error: console.error,
    })
    .withDappAddress(DAppAddress)
    .build();

  /**
   * Advance the machine state by sending an input to the machine.
   * If the machine is not in a state that expects an input, an error will be thrown.
   * Error already is logged by the @see {CartesiClient.advance}
   */
  static async sendInput(payload: Record<string, unknown>, signer?: Signer, provider?: Provider): Promise<void> {
    try {
      if (provider) Cartesi.cartesiClient.setProvider(provider);
      if (signer) Cartesi.cartesiClient.setSigner(signer);
      await Cartesi.cartesiClient.advance(payload);
    } catch (_error) {
      return;
    }
  }

  /**
   * Inspect the machine state.
   * Try get first report and parse the payload.
   * Error already is logged by the @see {CartesiClient.inspect}
   */
  static async inspectWithJson<T extends Record<string, unknown>>(
    json: Record<string, unknown>,
  ): Promise<T | null> {
    return Cartesi.cartesiClient.inspect<typeof json, T>(json);
  }
}
