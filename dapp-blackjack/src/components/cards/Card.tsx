import { CardM, Rank } from "./CardM"
import type { SuitType } from "./Suit"

export function Card({ name }: { name: string }) {
    const [value, suit] = name.split('-')
    const lowercaseSuit = suit.toLowerCase();

    const assertSuit = (suit: string): suit is SuitType => {
        return ['spades', 'hearts', 'clubs', 'diamonds'].includes(suit)
    }

    const assertRank = (rank: string): rank is Rank => {
        return ['A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', "Q", "K"].includes(rank)
    }

    if (!assertSuit(lowercaseSuit)) {
        console.error(`Invalid suit: ${suit}`)
        return null
    }

    if (!assertRank(value)) {
        console.error(`Invalid rank: ${value}`)
        return null
    }

    return <CardM rank={value} suit={lowercaseSuit} />
}