// it("should inspect state", async () => {
    //   const payload = { foo: "bar" };
    //   const expected = { bar: "baz" };

    //   mocker.mock({
    //     url: "http://localhost:8545",
    //   });

    //   mocker.mockResponseOnce(
    //     JSON.stringify({ reports: [{ payload: Buffer.from(JSON.stringify(expected)).toString("hex") }] })
    //   );

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toEqual(expected);
    // });

    // it("should return null when response is invalid", async () => {
    //   const payload = { foo: "bar" };
    //   mocker.mockResponseOnce(JSON.stringify({}));

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toBeNull();
    // });

    // it("should return null when response is not an object", async () => {
    //   const payload = { foo: "bar" };
    //   mocker.mockResponseOnce(JSON.stringify([]));

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toBeNull();
    // });

    // it("should return null when reports array is not an array", async () => {
    //   const payload = { foo: "bar" };
    //   mocker.mockResponseOnce(JSON.stringify({ reports: {} }));

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toBeNull();
    // });

    // it("should return null when reports array is empty", async () => {
    //   const payload = { foo: "bar" };
    //   mocker.mockResponseOnce(JSON.stringify({ reports: [] }));

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toBeNull();
    // });

    // it("should return null when first report is not an object", async () => {
    //   const payload = { foo: "bar" };
    //   mocker.mockResponseOnce(JSON.stringify({ reports: ["invalid"] }));

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toBeNull();
    // });

    // it("should return null when first report has no payload", async () => {
    //   const payload = { foo: "bar" };
    //   mocker.mockResponseOnce(JSON.stringify({ reports: [{ foo: "bar" }] }));

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toBeNull();
    // });

    // it("should return null when payload is not a string", async () => {
    //   const payload = { foo: "bar" };
    //   mocker.mockResponseOnce(JSON.stringify({ reports: [{ payload: 42 }] }));

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toBeNull();
    // });

    // it("should return null when payload is not valid JSON", async () => {
    //   const payload = { foo: "bar" };
    //   mocker.mockResponseOnce(JSON.stringify({ reports: [{ payload: "invalid" }] }));

    //   const result = await cartesiClient.inspect(payload);

    //   expect(result).toBeNull();
    // });