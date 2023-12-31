// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { ethers, Signer, JsonRpcProvider, Provider } from "ethers";
import { Argv } from "yargs";

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

export type Connection = {
    provider: Provider;
    signer?: Signer;
};

/**
 * Connect to a JSON-RPC provider and return a signer or provider
 * @param rpc JSON-RPC provider URL
 * @param mnemonic optional mnemonic to sign transactions
 * @param accountIndex account index of mnemonic (default to 0)
 * @returns signer if mnemonic is provided, provider otherwise
 */
export const connect = (
    rpc: string,
    mnemonic?: string,
    accountIndex?: number
): Connection => {
    // connect to JSON-RPC provider
    const provider = new JsonRpcProvider(rpc);

    // create signer to be used to send transactions
    let signer: Connection['signer'] | undefined;

    if (mnemonic) {
        const path = `m/44'/60'/0'/0/${accountIndex ?? 0}`;
        const mneu = ethers.Mnemonic.fromPhrase(mnemonic);
        const hdNode = ethers.HDNodeWallet.fromMnemonic(mneu, path);

        signer = hdNode.connect(provider);
    }

    return {
        provider,
        signer,
    };
};