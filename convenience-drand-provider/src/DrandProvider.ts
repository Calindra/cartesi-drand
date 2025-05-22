import {
  ChainOptions,
  HttpCachingChain,
  HttpChainClient,
  fetchBeacon,
} from "drand-client";
import InputSender from "./cartesi/InputSender";
import { CartesiConfig, DrandConfig, InputSenderConfig } from "./configs";
import { setTimeout } from "node:timers/promises";

interface PendingDrandBeacon {
  reports?: Array<{
    payload?: string;
  }>;
}

export class DrandProvider {
  desiredState: "RUNNING" | "STOPPED" = "RUNNING";

  inputSenderConfig: InputSenderConfig = {
    /** @todo change to dotenv */
    dappAddress: "0x9Da41EcC954E2b4DC2678F940F59EC48987BA887",
    mnemonic: "test test test test test test test test test test test junk",
    rpc: new URL(process.env.RPC_ENDPOINT ?? "http://localhost:8545").href,
    accountIndex: 0,
  };

  cartesiConfig: CartesiConfig = {
    inspectEndpoint: new URL(
      `/inspect/${this.inputSenderConfig.dappAddress}`,
      process.env.INSPECT_ENDPOINT ?? "http://localhost:8080"
    ).href,
  };

  /**
   * @todo change to dotenv
   * GET https://api.drand.sh/{hash}/info
   */
  drandConfig: DrandConfig = {
    chainHash:
      "52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971",
    publicKey:
      "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a", // (hex encoded)
    secondsToWait: 3,
  };

  lastPendingTime = 0;
  secondsToWait: number = 6;
  private drandClient: HttpChainClient;
  inputSender: InputSender;

  constructor() {
    console.log("inspectEndpoint base url", this.cartesiConfig.inspectEndpoint);
    this.drandClient = this.createDrandClient();
    this.inputSender = new InputSender(this.inputSenderConfig);
  }

  async pendingDrandBeacon() {
    try {
      // url = "http://localhost:5005/inspect/pendingdrandbeacon"
      console.log(`${new Date().toISOString()}: Fetching pending drand beacon`);
      const data = await fetch(this.cartesiConfig.inspectEndpoint, {
        method: "POST",
      });
      if (!data.ok) {
        throw new Error(`data: ${data.status} ${data.statusText}`);
      }
      const res: PendingDrandBeacon = await data.json();

      console.log("Result", res);

      if (Array.isArray(res.reports) && res.reports.length > 0) {
        const firstReport = res.reports.at(0);
        if (firstReport?.payload && firstReport.payload !== "0x00") {
          return { inputTime: Number(firstReport.payload) };
        }
      }
    } catch (error) {
      console.error(
        `${new Date().toISOString()}: Error on pending drand beacon`,
        error
      );
    }

    return null;
  }

  private createDrandClient() {
    const options: ChainOptions = {
      chainVerificationParams: {
        chainHash: this.drandConfig.chainHash,
        publicKey: this.drandConfig.publicKey,
      },
      disableBeaconVerification: false,
      noCache: false,
    };
    const chain = new HttpCachingChain(
      `https://api.drand.sh/${this.drandConfig.chainHash}`,
      options
    );
    return new HttpChainClient(chain, options);
  }

  private configureInputSender() {
    this.inputSender.config = this.inputSenderConfig;
  }

  async run() {
    this.desiredState = "RUNNING";
    this.configureInputSender();
    while (this.desiredState === "RUNNING") {
      try {
        const pending = await this.pendingDrandBeacon();
        if (this.canSendBeacon(pending)) {
          const beacon = await fetchBeacon(this.drandClient);
          console.log("sending beacon", beacon.round);
          this.inputSender.sendInput({ payload: JSON.stringify({ beacon }) });
          this.lastPendingTime = pending.inputTime;
        }
        await this.someTime();
      } catch (e) {
        console.error(e);
      }
    }
  }

  private canSendBeacon(
    pending: Awaited<ReturnType<typeof this.pendingDrandBeacon>>
  ): pending is NonNullable<typeof pending> {
    return (
      (pending &&
        this.lastPendingTime !== pending.inputTime &&
        pending.inputTime < Date.now() / 1000 - this.secondsToWait) ??
      false
    );
  }

  someTime() {
    return setTimeout(Math.round(this.secondsToWait * 1000));
    // return new Promise(resolve => globalThis.setTimeout(resolve, Math.round(this.secondsToWait * 1000)))
  }

  stop() {
    this.desiredState = "STOPPED";
  }
}
