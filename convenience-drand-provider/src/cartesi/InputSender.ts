import { Args, rollups } from "./rollups";
// import { ContractTransactionResponse, ethers } from "ethers";

import { connect } from "./connect";
// import { IInputBox } from "@cartesi/rollups";
import { InputSenderConfig } from "../configs";
import { isBytesLike } from "./utils";
import { stringToBytes } from "viem";

export default class InputSender {
  config: InputSenderConfig;
  // inputBox?: IInputBox

  constructor(config: InputSenderConfig) {
    this.config = config;
  }

  async createInputBox(args: Args) {
    // connect to provider
    console.log(`connecting to ${this.config.rpc}`);
    const connectAccountRequest = connect(
      this.config.rpc,
      this.config.mnemonic,
      this.config.accountIndex
    );

    // const chainId = await result.getChainId();
    // console.log(`connected to chain ${chainId}`);

    const finalArgs = { ...args };
    if (!finalArgs.address) {
      finalArgs.address = this.config.dappAddress;
    }
    // connect to rollups,
    const result = await rollups(connectAccountRequest, finalArgs);

    return result;
  }

  async findOrCreateInputBox(args: Args) {
    // if (this.inputBox) {
    //     return this.inputBox
    // } else {
    //     return this.inputBox = await this.createInputBox(args)
    // }
    return null;
  }

  async sendInput(args: Args) {
    const { payload } = args;
    const dappAddress = this.config.dappAddress || args.address;

    if (!dappAddress) {
      throw new Error("unable to resolve DApp address");
    }

    const inputContract = await this.findOrCreateInputBox(args);

    // const signerAddress = await inputContract.getAddress();
    // console.log(`using account "${signerAddress}"`);

    // use message from command line option, or from user prompt
    console.log(`sending "${payload}" to "${dappAddress}"`);

    // convert string to input bytes (if it's not already bytes-like)
    const inputBytes = isBytesLike(payload) ? payload : stringToBytes(payload);

    // send transaction
    // const tx =  <ContractTransactionResponse>await inputContract.addInput(dappAddress, inputBytes);
    // console.log(`transaction: ${tx.hash}`);
    // console.log("waiting for confirmation...");
    // const receipt = await tx.wait(1);
    // console.log('receipt.transactionHash', receipt?.hash)
    console.log(new Date().toISOString());
  }
}
