import { type CartesiClient, CartesiClientBuilder } from "web3-client";
import type { Provider, Signer } from "ethers";

// sunodo v0.11.2
const CARTESI_APPLICATION_ADDRESS = "0xab7528bb862fb57e8a2bcd567a2e929a0be56a5e"

const CARTESI_INSPECT_ENDPOINT = new URL(
  process.env.CARTESI_INSPECT_ENDPOINT ?? "https://localhost:8080/inspect",
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
    .withDappAddress(CARTESI_APPLICATION_ADDRESS)
    .build();

  /**
   * Advance the machine state by sending an input to the machine.
   * If the machine is not in a state that expects an input, an error will be thrown.
   * Error already is logged by the @see {CartesiClient.advance}
   */
  static async sendInput(payload: Record<string, unknown>, signer?: Signer, provider?: Provider): Promise<void> {
    try {
      if (provider) Cartesi.cartesiClient.setProvider(provider as any);
      if (signer) Cartesi.cartesiClient.setSigner(signer as any);
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
