import { CardProp } from "./CardProp";
import { Suit, getColorClassName } from "./Suit";

export function Card6({ suit }: CardProp) {
    return (<div className="card">
        <div className={`front ${getColorClassName(suit)}`}>
            <div className="index">6<br /><Suit name={suit} /></div>
            <div className="spotA1"><Suit name={suit} /></div>
            <div className="spotA3"><Suit name={suit} /></div>
            <div className="spotA5"><Suit name={suit} /></div>
            <div className="spotC1"><Suit name={suit} /></div>
            <div className="spotC3"><Suit name={suit} /></div>
            <div className="spotC5"><Suit name={suit} /></div>
        </div>
    </div>)
}