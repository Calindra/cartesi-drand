import { IInputBox__factory, type InputBox } from "@cartesi/rollups";
import {
  ethers,
  type Signer,
  type Provider,
  Wallet,
  AddressLike,
  resolveAddress,
  ContractTransactionResponse,
} from "ethers";
type ObjectLike = Record<string, unknown>;

export interface Log {
  info(...args: unknown[]): void;
  error(...args: unknown[]): void;
}

export interface CartesiContructor {
  endpoint_cartesi_rollups: URL;
  address_dapp: AddressLike;
  signer: Signer;
  wallet: Wallet;
  provider: Provider;
  logger: Log;
}

export class Utils {
  static isObject(value: unknown): value is ObjectLike {
    return typeof value === "object" && value !== null;
  }

  static isArrayNonNullable<T = unknown>(value: unknown): value is Array<T> {
    return Array.isArray(value) && value.length > 0;
  }
}

export class Hex {
  static hex2a(hex: string) {
    let str = "";

    for (let i = 0; i < hex.length; i += 2) {
      let v = parseInt(hex.substring(i, i + 2), 16);
      if (v) str += String.fromCharCode(v);
    }
    return str;
  }
}

export class CartesiClient {
  constructor(private readonly config: CartesiContructor) {}

  /**
   * Convert AddressLike, type used by ethers to string
   */
  private async getAddress(): Promise<string> {
    return resolveAddress(this.config.address_dapp);
  }

  /**
   * @param payload The data to be sent to the Cartesi Machine, transform to payload
   * used to request reports
   */
  async inspect<T extends ObjectLike, U extends ObjectLike>(payload: T): Promise<U | null> {
    const inputJSON = JSON.stringify({ input: payload });
    const jsonEncoded = encodeURIComponent(inputJSON);

    const url = new URL(this.config.endpoint_cartesi_rollups);
    url.pathname = url.pathname.replace(/\/$/, "");
    url.pathname += `/${jsonEncoded}`;

    try {
      const response = await fetch(url);
      const result: unknown = await response.json();

      if (Utils.isObject(result) && "reports" in result && Utils.isArrayNonNullable(result.reports)) {
        const firstReport = result.reports.at(0);

        if (Utils.isObject(firstReport) && "payload" in firstReport && typeof firstReport.payload === "string") {
          const payload = Hex.hex2a(firstReport.payload.replace(/^0x/, ""));
          return JSON.parse(payload);
        }
      }
    } catch (e) {
      this.config.logger.error(e);
    }

    return null;
  }

  /**
   *
   * @param payload The data to be sent to the Cartesi Machine, transform to payload
   */
  async advance<T extends ObjectLike>(payload: T) {
    const { logger } = this.config;

    try {
      const { provider, signer } = this.config;
      const [address, network] = await Promise.all([this.getAddress(), provider.getNetwork()]);
      logger.info(`connected to chain ${network.chainId}`);

      // connect to rollups,
      const inputContract = IInputBox__factory.connect(address, signer);
      const signerAddress = await signer.getAddress();
      logger.info(`using account "${signerAddress}"`);

      // use message from command line option, or from user prompt
      logger.info(`sending "${JSON.stringify(payload)}"`);

      // convert string to input bytes (if it's not already bytes-like)
      const inputBytes = ethers.toUtf8Bytes(
        JSON.stringify({
          input: payload,
        })
      );

      // send transaction
      const tx = <ContractTransactionResponse>await inputContract.addInput(address, inputBytes);
      logger.info(`transaction: ${tx.hash}`);
      logger.info("waiting for confirmation...");
      const receipt = await tx.wait(1);
      logger.info(JSON.stringify(receipt));
    } catch (e) {
      logger.error(e);
    }
  }
}
