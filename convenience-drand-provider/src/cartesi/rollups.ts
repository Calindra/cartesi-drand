// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// import { Signer, Provider } from "ethers";

/**
import {
    IInputBox,
    IInputBox__factory,
    ICartesiDApp,
    ICartesiDApp__factory,
    IERC20Portal,
    IERC20Portal__factory,
    IERC721Portal,
    IERC721Portal__factory,
} from "@cartesi/rollups";
*/
import type { Argv } from "yargs";
import { networks } from "./networks";
import { checkIfIsDeployment, type Contract, type Deployment } from "./abi";
import { readFile } from "node:fs/promises";
import { createWalletClient } from "viem";
import type { ConnectAccount } from "./connect";
import { walletActionsL1 } from "@cartesi/viem";

export interface Args {
  dapp?: string;
  address?: string;
  addressFile?: string;

  deploymentFile?: string;
  payload: string;
}

interface Contracts {
  dapp: string;
  deployment: Deployment;
  client: any;
}

/**
 * Builder for args for connecting to Rollups instance
 * @param yargs yargs instance
 * @returns Argv instance with all options
 */
export const builder = <T>(yargs: Argv<T>): Argv<Args & T> => {
  return yargs
    .option("dapp", {
      describe: "DApp name",
      type: "string",
      default: "dapp",
    })
    .option("address", {
      describe: "Rollups contract address",
      type: "string",
    })
    .option("addressFile", {
      describe: "File with rollups contract address",
      type: "string",
    })
    .option("deploymentFile", {
      describe: "JSON file with deployment of rollups contracts",
      type: "string",
    })
    .option("payload", {
      describe: "Payload to send to DApp",
      type: "string",
      default: "0xdeadbeef",
    });
};

const readDeployment = async (deploymentFile: string): Promise<Deployment> => {
  const data = await readFile(deploymentFile);
  const deployment = JSON.parse(data.toString("utf-8"));
  if (!checkIfIsDeployment(deployment)) {
    throw new Error(`invalid deployment file ${deploymentFile}`);
  }
  return deployment;
};

/**
 * Connect to instance of Rollups application
 * @param chainId number of chain id of connected network
 * @param provider provider or signer of connected network
 * @param args args for connection logic
 * @returns Connected rollups contracts
 */
export const rollups = async (
  walletRequest: ConnectAccount,
  args: Args
): Promise<Contracts> => {
  const address = args.address;

  if (!address) {
    throw new Error("unable to resolve DApp address");
  }

  const wallet = createWalletClient({
    transport: walletRequest.transport,
    account: walletRequest.account,
  }).extend(walletActionsL1());

  const chainId = await wallet.getChainId();
  const network = networks.get(chainId);
  if (!network) {
    throw new Error(`chain ${chainId} not supported`);
  }

  if (args.deploymentFile) {
    const deployment = await readDeployment(args.deploymentFile);

    return {
      dapp: address,
      deployment,
      client: wallet,
    };
  }

  return {
    dapp: address,
    deployment: {
      name: network.name,
      chainId: network.chain.id.toString(),
      contracts: {},
    },
    client: wallet,
  };
};
