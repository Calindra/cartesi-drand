import king from "../../assets/king.gif";
import { CardProp } from "./CardProp";
import { Suit, getColorClassName } from "./Suit";

export function CardK({ suit }: CardProp) {
  return (<div className="card">
    <div className={`front ${getColorClassName(suit)}`}>
      <div className="index">K<br /><Suit name={suit} /></div>
      <img className="face" src={king} alt="" width="80" height="130" />
      <div className="spotA1"><Suit name={suit} /></div>
      <div className="spotC5"><Suit name={suit} /></div>
    </div>
  </div>)
}