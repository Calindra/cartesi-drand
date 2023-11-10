import mock, { Mocker } from "http-request-mock";
import { expect, it, describe, beforeEach, afterEach, jest } from "@jest/globals";
import { CartesiClient, CartesiClientBuilder } from "../src/main";
import { JsonRpcProvider, Signer, ethers } from "ethers";

describe("Main", () => {
  const mocker = mock.setupForFetch();

  let cartesiClient: CartesiClient;
  const endpoint = new URL("http://localhost:8545");

  beforeEach(async () => {
    const provider = ethers.getDefaultProvider("http://localhost:8545");
    cartesiClient = new CartesiClientBuilder().withEndpoint(endpoint).withProvider(provider).build();
  });

  afterEach(() => {
    mocker.reset();
  });
});

describe("CartesiClient", () => {
  const mocker = mock.setupForFetch();

  let cartesiClient: CartesiClient;
  const endpoint = new URL("http://localhost:8545");

  beforeEach(async () => {
    const provider = ethers.getDefaultProvider(endpoint.href);
    cartesiClient = new CartesiClientBuilder().withEndpoint(endpoint).withProvider(provider).build();
  });

  afterEach(() => {
    mocker.reset();
  });

  describe("inspect", () => {
    it("should return null if the response is not valid", async () => {
      // Arrange
      const payload = { foo: "bar" };
      const encode = encodeURIComponent(JSON.stringify(payload));
      const url = new URL(`/inspect/${encode}`, endpoint.href);
      mocker.get(url.href, { status: 404 });

      // Act
      const result = await cartesiClient.inspect(payload);

      // Assert
      expect(result).toBeNull();
    });

    it("should return the payload from the first report if the response is valid", async () => {
      // Arrange
      const payload = { foo: "bar" };
      const encode = encodeURIComponent(JSON.stringify(payload));
      const url = new URL(`/inspect/${encode}`, endpoint.href);
      mocker.get(url.href, { body: { reports: [{ payload: "0x7b22626172223a22666f6f227d" }] } });

      // Act
      const result = await cartesiClient.inspect(payload);

      // Assert
      expect(result).toEqual({ bar: "foo" });
    });
  });

  describe("advance", () => {
    // it("should log an error if an exception is thrown", async () => {
    //   // Arrange
    //   const payload = { foo: "bar" };
    //   const logger = { error: jest.fn() };
    //   const provider = { getNetwork: jest.fn().mockRejectedValue(new Error("network error")) };
    //   const signer = { getAddress: jest.fn().mockResolvedValue("0x123") };
    //   const inputContract = { addInput: jest.fn().mockRejectedValue(new Error("contract error")) };
    //   const config = { logger, provider, signer };
    //   const client = new CartesiClient(config);
    //   jest.spyOn(client, "getInputContract").mockResolvedValue(inputContract);
    //   // Act
    //   await client.advance(payload);
    //   // Assert
    //   expect(logger.error).toHaveBeenCalledWith(new Error("network error"));
    //   expect(logger.error).toHaveBeenCalledWith(new Error("contract error"));
    // });
    // it("should log the transaction hash and receipt if the transaction is successful", async () => {
    //   // Arrange
    //   const payload = { foo: "bar" };
    //   const logger = { info: jest.fn() };
    //   const provider = { getNetwork: jest.fn().mockResolvedValue({ chainId: 1 }) };
    //   const signer = { getAddress: jest.fn().mockResolvedValue("0x123") };
    //   const inputContract = { addInput: jest.fn().mockResolvedValue({ hash: "0xabc" }) };
    //   const config = { logger, provider, signer };
    //   const client = new CartesiClient(config);
    //   jest.spyOn(client, "getInputContract").mockResolvedValue(inputContract);
    //   // Act
    //   await client.advance(payload);
    //   // Assert
    //   expect(logger.info).toHaveBeenCalledWith(`connected to chain ${provider.getNetwork().chainId}`);
    //   expect(logger.info).toHaveBeenCalledWith(`using account "${await signer.getAddress()}"`);
    //   expect(logger.info).toHaveBeenCalledWith(`sending "${JSON.stringify(payload)}"`);
    //   expect(logger.info).toHaveBeenCalledWith(`transaction: ${inputContract.addInput().hash}`);
    //   expect(logger.info).toHaveBeenCalledWith(expect.stringContaining('"blockNumber":1'));
    // });
  });
});
