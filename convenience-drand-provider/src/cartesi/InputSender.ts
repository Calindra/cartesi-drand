import {
    rollups,
} from "./rollups";
import { ethers } from "ethers";

import {
    connect
} from "./connect";
import { IInputBox } from "@cartesi/rollups";

export default class InputSender {

    /**
     * mnemonic default value
     */
    mnemonic: string = 'test test test test test test test test test test test junk'

    /**
     * default value
     */
    rpc: string = 'http://localhost:8545'

    /**
     * default value
     */
    accountIndex: number = 0

    /**
     * DApp address
     */
    address: string = ''

    inputBox?: IInputBox

    async createInputBox(args: any) {
        // connect to provider
        console.log(`connecting to ${this.rpc}`);
        const { provider, signer } = connect(this.rpc, this.mnemonic, this.accountIndex);

        const network = await provider.getNetwork();
        console.log(`connected to chain ${network.chainId}`);

        // connect to rollups,
        const { inputContract } = await rollups(
            network.chainId,
            signer || provider,
            args
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
        const dappAddress = this.address || args.address

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
        console.log({ receipt })
    };
}
