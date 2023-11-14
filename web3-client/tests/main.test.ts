import mock from "http-request-mock";
import { expect, it, describe, beforeEach, afterEach, jest } from "@jest/globals";
import { CartesiClient, CartesiClientBuilder } from "../src/main";
import { Network, type Provider, ethers } from "ethers";
import { Hex } from "../src/hex";
import type { InputBox } from "@cartesi/rollups";
import type { Log } from "../src/types";

function generateValidEth(): string {
  const hexChars = "0123456789abcdef";
  let address = "0x";
  for (let i = 0; i < 40; i++) {
    address += hexChars[Math.floor(Math.random() * hexChars.length)];
  }
  return address;
}

describe("CartesiClient", () => {
  const mocker = mock.setupForUnitTest("fetch");

  let cartesiClient: CartesiClient;
  const endpoint = new URL("http://localhost:8545/inspect");

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
      const payload = { action: "show_games" };
      const wrongBody = {
        foo: "bar",
      };
      mocker.get(endpoint.href, wrongBody, {
        times: 1,
      });

      // Act
      const result = await cartesiClient.inspect(payload);

      // Assert
      expect(result).toBeNull();
    });

    it("should return the payload from the first report if the response is valid", async () => {
      // Arrange
      const payload = { action: "show_games" };
      const games = { games: [1, 2, 3] };
      const gamesPayload = Hex.obj2hex(games);
      mocker.get(
        endpoint.href,
        {
          reports: [{ payload: gamesPayload }],
        },
        {
          times: 1,
        }
      );

      // Act
      const result = await cartesiClient.inspect(payload);

      // Assert
      expect(result).toMatchObject(games);
    });
  });

  describe("advance", () => {
    describe("should error", () => {
      it("Error network if an exception is thrown", async () => {
        // Arrange
        const payload = { action: "new_player", name: "calindra" };
        const logger: Log = { error: jest.fn(), info: console.log };
        const address = generateValidEth();

        const provider = {
          getNetwork: jest.fn<() => Promise<unknown>>().mockRejectedValueOnce(new Error("network error")),
        } as any as Provider;


        const client = new CartesiClientBuilder()
          .withDappAddress(address)
          .withLogger(logger) //omit error log
          .withProvider(provider)
          .build();
        // Act / Assert
        return expect(client.advance(payload)).rejects.toThrow("network error");
      });

      it("Error contract if an exception is thrown", async () => {
        // Arrange
        const payload = { action: "new_player", name: "calindra" };
        const logger: Log = { error: jest.fn(), info: console.log };
        const address = generateValidEth();

        const provider: Pick<Provider, "getNetwork"> = {
          getNetwork: jest
            .fn<Provider["getNetwork"]>()
            .mockReturnValueOnce(Promise.resolve(new Network("homestead", 1))),
        };

        const inputContract: Pick<InputBox, "addInput"> = {
          addInput: jest.fn<InputBox["addInput"]>().mockRejectedValueOnce(new Error("contract error")),
        };


        const client = new CartesiClientBuilder()
          .withDappAddress(address)
          .withLogger(logger) //omit error log
          .withProvider(provider as Provider)
          .build();
        jest.spyOn(client, "getInputContract").mockResolvedValue(inputContract as InputBox);
        // Act / Assert
        return expect(client.advance(payload)).rejects.toThrow("contract error");
      });
    });
    // it("should error network if an exception is thrown", async () => {
    //   // Arrange
    //   const payload = { action: "new_player", name: "calindra" };
    //   const logger: Log = { error: jest.fn(), info: jest.fn() };

    //   const provider = {
    //     getNetwork: jest.fn<() => Promise<unknown>>().mockRejectedValueOnce(new Error("network error")),
    //   } as any as Provider;

    //   const inputContract = {
    //     addInput: jest.fn<() => Promise<unknown>>().mockRejectedValueOnce(new Error("contract error")),
    //   } as any as InputBox;

    //   const address = generateValidEth();

    //   const client = new CartesiClientBuilder()
    //     .withDappAddress(address)
    //     .withLogger(logger) //omit error log
    //     .withProvider(provider)
    //     .build();
    //   jest.spyOn(client, "getInputContract").mockResolvedValue(inputContract);
    //   // Act / Assert
    //   return expect(client.advance(payload)).rejects.toThrow("network error");
    // });

    it.skip("should call successful", async () => {
      // Arrange
      const payload = { action: "new_player", name: "calindra" };

      const address = generateValidEth();

      // const inputContract = {
      //   addInput: jest.fn().mockReturnValueOnce({
      //     connect: jest.fn(),
      //   }),
      // } as any as InputBox;
      const client = new CartesiClientBuilder().withDappAddress(address).build();
      // jest.spyOn(client, "getInputContract").mockResolvedValue(inputContract);
      // Act / Assert
      return expect(client.advance(payload)).resolves.not.toThrow();
    });
  });
});
