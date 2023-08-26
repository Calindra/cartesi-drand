import { CardProp } from "./CardProp";
import { Suit, getColorClassName } from "./Suit";

export function Card9({ suit }: CardProp) {
    return (<div className="card">
        <div className={`front ${getColorClassName(suit)}`}>
            <div className="index">9<br /><Suit name={suit} /></div>
            <div className="spotA1"><Suit name={suit} /></div>
            <div className="spotA2"><Suit name={suit} /></div>
            <div className="spotA4"><Suit name={suit} /></div>
            <div className="spotA5"><Suit name={suit} /></div>
            <div className="spotB3"><Suit name={suit} /></div>
            <div className="spotC1"><Suit name={suit} /></div>
            <div className="spotC2"><Suit name={suit} /></div>
            <div className="spotC4"><Suit name={suit} /></div>
            <div className="spotC5"><Suit name={suit} /></div>
        </div>
    </div>)
}