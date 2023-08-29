import { Card10 } from "./Card10"
import { Card2 } from "./Card2"
import { Card3 } from "./Card3"
import { Card4 } from "./Card4"
import { Card5 } from "./Card5"
import { Card6 } from "./Card6"
import { Card7 } from "./Card7"
import { Card8 } from "./Card8"
import { Card9 } from "./Card9"
import { CardA } from "./CardA"
import { CardJ } from "./CardJ"
import { CardK } from "./CardK"
import { CardQ } from "./CardQ"

export function Card({ name }: { name: string }) {
    const [value, suit] = name.split('-')
    if (value === 'A') return <CardA suit={suit.toLowerCase() as any} />
    if (value === '2') return <Card2 suit={suit.toLowerCase() as any} />
    if (value === '3') return <Card3 suit={suit.toLowerCase() as any} />
    if (value === '4') return <Card4 suit={suit.toLowerCase() as any} />
    if (value === '5') return <Card5 suit={suit.toLowerCase() as any} />
    if (value === '6') return <Card6 suit={suit.toLowerCase() as any} />
    if (value === '7') return <Card7 suit={suit.toLowerCase() as any} />
    if (value === '8') return <Card8 suit={suit.toLowerCase() as any} />
    if (value === '9') return <Card9 suit={suit.toLowerCase() as any} />
    if (value === '10') return <Card10 suit={suit.toLowerCase() as any} />
    if (value === 'J') return <CardJ suit={suit.toLowerCase() as any} />
    if (value === 'Q') return <CardQ suit={suit.toLowerCase() as any} />
    if (value === 'K') return <CardK suit={suit.toLowerCase() as any} />
}