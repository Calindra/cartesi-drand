import {
    rollups,
} from "./rollups";
import { ethers } from "ethers";

import {
    connect
} from "./connect";
import { IInputBox } from "@cartesi/rollups";
import { InputSenderConfig } from "../configs";

export default class InputSender {

    config: InputSenderConfig
    inputBox?: IInputBox
    
    constructor(config: InputSenderConfig) {
        this.config = config
    }

    async createInputBox(args: any) {
        // connect to provider
        console.log(`connecting to ${this.config.rpc}`);
        const { provider, signer } = connect(this.config.rpc, this.config.mnemonic, this.config.accountIndex);

        const network = await provider.getNetwork();
        console.log(`connected to chain ${network.chainId}`);

        const finalArgs = { ...args }
        if (!finalArgs.address) {
            finalArgs.address = this.config.dAppAddress
        }
        // connect to rollups,
        const { inputContract } = await rollups(
            network.chainId,
            signer || provider,
            finalArgs
        );
        return inputContract
    }

    async findOrCreateInputBox(args: any) {
        if (this.inputBox) {
            return this.inputBox
        } else {
            return this.inputBox = await this.createInputBox(args)
        }
    }

    async sendInput(args: any) {
        const { payload } = args;
        const dappAddress = this.config.dAppAddress || args.address

        const inputContract = await this.findOrCreateInputBox(args)

        const signerAddress = await inputContract.signer.getAddress();
        console.log(`using account "${signerAddress}"`);

        // use message from command line option, or from user prompt
        console.log(`sending "${payload}" to "${dappAddress}"`);

        // convert string to input bytes (if it's not already bytes-like)
        const inputBytes = ethers.utils.isBytesLike(payload)
            ? payload
            : ethers.utils.toUtf8Bytes(payload);


        // send transaction
        const tx = await inputContract.addInput(dappAddress, inputBytes);
        console.log(`transaction: ${tx.hash}`);
        console.log("waiting for confirmation...");
        const receipt = await tx.wait(1);
        console.log('receipt.transactionHash', receipt.transactionHash)
        console.log(new Date().toISOString())
    };
}
