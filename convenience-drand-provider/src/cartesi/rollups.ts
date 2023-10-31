// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import type { Signer, Provider } from "ethers";

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
import { Argv } from "yargs";
import { networks } from "./networks.ts";
import { Deployment, Contract } from "./abi.ts";
import {
    readAddressFromFile,
    readAllContractsFromDir,
    readDeploymentFromFile
} from "./utils.ts"
import { getProvider } from "./adapter.ts";

export interface ArgsBuilder {
    dapp: string;
    address?: string;
    addressFile?: string;
    deploymentFile?: string;
}

export interface Args {
    payload: string;
    address?: string;
    deploymentFile?: string;
}

interface Contracts {
    dapp: string;
    inputContract: IInputBox;
    outputContract: ICartesiDApp;
    erc20Portal: IERC20Portal;
    erc721Portal: IERC721Portal;
    deployment: Deployment

}

/**
 * Builder for args for connecting to Rollups instance
 * @param yargs yargs instance
 * @returns Argv instance with all options
 */
export const builder = <T>(yargs: Argv<T>) => {
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
        });
};


/**
 * Read address from file located at deployment path
 * @param dapp DApp name
 * @param chainId number of chain id of connected network
 * @returns address or undefined if can't resolve network name of file does not exist
 */
const readDApp = (
    dapp: string | undefined,
    chainId: number
): string | null => {
    const network = networks[chainId];
    if (network && dapp) {
        return readAddressFromFile(`../deployments/${network.name}/${dapp}.json`);
    }
    return null;
};


const readDeployment = async (chainId: number, args: Args): Promise<Deployment> => {
    if (args.deploymentFile) {
        const deployment = readDeploymentFromFile(args.deploymentFile);
        if (!deployment) {
            throw new Error(
                `rollups deployment '${args.deploymentFile}' not found`
            );
        }
        return deployment;
    } else {
        const network = networks[chainId];
        if (!network) {
            throw new Error(`unsupported chain ${chainId}`);
        }

        if (network.name === "localhost") {

            const contracts: Record<string, Contract> =
                readAllContractsFromDir("../deployments/localhost",
                    "../common-contracts/deployments/localhost");

            const deployment = { chainId: chainId.toString(), name: "localhost", contracts: contracts };
            return deployment as Deployment;
        }

        const deployment = await import(`@cartesi/rollups/export/abi/${network.name}.json`);
        // const deployment = require(`@cartesi/rollups/export/abi/${network.name}.json`);
        if (!deployment) {
            throw new Error(`rollups not deployed to network ${network.name}`);
        }
        return deployment as Deployment;
    }
};

/**
 * Connect to instance of Rollups application
 * @param chainId number of chain id of connected network
 * @param provider provider or signer of connected network
 * @param args args for connection logic
 * @returns Connected rollups contracts
 */
export const rollups = async (
    chainId: number,
    provider: Provider | Signer,
    args: Args
): Promise<Contracts> => {
    const address = args.address;

    if (!address) {
        throw new Error("unable to resolve DApp address");
    }

    const deployment = await readDeployment(chainId, args);
    const InputBox = deployment.contracts["InputBox"];
    const ERC20Portal = deployment.contracts["ERC20Portal"];
    const ERC721Portal = deployment.contracts["ERC721Portal"];

    const providerOut = getProvider(provider);

    // connect to contracts
    const inputContract = IInputBox__factory.connect(
        // InputBox.address,
        "0x59b22D57D4f067708AB0c00552767405926dc768",
        providerOut
    );
    const outputContract = ICartesiDApp__factory.connect(address, providerOut);
    const erc20Portal = IERC20Portal__factory.connect(
        ERC20Portal.address,
        providerOut
    );
    const erc721Portal = IERC721Portal__factory.connect(
        ERC721Portal.address,
        providerOut
    );

    return {
        dapp: address,
        inputContract,
        outputContract,
        erc20Portal,
        erc721Portal,
        deployment
    };
};