export type SuitType = 'hearts' | 'spades' | 'clubs' | 'diamonds'

export function Suit({ name }: { name: SuitType }) {
    if (name === 'hearts') return <>&hearts;</>
    if (name === 'spades') return <>&spades;</>
    if (name === 'clubs') return <>&clubs;</>
    if (name === 'diamonds') return <>&diams;</>
    return "?"
}

export function getColorClassName(name: SuitType) {
    if (name === 'hearts' || name === 'diamonds') return "red"
    if (name === 'spades' || name === 'clubs') return "black"
    return ""
}
