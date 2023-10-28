import { PlayerHand } from "../models/PlayerHand";
import { Scoreboard } from "../models/Scoreboard";
import { Card } from "./cards/Card";
import { SuitType } from "./cards/Suit";

interface Props {
    currentPlayerName?: string
    hands: { players: PlayerHand[] },
    scoreboard?: Scoreboard,
    hit: () => void,
    stand: () => void,
}

export function GamePlay({ hands, scoreboard, currentPlayerName, hit, stand }: Props) {
    return (
        <div style={{ marginTop: '20px' }}>
            {scoreboard && (
                <div style={{ marginBottom: '10px', fontSize: '19px' }}>Winner: {scoreboard.winner}</div>
            )}
            <div style={{ display: 'flex' }}>
                {hands?.players?.map(playerHand => {
                    return (
                        <div>
                            <div key={playerHand.name} style={{ position: 'relative', height: '200px', width: '179px' }}>
                                {playerHand.name} - {playerHand.points}
                                {playerHand.hand?.map((card: SuitType, i: number) => {
                                    return (
                                        <div key={`${i}-${card}`} style={{ position: 'absolute', rotate: `${(i - 1) * 12}deg`, translate: `${i * 12}px ${10 + i * 3}px` }}>
                                            <Card name={card} />
                                        </div>
                                    )
                                })}

                            </div>
                            {currentPlayerName === playerHand.name && (
                                <div>
                                    <button
                                        className="p-2 rounded cursor-pointer bg-red-600 hover:bg-red-800 transition disabled:opacity-50 disabled:hover:bg-red-600 disabled:cursor-not-allowed"
                                        onClick={hit}
                                        type="button"
                                        disabled={playerHand.is_standing}
                                        style={{ marginRight: '9px' }}
                                    >
                                        Hit
                                    </button>
                                    <button
                                        className="p-2 rounded cursor-pointer bg-red-600 hover:bg-red-800 transition disabled:opacity-50 disabled:hover:bg-red-600 disabled:cursor-not-allowed"
                                        onClick={stand}
                                        disabled={playerHand.is_standing}
                                        type="button"
                                    >
                                        Stand
                                    </button>
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
        </div>
    )
}