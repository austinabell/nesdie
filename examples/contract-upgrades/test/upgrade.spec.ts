import path from "path";
import * as fs from "fs/promises";
import { Runner } from "near-runner";

describe(`Running on ${Runner.getNetworkFromEnv()}`, () => {
  let runner: Runner;
  jest.setTimeout(60_000);

  beforeAll(async () => {
    runner = await Runner.create(async ({ root }) => ({
      contract: await root.createAndDeploy(
        "upgrade",
        path.join(__dirname, "..", "res", "upgrade_a.wasm")
      ),
      ali: await root.createAccount("ali"),
    }));
  });

  test("upgrade contract", async () => {
    await runner.run(async ({ contract }) => {
      // Assert that function on contract b can't be called yet
      await expect(async () =>
        contract
          .createTransaction(contract)
          .functionCall("some_new_function", new Uint8Array())
          .signAndSend()
      ).rejects.toThrow();

      let res = await contract
        .createTransaction(contract)
        .functionCall(
          "upgrade",
          await fs.readFile(path.join(__dirname, "..", "res", "upgrade_b.wasm"))
        )
        .signAndSend();

      // Assert that migrate function is called
      let log = res.receipts_outcome[1].outcome.logs[0];
      expect(log).toEqual("performing arbitrary migration logic");

      res = await contract
        .createTransaction(contract)
        .functionCall("some_new_function", new Uint8Array())
        .signAndSend();
      log = res.receipts_outcome[0].outcome.logs[0];
      expect(log).toEqual("can call some new function now!");
    });
  });
});
