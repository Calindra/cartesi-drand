import { IInputBox__factory } from "@cartesi/rollups";
// We'll use ethers to interact with the Ethereum network and our contract
import { ethers, Signer } from "ethers";
import InputBox from "../deployments/InputBox.json";
import DApp from "../deployments/dapp.json"

// const CARTESI_INSPECT_ENDPOINT = 'http://localhost:5005/inspect'
const CARTESI_INSPECT_ENDPOINT = 'https://5005-cartesi-rollupsexamples-mk3ozp0tglt.ws-us104.gitpod.io/inspect'
export class Cartesi {
    static async sendInput(payload: any, signer: any, provider: any) {

        const network = await provider.getNetwork();
        console.log(`connected to chain ${network.chainId}`);

        // connect to rollups,
        const inputContract = IInputBox__factory.connect(
            InputBox.address,
            signer
        );
        const signerAddress = await signer.getAddress();
        console.log(`using account "${signerAddress}"`);

        // use message from command line option, or from user prompt
        console.log(`sending "${JSON.stringify(payload)}"`);

        // convert string to input bytes (if it's not already bytes-like)
        const inputBytes = ethers.toUtf8Bytes(JSON.stringify({
            input: payload
        }));

        // send transaction
        const dappAddress = DApp.address;// '0x142105FC8dA71191b3a13C738Ba0cF4BC33325e2'
        const tx: any = await inputContract.addInput(dappAddress, inputBytes);
        // const tx: any = await inputContract.addInput(dappAddress, inputBytes);
        console.log(`transaction: ${tx.hash}`);
        console.log("waiting for confirmation...");
        const receipt = await tx.wait(1);
        console.log(JSON.stringify(receipt))
        // find reference to notice from transaction receipt
        // const inputKeys = getInputKeys(receipt);
        // console.log(
        //     `input ${inputKeys.input_index} added`
        // );
    }

    static hex2a(hex: string) {
        var str = '';
        for (var i = 0; i < hex.length; i += 2) {
            var v = parseInt(hex.substring(i, i + 2), 16);
            if (v) str += String.fromCharCode(v);
        }
        return str;
    }

    static async inspectWithJson(json: any) {
        const jsonString = JSON.stringify({ input: json });
        const jsonEncoded = encodeURIComponent(jsonString)
        const response = await fetch(`${CARTESI_INSPECT_ENDPOINT}/${jsonEncoded}`);
        const data = await response.json();
        console.log(data)
        if (!data.reports?.length) {
            return null
        }
        const payload = Cartesi.hex2a(data.reports[0].payload.replace(/^0x/, ""))
        console.log({ payload })
        return JSON.parse(payload)
    }
}
