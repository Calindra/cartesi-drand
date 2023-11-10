import { IInputBox__factory, type InputBox } from "@cartesi/rollups";
import {
  ethers,
  type Signer,
  type Provider,
  Wallet,
  type AddressLike,
  resolveAddress,
  type ContractTransactionResponse,
} from "ethers";
import { Utils } from "./utils";
import { Hex } from "./hex";
import type { ObjectLike, Log } from "./types";

export interface CartesiContructor {
  /**
   * The endpoint of the Cartesi Rollups server
   */
  endpoint: URL;
  /**
   * Input box address
   * AddressLike, type used by ethers to string
   */
  address: AddressLike;
  signer: Signer;
  wallet?: Wallet;
  provider: Provider;
  logger: Log;
}

export class CartesiClientBuilder {
  private endpoint: URL;
  private address: AddressLike;
  private signer: Signer;
  private wallet?: Wallet;
  private provider: Provider;
  private logger: Log;

  constructor() {
    this.endpoint = new URL("http://localhost:8545");
    this.address = "";
    this.signer = new ethers.VoidSigner("0x");
    this.provider = ethers.getDefaultProvider(this.endpoint.href);
    this.logger = {
      info: console.log,
      error: console.error,
    };
  }

  withEndpoint(endpoint: URL): CartesiClientBuilder {
    this.endpoint = endpoint;
    return this;
  }

  withAddress(address: AddressLike): CartesiClientBuilder {
    this.address = address;
    return this;
  }

  withSigner(signer: Signer): CartesiClientBuilder {
    this.signer = signer;
    return this;
  }

  withWallet(wallet: Wallet): CartesiClientBuilder {
    this.wallet = wallet;
    return this;
  }

  withProvider(provider: Provider): CartesiClientBuilder {
    this.provider = provider;
    return this;
  }

  withLogger(logger: Log): CartesiClientBuilder {
    this.logger = logger;
    return this;
  }

  build(): CartesiClient {
    return new CartesiClient({
      endpoint: this.endpoint,
      address: this.address,
      signer: this.signer,
      wallet: this.wallet,
      provider: this.provider,
      logger: this.logger,
    });
  }
}

export class CartesiClient {
  private static inputContract: InputBox;

  constructor(private readonly config: CartesiContructor) {}

  /**
   * Convert AddressLike, type used by ethers to string
   */
  private async getAddress(): Promise<string> {
    return resolveAddress(this.config.address);
  }

  /**
   * Singleton to create contract
   */
  private async getInputContract(): Promise<InputBox> {
    if (!CartesiClient.inputContract) {
      const address = await this.getAddress();
      CartesiClient.inputContract = IInputBox__factory.connect(address, this.config.signer);
    }
    return CartesiClient.inputContract;
  }

  /**
   * @param payload The data to be sent to the Cartesi Machine, transform to payload
   * used to request reports
   */
  async inspect<T extends ObjectLike, U extends ObjectLike>(payload: T): Promise<U | null> {
    const inputJSON = JSON.stringify({ input: payload });
    const jsonEncoded = encodeURIComponent(inputJSON);

    const url = new URL(this.config.endpoint);
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
      const inputContract = await this.getInputContract();
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
