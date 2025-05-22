import { Address, http, publicActions, stringToHex, walletActions, zeroAddress, type Account } from "viem";
import { Utils } from "./utils";
import { Hex } from "./hex";
import type { ObjectLike, Log } from "./types";
import { publicActionsL1, walletActionsL1, getInputsAdded, createCartesiPublicClient } from "@cartesi/viem";

export interface CartesiConstructor {
  /**
   * The endpoint of the Cartesi Rollups server
   */
  endpoint: URL;
  account?: Account;
  dapp_address: Address;
  logger: Log;
}

export class CartesiClientBuilder {
  private endpoint: URL;
  private dappAddress: Address;
  private logger: Log;
  private account?: Account;

  constructor() {
    this.endpoint = new URL("http://localhost:8545");
    this.dappAddress = zeroAddress;
    this.logger = {
      info: console.log,
      error: console.error,
    };
  }

  withEndpoint(endpoint: URL | string): CartesiClientBuilder {
    this.endpoint = new URL(endpoint);
    return this;
  }

  withDappAddress(address: Address): CartesiClientBuilder {
    this.dappAddress = address;
    return this;
  }

  withLogger(logger: Log): CartesiClientBuilder {
    this.logger = logger;
    return this;
  }

  withAccount(account: Account): CartesiClientBuilder {
    this.account = account;
    return this;
  }

  build(): CartesiClient {
    return new CartesiClient({
      endpoint: this.endpoint,
      dapp_address: this.dappAddress,
      logger: this.logger,
      account: this.account,
    });
  }
}

export class CartesiClient {
  constructor(private readonly config: CartesiConstructor) {}

  /**
   * Convert AddressLike, type used by ethers to string
   */
  async getDappAddress(): Promise<Address> {
    // return getAddress(this.config.dapp_address);
    return this.config.dapp_address;
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
          Accept: "application/json",
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

  getClient(account: Account) {
    return createCartesiPublicClient({
      account,
      transport: http(this.config.endpoint.href),
    })
      .extend(walletActions)
      .extend(publicActions)
      .extend(walletActionsL1())
      .extend(publicActionsL1());
  }

  // getCartesiClient() {
  //   return createCartesiPublicClient({
  //     transport: http(this.config.endpoint.href),
  //   });
  // }

  /**
   * Send InputBox
   * @param payload The data to be sent to the Cartesi Machine, transform to payload
   */
  async advance<T extends ObjectLike>(payload: T) {
    const { logger } = this.config;

    const account = this.config.account;
    if (!account) {
      throw new Error("No account provided");
    }

    const client = this.getClient(account);

    try {
      logger.info("getting client", client);
      logger.info("getting account address", account);
      const signerAddress = await account.getAddress?.();

      const chainId = await client.getChainId();

      logger.info(`connected to chain ${chainId}`);
      logger.info(`using account "${signerAddress}"`);

      // use message from command line option, or from user prompt
      logger.info(`sending "${JSON.stringify(payload)}"`);

      // convert string to input bytes (if it's not already bytes-like)
      const inputBytes = stringToHex(
        JSON.stringify({
          input: payload,
        }),
      );

      const dappAddress = await this.getDappAddress();

      // send transaction
      const tx = await client.addInput({
        account,
        chain: client.chain,
        application: dappAddress,
        payload: inputBytes,
      });

      logger.info(`transaction: ${tx}`);
      logger.info("waiting for receipt...");
      const receipt = await client.waitForTransactionReceipt({
        hash: tx,
      });
      logger.info(JSON.stringify(receipt));

      logger.info("check logs for input...");
      const [inputAdded] = getInputsAdded(receipt);

      if (inputAdded) {
        logger.info(`input event added: ${inputAdded}`);
        const { index: inputIndex } = inputAdded;
        logger.info("waiting for input to be processed...");

        const input = await client.waitForInput({
          application: dappAddress,
          inputIndex,
        });

        console.log("input processed", input);
      } else {
        logger.info("no input added");
      }
    } catch (e) {
      logger.error(e);

      if (e instanceof Error) {
        throw e;
      }

      throw new Error("Error on advance", { cause: e });
    }
  }
}
