import fs from "fs";
import { Contract, Deployment } from "./abi";

/**
 * Read address from json file
 * @param path Path of file with address in json file
 * @returns address or undefined if file does not exist
 */
export const readAddressFromFile = (path: string) => {
    try {
        const data = readObjectFromFile(path);

        if (!data) {
            throw new Error(`File ${path} is empty`);
        }

        const address = data.address;

        if (typeof address !== "string") {
            throw TypeError(`address is not a string: ${address}`);
        }

        return address;
    } catch (e) {
        console.error(`Error reading address from ${path}:`, e);
    }

    return null;
};

/**
 * Read object from json file
 * @param path Path of file with object in json file
 * @returns object or undefined if file does not exist
 * @throws Error if file exists but is not valid json
 */
export const readObjectFromFile = (path: string) => {
    if (fs.existsSync(path)) {
        const file = fs.readFileSync(path, "utf8");
        const data: Record<string, unknown> = JSON.parse(file);
        return data;
    }

    return null;
};

/**
 * Read contract from json file
 * @param path Path of file with Contract in json file
 * @returns The Contract or undefined if file does not exist
 */
export const readContractFromFile = (path: string): Contract | null => {
    try {
        const data = readObjectFromFile(path)

        if (data) {
            return Contract.fromObj(data);
        }
    } catch (e) {
        console.error(`Error reading contract from ${path}:`, e);
    }

    return null
}

export const readDeploymentFromFile = (path: string): Deployment | null => {
    try {
        const data = readObjectFromFile(path)

        if (data) {
            return Deployment.fromObj(data);
        }
    } catch (e) {
        console.error(`Error reading deployment from ${path}:`, e);
    }

    return null
}

export const readAllContractsFromDir = (...paths: string[]): Record<string, Contract> => {
    const contracts: Record<string, Contract> = {};
    for (let i = 0; i < paths.length; i++) {
        let path = paths[i];
        if (path && fs.existsSync(path)) {
            const deployContents: fs.Dirent[] = fs.readdirSync(path, { withFileTypes: true })
            deployContents.forEach(deployEntry => {
                if (deployEntry.isFile()) {
                    const filename = deployEntry.name;
                    if (filename.endsWith(".json") && filename !== "dapp.json") {
                        const contractName = filename.substring(0, filename.lastIndexOf("."));
                        const contract = readContractFromFile(`${path}/${filename}`);

                        if (contract) {
                            contracts[contractName] = contract
                        } else {
                            console.error(`Error reading contract from ${path}/${filename}`);
                        }

                    }
                }
            });
        }
    }
    return contracts
}