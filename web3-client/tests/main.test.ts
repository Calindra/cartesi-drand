import mock from "http-request-mock";
import { expect, it, describe, beforeEach } from "@jest/globals";

describe("Test", () => {
  const mocker = mock.setupForFetch();
  beforeEach(() => {
    mocker.reset();
  });

  it("should could inspect state", async () => {
    expect(true).toBe(true);
  });
});
