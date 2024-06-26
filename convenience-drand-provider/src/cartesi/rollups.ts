// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { Signer, Provider } from "ethers";
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
import { networks } from "./networks";
import { Deployment } from "./abi";
import {
    readAddressFromFile,
} from "./utils"
import localhost from "@sunodo/devnet/export/abi/localhost.json"

export interface Args {
    dapp?: string;
    address?: string;
    addressFile?: string;
    deploymentFile?: string;
    payload: string;
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
        }).option("payload", {
            describe: "Payload to send to DApp",
            type: "string",
            default: "0xdeadbeef",
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
): string | undefined => {
    const network = networks[chainId];
    if (network && dapp) {
        return readAddressFromFile(`../deployments/${network.name}/${dapp}.json`);
    }
};


const readDeployment = async (chainId: number, args: Args): Promise<Deployment> => {
    if (args.deploymentFile) {
        const deployment = require(args.deploymentFile);
        if (!deployment) {
            throw new Error(
                `rollups deployment '${args.deploymentFile}' not found`
            );
        }
        return deployment as Deployment;
    } else {
        const network = networks[chainId];
        if (!network) {
            throw new Error(`unsupported chain ${chainId}`);
        }

        if (network.name === "localhost") {
            // const deployment: Deployment = { chainId: chainId.toString(), name: localhost.name, contracts: localhost.contracts };
            // return deployment;
            return localhost;
        }

        const deployment = require(`@cartesi/rollups/export/abi/${network.name}.json`);
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

    // connect to contracts
    const inputContract = IInputBox__factory.connect(
        InputBox.address,
        provider as any
    );
    const outputContract = ICartesiDApp__factory.connect(address, provider as any);
    const erc20Portal = IERC20Portal__factory.connect(
        ERC20Portal.address,
        provider as any
    );
    const erc721Portal = IERC721Portal__factory.connect(
        ERC721Portal.address,
        provider as any
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