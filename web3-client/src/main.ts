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
import InputBoxContract from "@cartesi/rollups/deployments/mainnet/InputBox.json";

export interface CartesiContructor {
  /**
   * The endpoint of the Cartesi Rollups server
   */
  endpoint: URL;
  /**
   * AddressLike, type used by ethers to string
   */
  dapp_address: AddressLike;
  signer: Signer;
  wallet?: Wallet;
  provider: Provider;
  logger: Log;
}

export class CartesiClientBuilder {
  private endpoint: URL;
  private dappAddress: AddressLike;
  private signer: Signer;
  private wallet?: Wallet;
  private provider: Provider;
  private logger: Log;

  constructor() {
    this.endpoint = new URL("http://localhost:8545");
    this.dappAddress = "";
    this.provider = ethers.getDefaultProvider(this.endpoint.href);
    this.signer = new ethers.VoidSigner("0x", this.provider);
    this.logger = {
      info: console.log,
      error: console.error,
    };
  }

  withEndpoint(endpoint: URL | string): CartesiClientBuilder {
    this.endpoint = new URL(endpoint);
    return this;
  }

  withDappAddress(address: AddressLike): CartesiClientBuilder {
    this.dappAddress = address;
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
      dapp_address: this.dappAddress,
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
  async getDappAddress(): Promise<string> {
    return resolveAddress(this.config.dapp_address);
  }

  /**
   * Singleton to create contract
   */
  async getInputContract(): Promise<InputBox> {
    if (!CartesiClient.inputContract) {
      const address = InputBoxContract.address;
      CartesiClient.inputContract = IInputBox__factory.connect(address, this.config.signer);
    }
    return CartesiClient.inputContract;
  }

  /**
   * Inspect the machine state and try to get the first report and parse the payload.
   *
   * @param payload The data to be sent to the Cartesi Machine, transform to payload
   * used to request reports
   */
  async inspect<T extends ObjectLike, U extends ObjectLike>(payload: T): Promise<U | null> {
    try {
      const inputJSON = JSON.stringify({ input: payload });
      const jsonEncoded = encodeURIComponent(inputJSON);

      const url = new URL(this.config.endpoint);
      url.pathname += `/${jsonEncoded}`;

      this.config.logger.info("Inspecting endpoint: ", url.href);

      const response = await fetch(url.href, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      });
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
   * Send InputBox
   * @param payload The data to be sent to the Cartesi Machine, transform to payload
   */
  async advance<T extends ObjectLike>(payload: T) {
    const { logger } = this.config;

    try {
      const { provider, signer } = this.config;
      logger.info("getting network", provider);
      const network = await provider.getNetwork();
      logger.info("getting siggner address", signer);
      const signerAddress = await signer.getAddress();

      logger.info(`connected to chain ${network.chainId}`);
      logger.info(`using account "${signerAddress}"`);

      // connect to rollups,
      const inputContract = await this.getInputContract();

      // use message from command line option, or from user prompt
      logger.info(`sending "${JSON.stringify(payload)}"`);

      // convert string to input bytes (if it's not already bytes-like)
      const inputBytes = ethers.toUtf8Bytes(
        JSON.stringify({
          input: payload,
        })
      );

      const dappAddress = await this.getDappAddress();

      // send transaction
      const tx = <ContractTransactionResponse>await inputContract.addInput(dappAddress, inputBytes);
      logger.info(`transaction: ${tx.hash}`);
      logger.info("waiting for confirmation...");
      const receipt = await tx.wait(1);
      logger.info(JSON.stringify(receipt));
    } catch (e) {
      logger.error(e);

      if (e instanceof Error) {
        throw e;
      }

      throw new Error("Error on advance", { cause: e });
    }
  }
}
