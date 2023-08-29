import { CardProp } from "./CardProp";
import { Suit, getColorClassName } from "./Suit";

export function CardA({ suit }: CardProp) {
    return (<div className="card">
        <div className={`front ${getColorClassName(suit)}`}>
            <div className="index">A<br /><Suit name={suit} /></div>
            <div className="ace"><Suit name={suit} /></div>
        </div>
    </div>)
}