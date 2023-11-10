import mock from "http-request-mock";
import { expect, it, describe, beforeEach, afterEach, jest } from "@jest/globals";
import { CartesiClient, CartesiClientBuilder } from "../src/main";
import { Provider, ethers } from "ethers";
import { Hex } from "../src/hex";
import { InputBox } from "@cartesi/rollups";

describe("CartesiClient", () => {
  const mocker = mock.setupForUnitTest("fetch");

  let cartesiClient: CartesiClient;
  const endpoint = new URL("http://localhost:8545/inspect");

  function generate_address(): string {

    for (let i = 0; i < 40; i++) {

    }


    return "0x123";
  }

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
    it("should error network if an exception is thrown", async () => {
      // Arrange
      const payload = { foo: "bar" };
      const provider = {
        getNetwork: jest.fn<() => Promise<unknown>>().mockRejectedValueOnce(new Error("network error")),
      } as any as Provider;

      const inputContract = {
        addInput: jest.fn<() => Promise<unknown>>().mockRejectedValueOnce(new Error("contract error")),
      } as any as InputBox;
      const client = new CartesiClientBuilder().withAddress("0x123").withProvider(provider).build();
      jest.spyOn(client, "getInputContract").mockResolvedValue(inputContract);
      // jest.spyOn(client, "getAddress").mockResolvedValue("0x123");
      // Act / Assert
      try {
        await client.advance(payload);
      } catch (err) {
        expect(err).toMatchObject({ error: "network error" });
      }
      // expect(client.advance(payload)).rejects.toThrow("network error");
    });

    it("should call successful", async () => {
      // Arrange
      const payload = { foo: "bar" };

      const inputContract = {
        addInput: jest.fn().mockReturnValueOnce({
          connect: jest.fn(),
        }),
      } as any as InputBox;
      const client = new CartesiClientBuilder().build();
      jest.spyOn(client, "getInputContract").mockResolvedValue(inputContract);
      jest.spyOn(client, "getAddress").mockResolvedValue("0x123");
      // Act / Assert
      // expect(client.advance(payload)).rejects.toThrow("network error");
    });
  });
});
