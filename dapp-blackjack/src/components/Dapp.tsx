import React from "react";

// We'll use ethers to interact with the Ethereum network and our contract
import { BrowserProvider, parseUnits, ethers, isBytesLike, Signer } from "ethers";

// We import the contract's artifacts and address here, as we are going to be
// using them with ethers
// import TokenArtifact from "../contracts/Token.json";
// import contractAddress from "../contracts/contract-address.json";

// All the logic of this dapp is contained in the Dapp component.
// These other components are just presentational ones: they don't have any
// logic. They just render HTML.
import { NoWalletDetected } from "./NoWalletDetected";
import { ConnectWallet } from "./ConnectWallet";
import { Loading } from "./Loading";
// import { Transfer } from "./Transfer";
// import { TransactionErrorMessage } from "./TransactionErrorMessage";
// import { WaitingForTransactionMessage } from "./WaitingForTransactionMessage";
// import { NoTokensMessage } from "./NoTokensMessage";
import { Provider } from "@ethersproject/providers";
import { ICartesiDApp, ICartesiDApp__factory, IERC20Portal, IERC20Portal__factory, IERC721Portal, IERC721Portal__factory, IInputBox, IInputBox__factory } from "@cartesi/rollups";
import InputBox from "../deployments/InputBox.json";
import ERC721Portal from "../deployments/ERC721Portal.json";
import ERC20Portal from "../deployments/ERC20Portal.json";
// This is the default id used by the Hardhat Network
const HARDHAT_NETWORK_ID = '31337';

// This is an error code that indicates that the user canceled a transaction
const ERROR_CODE_TX_REJECTED_BY_USER = 4001;

// This component is in charge of doing these things:
//   1. It connects to the user's wallet
//   2. Initializes ethers and the Token contract
//   3. Polls the user balance to keep it updated.
//   4. Transfers tokens by sending transactions
//   5. Renders the whole application
//
// Note that (3) and (4) are specific of this sample application, but they show
// you how to keep your Dapp and contract's state in sync,  and how to send a
// transaction.
export class Dapp extends React.Component {
    state: any
    initialState: {
        // The info of the token (i.e. It's Name and symbol)
        tokenData: undefined;
        // The user's address and balance
        selectedAddress: undefined; balance: undefined;
        // The ID about transactions being sent, and any possible error with them
        txBeingSent: undefined; transactionError: undefined; networkError: undefined;
    };
    private _provider: any;
    private _token: ethers.Contract | any;
    private _pollDataInterval: any;
    private _signer: any;
    constructor(props) {
        super(props);

        // We store multiple things in Dapp's state.
        // You don't need to follow this pattern, but it's an useful example.
        this.initialState = {
            // The info of the token (i.e. It's Name and symbol)
            tokenData: undefined,
            // The user's address and balance
            selectedAddress: undefined,
            balance: undefined,
            // The ID about transactions being sent, and any possible error with them
            txBeingSent: undefined,
            transactionError: undefined,
            networkError: undefined,
        };

        this.state = this.initialState;
    }

    render() {
        // Ethereum wallets inject the window.ethereum object. If it hasn't been
        // injected, we instruct the user to install a wallet.
        if ((window as any).ethereum === undefined) {
            return <NoWalletDetected />;
        }

        // The next thing we need to do, is to ask the user to connect their wallet.
        // When the wallet gets connected, we are going to save the users's address
        // in the component's state. So, if it hasn't been saved yet, we have
        // to show the ConnectWallet component.
        //
        // Note that we pass it a callback that is going to be called when the user
        // clicks a button. This callback just calls the _connectWallet method.
        if (!this.state.selectedAddress) {
            return (
                <ConnectWallet
                    connectWallet={() => this._connectWallet()}
                    networkError={this.state.networkError}
                    dismiss={() => this._dismissNetworkError()}
                />
            );
        }

        // If the token data or the user's balance hasn't loaded yet, we show
        // a loading component.
        if (!this.state.games) {
            return <Loading />;
        }

        // If everything is loaded, we render the application.
        return (
            <div className="container p-4">
                <div className="row">
                    <div className="col-12">
                        <h1>
                            Black Jack
                        </h1>
                        <p>
                            Welcome <b>{this.state.selectedAddress}</b>.
                        </p>
                        <button onClick={() => {
                            this._newPlayer()
                        }}>New Player</button>
                        <button onClick={() => {
                            this._joinGame()
                        }}>Join Game</button>
                    </div>
                </div>

                <hr />

                <div className="row">
                    <div className="col-12">
                        {JSON.stringify(this.state.games.games)}
                    </div>
                </div>
            </div>
        );
    }

    componentWillUnmount() {
        // We poll the user's balance, so we have to stop doing that when Dapp
        // gets unmounted
        this._stopPollingData();
    }

    async _connectWallet() {
        // This method is run when the user clicks the Connect. It connects the
        // dapp to the user's wallet, and initializes it.

        // To connect to the user's wallet, we have to run this method.
        // It returns a promise that will resolve to the user's address.
        const [selectedAddress] = await (window as any).ethereum.request({ method: 'eth_requestAccounts' });

        // Once we have the address, we can initialize the application.

        // First we check the network
        this._checkNetwork();

        this._initialize(selectedAddress);

        // We reinitialize it whenever the user changes their account.
        (window as any).ethereum.on("accountsChanged", ([newAddress]) => {
            this._stopPollingData();
            // `accountsChanged` event can be triggered with an undefined newAddress.
            // This happens when the user removes the Dapp from the "Connected
            // list of sites allowed access to your addresses" (Metamask > Settings > Connections)
            // To avoid errors, we reset the dapp state 
            if (newAddress === undefined) {
                return this._resetState();
            }

            this._initialize(newAddress);
        });
    }

    _initialize(userAddress) {
        // This method initializes the dapp

        // We first store the user's address in the component's state
        this.setState({
            selectedAddress: userAddress,
        });

        // Then, we initialize ethers, fetch the token's data, and start polling
        // for the user's balance.

        // Fetching the token data and the user's balance are specific to this
        // sample project, but you can reuse the same initialization pattern.
        this._initializeEthers();
        this._readGames();
        // this._startPollingData();
    }

    async _initializeEthers() {
        // We first initialize ethers by creating a provider using window.ethereum
        // (window as any)._ethers = ethers;
        this._provider = new BrowserProvider((window as any).ethereum);

        // this._provider = new ethers.BrowserProvider()

        // Then, we initialize the contract using that provider and the token's
        // artifact. You can do this same thing with your contracts.
        // this._token = new ethers.Contract(
        //     contractAddress.Token,
        //     TokenArtifact.abi,
        //     this._provider.getSigner(0)
        // );

        // It also provides an opportunity to request access to write
        // operations, which will be performed by the private key
        // that MetaMask manages for the user.
        this._signer = await this._provider.getSigner();
    }

    async _newPlayer() {
        console.log('new player')
        await Cartesi.sendInput({
            action: 'new_player',
            name: 'Bob'
        }, this._signer, this._provider)
    }

    async _joinGame() {
        await Cartesi.sendInput({
            action: 'join_game',
            game_id: '1'
        }, this._signer, this._provider)
    }

    // The next two methods are needed to start and stop polling data. While
    // the data being polled here is specific to this example, you can use this
    // pattern to read any data from your contracts.
    //
    // Note that if you don't need it to update in near real time, you probably
    // don't need to poll it. If that's the case, you can just fetch it when you
    // initialize the app, as we do with the token data.
    async _startPollingData() {
        try {
            // We run it once immediately so we don't have to wait for it
            await this._showHands();
        } catch (e) {
            console.error(e);
        }
        this._pollDataInterval = setTimeout(() => this._startPollingData(), 3000);
    }

    _stopPollingData() {
        clearInterval(this._pollDataInterval);
        this._pollDataInterval = undefined;
    }

    async _readGames() {
        console.log('read game...')
        const games = await Cartesi.inspectWithJson({ action: 'show_games' })
        console.log(games)
        this.setState({ games })
    }

    async _showHands() {
        console.log('show hands...')
        const hands = await Cartesi.inspectWithJson({ action: 'show_hands', game_id: '1' })
        console.log(hands)
        this.setState({ hands })
    }

    // This method just clears part of the state.
    _dismissTransactionError() {
        this.setState({ transactionError: undefined });
    }

    // This method just clears part of the state.
    _dismissNetworkError() {
        this.setState({ networkError: undefined });
    }

    // This is an utility method that turns an RPC error into a human readable
    // message.
    _getRpcErrorMessage(error) {
        if (error.data) {
            return error.data.message;
        }

        return error.message;
    }

    // This method resets the state
    _resetState() {
        this.setState(this.initialState);
    }

    async _switchChain() {
        const chainIdHex = `0x${(+HARDHAT_NETWORK_ID).toString(16)}`
        await (window as any).ethereum.request({
            method: "wallet_switchEthereumChain",
            params: [{ chainId: chainIdHex }],
        });
        await this._initialize(this.state.selectedAddress);
    }

    // This method checks if the selected network is Localhost:8545
    _checkNetwork() {
        if ((window as any).ethereum.networkVersion !== HARDHAT_NETWORK_ID) {
            this._switchChain();
        }
    }
}


// const CARTESI_INSPECT_ENDPOINT = 'http://localhost:5005/inspect'
const CARTESI_INSPECT_ENDPOINT = 'https://5005-cartesi-rollupsexamples-mk3ozp0tglt.ws-us104.gitpod.io/inspect'
class Cartesi {
    static async sendInput(payload: any, signer: any, provider: any) {

        const network = await provider.getNetwork();
        console.log(`connected to chain ${network.chainId}`);

        // connect to rollups,
        const { dapp, inputContract } = await Cartesi.rollups(
            network.chainId,
            signer,
        );

        const signerAddress = await signer.getAddress();
        console.log(`using account "${signerAddress}"`);

        // use message from command line option, or from user prompt
        console.log(`sending "${JSON.stringify(payload)}"`);

        // convert string to input bytes (if it's not already bytes-like)
        const inputBytes = isBytesLike(payload)
            ? payload
            : ethers.toUtf8Bytes(payload);

        // send transaction
        const tx: any = await inputContract.addInput(dapp, inputBytes);
        console.log(`transaction: ${tx.hash}`);
        console.log("waiting for confirmation...");
        const receipt = await tx.wait(1);
        console.log(receipt)
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

    /**
 * Connect to instance of Rollups application
 * @param chainId number of chain id of connected network
 * @param provider provider or signer of connected network
 * @param args args for connection logic
 * @returns Connected rollups contracts
 */
    static async rollups(
        chainId: number,
        provider: Provider | Signer,
    ): Promise<Contracts> {
        const address = '0x142105FC8dA71191b3a13C738Ba0cF4BC33325e2'

        if (!address) {
            throw new Error("unable to resolve DApp address");
        }

        // const deployment = readDeployment(chainId, args);
        // const InputBox = deployment.contracts["InputBox"];
        // const ERC20Portal = deployment.contracts["ERC20Portal"];
        // const ERC721Portal = deployment.contracts["ERC721Portal"];

        // connect to contracts
        const inputContract = IInputBox__factory.connect(
            InputBox.address,
            provider
        );
        const outputContract = ICartesiDApp__factory.connect(address, provider);
        const erc20Portal = IERC20Portal__factory.connect(
            ERC20Portal.address,
            provider
        );
        const erc721Portal = IERC721Portal__factory.connect(
            ERC721Portal.address,
            provider
        );


        return {
            dapp: address,
            inputContract,
            outputContract,
            erc20Portal,
            erc721Portal,
            // deployment
        };
    };
}

interface Contracts {
    dapp: string;
    inputContract: IInputBox;
    outputContract: ICartesiDApp;
    erc20Portal: IERC20Portal;
    erc721Portal: IERC721Portal;
    // deployment: Deployment
}

export type Contract = {
    address: string;
    abi: any; // XXX: type it more? or any an existing package, like 'abitype'
};

export type Deployment = {
    name: string;
    chainId: string;
    contracts: Record<string, Contract>;
};