import React from "react";

// We'll use ethers to interact with the Ethereum network and our contract
import { BrowserProvider, Signer, ethers } from "ethers";

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
import { Card } from "./cards/Card";
import { SuitType } from "./cards/Suit";
// This is the default id used by the Hardhat Network
const HARDHAT_NETWORK_ID = '31337';
const HARDHAT_NETWORK_HEX = `0x${(+HARDHAT_NETWORK_ID).toString(16)}`;

// This is an error code that indicates that the user canceled a transaction
const ERROR_CODE_TX_REJECTED_BY_USER = 4001;

// This is for specific domain
interface GameData {
    gameIdSelected: string | null,
    games?: {
        id: string,
        players: number,
    }[],
    player: {
        name: string,
        address: string,
        joined: string[],
        playing: string[],
    } | null,
    hands: {
        players: {
            name: string,
            points: number,
            hand: SuitType[],
        }[],
    },
    isLoading: boolean,
    gameJoined: boolean,
    gamePlaying: boolean,
}

type ReponseHands = GameData['hands'] & ({
    scoreboard: {
        id: string,
        game_id: string,
        winner: string,
        players: string[]
    }, is_finished: true
} | { is_finished: false });

type ErrorRpc = { data: { message: string } } | { message: string };

interface DappState extends GameData {
    // The info of the token (i.e. It's Name and symbol)
    tokenData?: unknown,
    // The user's address and balance
    selectedAddress?: string,
    balance?: unknown,
    // The ID about transactions being sent, and any possible error with them
    txBeingSent?: unknown,
    transactionError?: unknown,
    networkError?: string,
}

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
export class Dapp extends React.Component<{}, DappState> {
    private ONCE = true;

    initialState: {
        // The info of the token (i.e. It's Name and symbol)
        tokenData: undefined;
        // The user's address and balance
        selectedAddress: undefined; balance: undefined;
        // The ID about transactions being sent, and any possible error with them
        txBeingSent: undefined; transactionError: undefined; networkError: undefined;
    };
    private _provider?: BrowserProvider;
    private _token?: ethers.Contract;
    private _pollDataInterval?: NodeJS.Timeout;
    private _signer?: Signer;
    private static readonly POLL_TIME_MS = 10_000;

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

        this.state = {
            ...this.initialState,
            games: undefined,
            hands: {
                players: [],
            },
            gameIdSelected: null,
            player: null,
            isLoading: false,
            gameJoined: false,
            gamePlaying: false,
        };
    }

    render() {
        // Ethereum wallets inject the window.ethereum object. If it hasn't been
        // injected, we instruct the user to install a wallet.
        if (window.ethereum === undefined) {
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
                </div>
            );
        }

        // If the token data or the user's balance hasn't loaded yet, we show
        // a loading component.
        if (this.state.isLoading) {
            return <h1 className="text-lg">Loading...</h1>
            // return <progress />;
        }

        const noGameSelected = this.state.gameIdSelected === null;
        const gamePlaying = this.state.gamePlaying;
        const noPlayerSelected = this.state.player === null;
        const noPlayerJoined = !this.state.gameJoined;

        const actions = [
            {
                id: 'show_games',
                label: 'Show Games',
                action: this._showGames.bind(this),
                disabled: gamePlaying,
            },{
                id: 'new_player',
                label: 'New Player',
                action: this._newPlayer.bind(this),
                disabled: !noPlayerSelected
            }, {
                id: "show_player",
                label: "Show Player",
                action: this._showPlayer.bind(this),
            },{
                id: 'join_game',
                label: 'Join Game',
                action: this._joinGame.bind(this),
                disabled: noGameSelected || noPlayerSelected || gamePlaying || !noPlayerJoined,
            }, {
                id: 'start_game',
                label: 'Start Game',
                action: this._startGame.bind(this),
                disabled: noGameSelected || noPlayerSelected || gamePlaying || noPlayerJoined,
            }, {
                id: 'choose_hit',
                label: 'Hit',
                action: this._chooseHit.bind(this),
                disabled: noGameSelected || noPlayerSelected || noPlayerJoined || !gamePlaying,
            }, {
                id: 'choose_stand',
                label: 'Stand',
                action: this._chooseStand.bind(this),
                disabled: noGameSelected || noPlayerSelected || noPlayerJoined || !gamePlaying,
            }, {
                id: 'show_hands',
                label: 'Show Hands',
                action: this._showHands.bind(this),
                disabled: noGameSelected || noPlayerSelected || noPlayerJoined || !gamePlaying,
            },
        ]


        let name: string | undefined;
        if (this.state.player) {
            name = `${this.state.player.name} (${this.state.player.address})`;
        } else {
            name = this.state.selectedAddress;
        }

        const gameIdSelected = this.state.gameIdSelected;

        // If everything is loaded, we render the application.
        return (
            <div className="container p-4">
                <div className="row">
                    <div className="col-12">
                        <h1>
                            Blackjack
                        </h1>
                        <p>
                            Welcome <b>{name}</b>.
                        </p>
                        {gameIdSelected !== null && <><p>
                            Game: <b>{gameIdSelected}</b>.
                        </p>
                        <p>
                            Joined: <b>{this.state.gameJoined ? 'Yes' : 'No'}</b>.
                        </p>
                        <p>
                            Playing: <b>{this.state.gamePlaying ? 'Yes' : 'No'}</b>.
                        </p>
                        </>}
                        <nav className="flex gap-2 mt-5 flex-row justify-between items-center flex-wrap border-b-2 border-gray-400">{
                            actions.map(({ id, label, action, disabled }) => {
                                return (
                                    <span key={id} className="flex-initial">
                                        <button
                                            className="p-2 rounded cursor-pointer bg-red-600 hover:bg-red-800 transition disabled:opacity-50 disabled:hover:bg-red-600 disabled:cursor-not-allowed"
                                            onClick={() => { action() }}
                                            disabled={disabled}
                                            type="button"
                                        >
                                            {label}
                                        </button>
                                    </span>
                                )
                            })
                        }
                        </nav>
                        {this.state.games && <section className="games">
                            {this.state.gameIdSelected === null && <h2>Select one game</h2>}
                            <div className="mt-2 flex flex-row gap-2 flex-wrap">
                                {this.state.games.map(({ id, players }) => (
                                    <button className={`p-2 rounded cursor-pointer transition disabled:cursor-not-allowed disabled:bg-slate-400 ${players === 0 ? "bg-indigo-600 hover:bg-indigo-800" : "bg-orange-600 hover:bg-orange-800"
                                        }`} disabled={id === this.state.gameIdSelected} onClick={() => {
                                            this._selectGame(id)
                                        }} key={id}>Game: {id}<hr />{players} Players</button>
                                ))}
                            </div>
                        </section>}
                    </div>
                </div>

                <div className="row">
                    <div className="col-12">
                        <code>
                            {JSON.stringify(this.state.hands || {})}
                            </code>
                    </div>
                    <div className="col-12">
                        {this.state.hands?.players?.map(playerHand => {
                            return (
                                <div key={playerHand.name} style={{ position: 'relative', height: '200px' }}>
                                    {playerHand.name} - {playerHand.points}
                                    {playerHand.hand?.map((card: SuitType, i: number) => {
                                        return (
                                            <div key={`${i}-${card}`} style={{ position: 'absolute', rotate: `${i * 12}deg`, translate: `${i * 12}px ${10 + i * 3}px` }}>
                                                <Card name={card} />
                                            </div>
                                        )
                                    })}
                                </div>
                            );
                        })}
                    </div>
                </div>
            </div>
        );
    }

    private _selectGame(gameIdSelected: string) {
        if (this.state.gameIdSelected !== null) {
            let response = globalThis.confirm('You are already in a game. Leave it?')

            if (!response) {
                return;
            }
        }

        this.setState({ gameIdSelected })
    }

    private _showGames() {
        console.log('show games...')
        this.setState({ isLoading: true })
        this._readGames().finally(() => {
            this.setState({ isLoading: false })
        });
    }

    private checkGameIdSelected(gameIdSelected: typeof this.state.gameIdSelected): asserts gameIdSelected is string {
        if (typeof gameIdSelected !== "string") {
            throw new Error('No game is selected')
        }
    }
    private checkSigner(signer: typeof this._signer): asserts signer is Signer {
        if (!signer) {
            throw new Error('Signer not initialized')
        }
    }

    private checkProvider(provider: typeof this._provider): asserts provider is BrowserProvider {
        if (!provider) {
            throw new Error('Provider not initialized')
        }
    }
    private async _chooseStand() {
        const game_id = this.state.gameIdSelected;
        this.checkGameIdSelected(game_id);
        this.checkSigner(this._signer);
        this.checkProvider(this._provider);
        await Cartesi.sendInput({ action: "stand", game_id }, this._signer, this._provider)
    }

    private async _chooseHit() {
        const table_id = this.state.gameIdSelected;
        this.checkGameIdSelected(table_id);
        this.checkSigner(this._signer);
        this.checkProvider(this._provider);
        await Cartesi.sendInput({ action: "hit", table_id }, this._signer, this._provider)
    }

    private async _startGame() {
        const game_id = this.state.gameIdSelected;
        this.checkGameIdSelected(game_id);
        this.checkSigner(this._signer);
        this.checkProvider(this._provider);
        await Cartesi.sendInput({ action: "start_game", game_id }, this._signer, this._provider)
    }

    componentDidMount(): void {
        if (this.ONCE) {
            this.ONCE = false;
            this._attachNetworkChanges();
        }
    }

    componentWillUnmount() {
        // We poll the user's balance, so we have to stop doing that when Dapp
        // gets unmounted
        this._stopPollingData();
    }

    /**
     * This method checks if the selected network is Localhost:8545
     */
    private async _handleChainChanged(chainId: string) {
        console.log("Change chain triggered", { chainId })

        /**
         * Convert chainId from hex to decimal and then to string.
         * @see https://docs.metamask.io/wallet/how-to/connect/detect-network/#chain-ids
         */
        if (chainId !== HARDHAT_NETWORK_HEX) {
            this._switchChain();
        }
    }

    async _attachNetworkChanges() {
        window.ethereum?.on("chainChanged", (chainId) => this._handleChainChanged(chainId))
    }

    async _connectWallet() {
        // This method is run when the user clicks the Connect. It connects the
        // dapp to the user's wallet, and initializes it.

        // To connect to the user's wallet, we have to run this method.
        // It returns a promise that will resolve to the user's address.
        const eth = window.ethereum;
        if (!eth) {
            console.error('No ethereum provider')
            return;
        }
        const [selectedAddress] = await eth.request({ method: 'eth_requestAccounts' });

        // Once we have the address, we can initialize the application.

        /**
         * First we check the network
         * We will check network when event is triggered like this:
         * @see https://docs.metamask.io/wallet/reference/provider-api/#chainchanged
         */
        // this._checkNetwork();

        this._initialize(selectedAddress);

        // We reinitialize it whenever the user changes their account.
        eth.on("accountsChanged", ([newAddress]) => {
            this._stopPollingData();
            /**
             * `accountsChanged` event can be triggered with an undefined newAddress.
             * This happens when the user removes the Dapp from the "Connected
             * list of sites allowed access to your addresses" (Metamask > Settings > Connections)
             * To avoid errors, we reset the dapp state
             * @see https://docs.metamask.io/wallet/reference/provider-api/#accountschanged
             */

            if (typeof newAddress !== "string") {
                return this._resetState();
            }

            this._initialize(newAddress);
        });
    }

    async _initialize(userAddress: string) {
        // This method initializes the dapp

        // We first store the user's address in the component's state
        this.setState({
            selectedAddress: userAddress,
            isLoading: true,
        });

        // Then, we initialize ethers, fetch the token's data, and start polling
        // for the user's balance.

        // Fetching the token data and the user's balance are specific to this
        // sample project, but you can reuse the same initialization pattern.
        this._initializeEthers();

        await Promise.all([
            this._readGames(),
            this._loadUserData(userAddress),
        ]);
        // this._showHands().catch(console.error);

        this.setState({ isLoading: false })

        // this._stopPollingData();
        // this._startPollingData();
    }
    private async _loadUserData(userAddress: string) {
        console.log('read player...')

        try {
            const player = await Cartesi.inspectWithJson<NonNullable<DappState['player']>>({ "action": "show_player", "address": userAddress })
            console.log({ player })

            if (player) {
                this.setState({ player })

                const playerIsPlaying = player.playing.length > 0;
                const gameIdSelected = player.playing.at(0) || player.joined.at(0);

                if (gameIdSelected) {
                    this.setState({
                        gameIdSelected,
                        gameJoined: true,
                        gamePlaying: playerIsPlaying,
                    })
                }
            }
        } catch (e) {
            console.log(e);
        }
    }

    async _initializeEthers() {
        // We first initialize ethers by creating a provider using window.ethereum
        // (window as any)._ethers = ethers;
        const eth = window.ethereum;
        if (!eth) {
            throw new Error('No ethereum provider')
        }
        this._provider = new BrowserProvider(eth);

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
        this.checkSigner(this._signer);
        this.checkProvider(this._provider);
        console.log('new player')
        const player = globalThis.prompt('Player name');
        if (!player) {
            return;
        }
        this.setState({ isLoading: true })
        await Cartesi.sendInput({
            action: 'new_player',
            name: player,
        }, this._signer, this._provider)
        if (!this.state.selectedAddress) {
            console.error('No selected address')
            return;
        }
        await this._loadUserData(this.state.selectedAddress);
        this.setState({ isLoading: false })
    }

    async _showPlayer() {
        const address = this.state.selectedAddress;
        if (!address) {
            console.error('No selected address')
            return;
        }
        this.setState({ isLoading: true });
        await this._loadUserData(address);
        this.setState({ isLoading: false });
    }

    async _joinGame() {
        const game_id = this.state.gameIdSelected;
        this.checkGameIdSelected(game_id);
        this.checkSigner(this._signer);
        this.checkProvider(this._provider);
        this.setState({ isLoading: true });
        await Cartesi.sendInput({
            action: 'join_game',
            game_id
        }, this._signer, this._provider)

        const address = this.state.selectedAddress;
        if (!address) {
            console.error('No selected address')
            this.setState({ isLoading: false })
            return;
        }
        /**
         * Maybe input still in process
         */
        await Promise.all([this._loadUserData(address),
            , this._readGames()]);
        this.setState({ gameJoined: true, isLoading: false });
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
            if (this.state.gamePlaying) {
                // We run it once immediately so we don't have to wait for it
                await this._showHands();
            }
        } catch (e) {
            console.error(e);
        }
        this._pollDataInterval = setTimeout(() => this._startPollingData(), Dapp.POLL_TIME_MS);
    }

    _stopPollingData() {
        clearInterval(this._pollDataInterval);
        this._pollDataInterval = undefined;
    }

    async _readGames() {
        console.log('read game...')

        try {
            const response = await Cartesi.inspectWithJson({ action: 'show_games' })

            if (response && "games" in response && Array.isArray(response.games)) {
                this.setState({ games: response.games })
            }
        } catch (e) {
            console.error(e);
        }
    }

    async _showHands() {
        const table_id = this.state.gameIdSelected;
        this.checkGameIdSelected(table_id);
        console.log('show hands...')
        const hands = await Cartesi.inspectWithJson<ReponseHands>({ action: 'show_hands', table_id })
        // const hands = JSON.parse(`{"game_id":"1","players":[{"hand":["3-Hearts","A-Spades","2-Spades","K-Spades"],"name":"Alice","points":14},{"hand":["A-Hearts","3-Spades"],"name":"Oshiro","points":14}],"table_id":"31cd40cd-0350-4d05-9dd3-592e30f7382d"}`)
        if (hands) {
            this.setState({ hands })
        }

        /**
         * reset state when game is finished
         */
        console.log({ hands })
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
    _getRpcErrorMessage(error: ErrorRpc) {
        if ("data" in error) {
            return error.data.message;
        }

        return error.message;
    }

    // This method resets the state
    _resetState() {
        this.setState(this.initialState);
    }

    async _switchChain() {
        const chainIdHex = HARDHAT_NETWORK_HEX;
        await window.ethereum?.request({
            method: "wallet_switchEthereumChain",
            params: [{ chainId: chainIdHex }],
        });
        const selectedAddress = this.state.selectedAddress;
        if (!selectedAddress) {
            console.error('No selected address')
            return;
        }
        this._initialize(selectedAddress);
    }

    /**
     * @deprecated
     * This method checks if the selected network is Localhost:8545
     * @see https://docs.metamask.io/wallet/how-to/connect/detect-network/#chain-ids
     **/
    private _checkNetwork() {
        if ((window as any).ethereum.networkVersion !== HARDHAT_NETWORK_ID) {
            this._switchChain();
        }
    }
}
