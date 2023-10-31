import {
    Args,
    rollups,
} from "./rollups.ts";
import { ContractTransactionResponse, ethers } from "ethers";

import {
    connect
} from "./connect.ts";
import { IInputBox } from "@cartesi/rollups";
import { InputSenderConfig } from "../configs.ts";

export default class InputSender {

    config: InputSenderConfig
    inputBox?: IInputBox

    constructor(config: InputSenderConfig) {
        this.config = config
    }

    async createInputBox(args: Args) {
        // connect to provider
        console.log(`connecting to ${this.config.rpc}`);
        const { provider, signer } = connect(this.config.rpc, this.config.mnemonic, this.config.accountIndex);

        const network = await provider.getNetwork();
        console.log(`connected to chain ${network.chainId}`);

        const finalArgs = { ...args }
        if (!finalArgs.address) {
            finalArgs.address = this.config.dappAddress
        }
        // connect to rollups,
        let chainId = Number(network.chainId)
        const { inputContract } = await rollups(
            chainId,
            signer || provider,
            finalArgs
        );
        return inputContract
    }

    async findOrCreateInputBox(args: Args) {
        if (this.inputBox) {
            return this.inputBox
        } else {
            return this.inputBox = await this.createInputBox(args)
        }
    }

    async sendInput(args: Args) {
        const { payload } = args;
        const dappAddress = this.config.dappAddress ?? args.address

        const inputContract = await this.findOrCreateInputBox(args)

        const signerAddress = await inputContract.getAddress();
        console.log(`using account "${signerAddress}"`);

        // use message from command line option, or from user prompt
        console.log(`sending "${payload}" to "${dappAddress}"`);

        // convert string to input bytes (if it's not already bytes-like)
        const inputBytes = ethers.isBytesLike(payload)
            ? payload
            : ethers.toUtf8Bytes(payload);


        // send transaction
        const tx =  <ContractTransactionResponse>await inputContract.addInput(dappAddress, inputBytes);
        console.log(`transaction: ${tx.hash}`);
        console.log("waiting for confirmation...");
        const receipt = await tx.wait(1);
        console.log('receipt.transactionHash', receipt?.hash)
        console.log(new Date().toISOString())
    };
}
