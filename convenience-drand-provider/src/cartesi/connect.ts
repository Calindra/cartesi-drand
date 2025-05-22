// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

//import { ethers, Signer, JsonRpcProvider, Provider } from "ethers";
import { Argv } from "yargs";
import { http, type HDAccount } from "viem";
import { mnemonicToAccount } from "viem/accounts";
export interface Args {
  rpc: string;
  mnemonic?: string;
  accountIndex: number;
}

const HARDHAT_DEFAULT_MNEMONIC =
  "test test test test test test test test test test test junk";

const HARDHAT_DEFAULT_RPC_URL = "http://localhost:8545";

/**
 * Builder for provider connection
 * @param yargs yargs instance
 * @param transactional indicate if will need to sign transactions, hence MNEMONIC is required
 * @returns Argv instance with all options
 */
export const builder = <T>(
  yargs: Argv<T>,
  transactional: boolean = false
): Argv<Args & T> => {
  return yargs
    .option("rpc", {
      describe: "JSON-RPC provider URL",
      type: "string",
      default: process.env.RPC_URL || HARDHAT_DEFAULT_RPC_URL,
    })
    .option("mnemonic", {
      describe: "Wallet mnemonic",
      type: "string",
      default: process.env.MNEMONIC || HARDHAT_DEFAULT_MNEMONIC,
      demandOption: transactional, // required if need to send transactions
    })
    .option("accountIndex", {
      describe: "Account index from mnemonic",
      type: "number",
      default: 0,
    });
};

export interface ConnectAccount {
  account?: HDAccount;
  transport: ReturnType<typeof http>;
}

/**
 * Connects to a JSON-RPC provider and optionally creates an HD wallet account for signing transactions.
 * @param rpc - The JSON-RPC provider URL.
 * @param mnemonic - (Optional) Mnemonic phrase to derive the wallet account.
 * @param accountIndex - (Optional) Index of the account to derive from the mnemonic (defaults to 0).
 * @returns An object containing the transport and, if a mnemonic is provided, the derived HD account.
 */
export const connect = (
  rpc: string,
  mnemonic?: string,
  accountIndex?: number
): ConnectAccount => {
  let account: HDAccount | undefined;

  if (mnemonic) {
    // create account from mnemonic
    account = mnemonicToAccount(mnemonic, {
      addressIndex: accountIndex ?? 0,
      // path: `m/44'/60'/0'/0/${accountIndex ?? 0}`,
    });
  }

  return {
    account,
    transport: http(rpc),
  };
};
