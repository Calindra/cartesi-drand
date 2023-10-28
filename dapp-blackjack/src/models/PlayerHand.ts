import { SuitType } from "../components/cards/Suit";


export interface PlayerHand {
    name: string,
    points: number,
    hand: SuitType[],
    is_busted: boolean,
    is_standing: boolean,
}