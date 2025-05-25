import { DrandProvider } from "../src/DrandProvider";
import nock from "nock";
import Helper from "./Helper";
import { describe, beforeEach, it, mock } from "node:test";
import * as assert from "node:assert/strict";
import type { Args } from "../src/cartesi/rollups";

describe("DrandProvider", () => {
  beforeEach(async () => {
    nock.cleanAll();
    mock.reset();
  });

  describe(".pendingDrandBeacon()", () => {
    it("should inform the inputTime when there is some random seed pending", async () => {

      const provider = new DrandProvider();
      const dappAddress = provider.inputSenderConfig.dappAddress;
      Helper.nockInspectEndpointRandomIsNeeded(dappAddress);
      const resp = await provider.pendingDrandBeacon();
      assert.ok(resp?.inputTime);
    });
    it("should respond null when inspect response is 0x00, aka no need for beacon", async () => {
      Helper.nockInspectEndpointRandomIsntNeeded();
      const provider = new DrandProvider();
      const resp = await provider.pendingDrandBeacon();
      assert.strictEqual(resp, null);
    });
  });

  describe(".run()", () => {
    it("should do the polling to see the need of the Drand's beacon", async () => {
      Helper.nockInspectEndpointRandomIsNeeded().persist();
      const provider = new DrandProvider();
      let inputSent: Args | undefined;
      mock.method(provider.inputSender, "sendInput", async (args: Args) => {
        inputSent = args;
      });
      mock.getter(provider.drandConfig, "secondsToWait", () => 0.3);
      const runPromise = provider.run();
      setTimeout(() => {
        provider.stop();
      }, 1000);
      await runPromise;
      assert.ok(inputSent);
      assert.ok(inputSent.payload);
      const payload = JSON.parse(inputSent.payload);
      assert.ok(payload.beacon);
    });
  });
});
