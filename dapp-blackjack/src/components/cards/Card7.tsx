import { CardProp } from "./CardProp";
import { Suit, getColorClassName } from "./Suit";

export function Card7({ suit }: CardProp) {
    return (<div className="card">
        <div className={`front ${getColorClassName(suit)}`}>
            <div className="index">7<br /><Suit name={suit} /></div>
            <div className="spotA1"><Suit name={suit} /></div>
            <div className="spotA3"><Suit name={suit} /></div>
            <div className="spotA5"><Suit name={suit} /></div>
            <div className="spotB2"><Suit name={suit} /></div>
            <div className="spotC1"><Suit name={suit} /></div>
            <div className="spotC3"><Suit name={suit} /></div>
            <div className="spotC5"><Suit name={suit} /></div>
        </div>
    </div>
    )
}