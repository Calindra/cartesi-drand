import { CardProp } from "./CardProp";
import { Suit, getColorClassName } from "./Suit";

export function Card2({ suit }: CardProp) {
    return (<div className="card">
    <div className={`front ${getColorClassName(suit)}`}>
      <div className="index">2<br /><Suit name={suit} /></div>
      <div className="spotB1"><Suit name={suit} /></div>
      <div className="spotB5"><Suit name={suit} /></div>
    </div>
  </div>)
}