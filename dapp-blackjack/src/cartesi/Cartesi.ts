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
  private readonly cartesiClient: CartesiClient;

  constructor(private readonly signer: Signer, private readonly provider: Provider) {
    this.cartesiClient = new CartesiClientBuilder()
      .withEndpoint(CARTESI_INSPECT_ENDPOINT)
      .withLogger({
        info: console.log,
        error: console.error,
      })
      .withProvider(provider)
      .withSigner(signer)
      .withDappAddress(DAppAddress)
      .build();
  }

  /**
   * Advance the machine state by sending an input to the machine.
   * If the machine is not in a state that expects an input, an error will be thrown.
   * Error already is logged by the @see {CartesiClient.advance}
   */
  async sendInput(payload: Record<string, unknown>): Promise<void> {
    try {
      await this.cartesiClient.advance(payload);
    } catch (_e) {
      return;
    }
  }

  /**
   * Inspect the machine state.
   * Try get first report and parse the payload.
   * Error already is logged by the @see {CartesiClient.inspect}
   */
  async inspectWithJson<T extends Record<string, unknown>>(json: Record<string, unknown>): Promise<T | null> {
    return this.cartesiClient.inspect<typeof json, T>(json);
  }
}
