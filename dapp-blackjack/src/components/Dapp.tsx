import React from "react";

// We'll use ethers to interact with the Ethereum network and our contract
import { BrowserProvider, ethers } from "ethers";

// We import the contract's artifacts and address here, as we are going to be
// using them with ethers
// import TokenArtifact from "../contracts/Token.json";
// import contractAddress from "../contracts/contract-address.json";

// All the logic of this dapp is contained in the Dapp component.
// These other components are just presentational ones: they don't have any
// logic. They just render HTML.
import { NoWalletDetected } from "./NoWalletDetected";
import { ConnectWallet } from "./ConnectWallet";
// import { Transfer } from "./Transfer";
// import { TransactionErrorMessage } from "./TransactionErrorMessage";
// import { WaitingForTransactionMessage } from "./WaitingForTransactionMessage";
// import { NoTokensMessage } from "./NoTokensMessage";
import { Cartesi } from "../cartesi/Cartesi";
import { Card10 } from "./cards/Card10";
import { CardK } from "./cards/CardK";
import { CardA } from "./cards/CardA";
import { Card2 } from "./cards/Card2";
import { Card3 } from "./cards/Card3";
import { Card4 } from "./cards/Card4";
import { Card5 } from "./cards/Card5";
import { Card6 } from "./cards/Card6";
import { Card7 } from "./cards/Card7";
import { Card8 } from "./cards/Card8";
import { Card9 } from "./cards/Card9";
import { CardJ } from "./cards/CardJ";
import { CardQ } from "./cards/CardQ";
import { CardBack } from "./cards/CardBack";
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
                <div>
                    <ConnectWallet
                        connectWallet={() => this._connectWallet()}
                        networkError={this.state.networkError}
                        dismiss={() => this._dismissNetworkError()}
                    />
                    <div>
                        <CardBack />
                        <CardA suit="hearts" />
                        <Card2 suit="clubs" />
                        <Card3 suit="diamonds" />
                        <Card4 suit="clubs" />
                        <Card5 suit="clubs" />
                        <Card6 suit="clubs" />
                        <Card7 suit="clubs" />
                        <Card8 suit="clubs" />
                        <Card9 suit="clubs" />
                        <Card10 suit="spades" />
                        <CardJ suit="spades" />
                        <CardQ suit="spades" />
                        <CardK suit="spades" />
                    </div>
                </div>
            );
        }

        // If the token data or the user's balance hasn't loaded yet, we show
        // a loading component.
        if (!this.state.games) {
            // return <Loading />;
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
                            Welcome <b>{this.state.player?.name ?? this.state.selectedAddress}</b>.
                        </p>
                        <button onClick={() => {
                            this._newPlayer()
                        }}>New Player</button>
                        <button onClick={() => {
                            this._joinGame("1")
                        }}>Join Game</button>
                        <button onClick={() => {
                            this._startGame("1")
                        }}>Start Game</button>
                        <button onClick={() => {
                            this._chooseHit("1")
                        }}>Hit</button>
                        <button onClick={() => {
                            this._chooseStand("1")
                        }}>Stand</button>
                        <button onClick={() => {
                            this._showHands("1")
                        }}>Show hands</button>
                    </div>
                </div>

                <hr />

                <div className="row">
                    <div className="col-12">
                        {JSON.stringify(this.state.hands || {})}
                    </div>
                </div>
            </div>
        );
    }
    private async _chooseStand(game_id: string) {
        await Cartesi.sendInput({ action: "stand", game_id }, this._signer, this._provider)
    }

    private async _chooseHit(game_id: string) {
        await Cartesi.sendInput({ action: "hit", game_id }, this._signer, this._provider)
    }

    private async _startGame(game_id: string) {
        await Cartesi.sendInput({ action: "start_game", game_id }, this._signer, this._provider)
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
        this._loadUserData(userAddress);
        // this._startPollingData();
    }
    private async _loadUserData(userAddress: any) {
        console.log('read player...')
        const player = await Cartesi.inspectWithJson({ "action": "show_player", "address": userAddress })
        this.setState({ player })
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
            name: 'Oshiro'
        }, this._signer, this._provider)
    }

    async _joinGame(game_id: string) {
        await Cartesi.sendInput({
            action: 'join_game',
            game_id
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
            await this._showHands("1");
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

    async _showHands(game_id: string) {
        console.log('show hands...')
        const hands = await Cartesi.inspectWithJson({ action: 'show_hands', game_id })
        if (hands) {
            this.setState({ hands })
        }
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


