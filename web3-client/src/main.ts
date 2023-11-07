import { IInputBox__factory, InputBox, InputBox } from "@cartesi/rollups";
import type { Signer, providers } from "ethers";
type ObjectLike = Record<string, unknown>;
type Provider = providers.Provider;

export interface Log {
  info(...args: unknown[]): void;
  error(...args: unknown[]): void;
}

export interface CartesiContructor {
  endpoint: URL;
  address: string;
  signer: Signer;
  wallet: Wallet;
  provider: Provider;
  logger: Log;
}

export class Wallet {}
// export class Provider {}

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
   * @param data The data to be sent to the Cartesi Machine, transform to payload
   */
  async inspect<T extends ObjectLike, U extends ObjectLike>(data: T): Promise<U | null> {
    const inputJSON = JSON.stringify({ input: data });
    const jsonEncoded = encodeURIComponent(inputJSON);

    const url = new URL(this.config.endpoint);
    url.pathname = url.pathname.replace(/\/$/, "");
    url.pathname += `/${jsonEncoded}`;

    try {
      const response = await fetch(url);
      const result: unknown = await response.json();

      if (Utils.isObject(result) && "reports" in result && Utils.isArrayNonNullable(result.reports)) {
        const firstReport = result.reports[0];

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
   * @param data The data to be sent to the Cartesi Machine, transform to payload
   */
  async advance<T extends ObjectLike>(data: T) {
    try {
      const { provider, logger, signer } = this.config;
      const network = await provider.getNetwork();
      logger.info(`connected to chain ${network.chainId}`);

      // connect to rollups,
      const inputContract = IInputBox__factory.connect(InputBox.address, signer);
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
      const dappAddress = DApp.address; // '0x142105FC8dA71191b3a13C738Ba0cF4BC33325e2'
      const tx = <ContractTransactionResponse>await inputContract.addInput(dappAddress, inputBytes);
      // const tx: any = await inputContract.addInput(dappAddress, inputBytes);
      console.log(`transaction: ${tx.hash}`);
      console.log("waiting for confirmation...");
      const receipt = await tx.wait(1);
      console.log(JSON.stringify(receipt));
      // find reference to notice from transaction receipt
      // const inputKeys = getInputKeys(receipt);
      // console.log(
      //     `input ${inputKeys.input_index} added`
      // );
    } catch (e) {
      console.error(e);
    }
  }
}
