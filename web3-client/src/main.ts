import { Address, createPublicClient, createWalletClient, http, stringToHex, zeroAddress, type Account } from "viem";
import { Utils } from "./utils";
import { Hex } from "./hex";
import type { ObjectLike, Log } from "./types";
import { publicActionsL1, walletActionsL1, getInputsAdded, createCartesiPublicClient } from "@cartesi/viem";

export interface CartesiConstructor {
  /**
   * The endpoint of the Cartesi Rollups server
   */
  endpoint: URL;
  signer?: Account;
  dapp_address: Address;
  logger: Log;
}

export class CartesiClientBuilder {
  private endpoint: URL;
  private dappAddress: Address;
  private logger: Log;
  private signer?: Account;

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

  withSigner(signer: any): CartesiClientBuilder {
    this.signer = signer;
    return this;
  }

  build(): CartesiClient {
    return new CartesiClient({
      endpoint: this.endpoint,
      dapp_address: this.dappAddress,
      logger: this.logger,
      signer: this.signer,
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

  /**
   * Send InputBox
   * @param payload The data to be sent to the Cartesi Machine, transform to payload
   */
  async advance<T extends ObjectLike>(payload: T) {
    const { logger } = this.config;

    const account = this.config.signer;
    if (!account) {
      throw new Error("No account provided");
    }

    const walletClient = createWalletClient({
      account,
      transport: http(this.config.endpoint.href),
    }).extend(walletActionsL1());

    const publicClient = createPublicClient({
      transport: http(this.config.endpoint.href),
    }).extend(publicActionsL1());

    const publicClientL2 = createCartesiPublicClient({
      transport: http(this.config.endpoint.href),
    });

    try {
      logger.info("getting network", walletClient);
      logger.info("getting signer address", account);
      const signerAddress = await account.getAddress?.();

      const chainId = await walletClient.getChainId();

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
      const tx = await walletClient.addInput({
        account,
        chain: walletClient.chain,
        application: dappAddress,
        payload: inputBytes,
      });

      // const tx = <ContractTransactionResponse>await inputContract.addInput(dappAddress, inputBytes);
      logger.info(`transaction: ${tx}`);
      logger.info("waiting for receipt...");
      const receipt = await publicClient.waitForTransactionReceipt({
        hash: tx,
      });
      logger.info(JSON.stringify(receipt));

      logger.info("check logs for input...");
      const [inputAdded] = getInputsAdded(receipt);

      if (inputAdded) {
        logger.info(`input event added: ${inputAdded}`);
        const { index: inputIndex } = inputAdded;
        logger.info("waiting for input to be processed...");

        const input = await publicClientL2.waitForInput({
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
