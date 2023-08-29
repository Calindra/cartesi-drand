import jack from "../../assets/jack.gif";
import { CardProp } from "./CardProp";
import { Suit, getColorClassName } from "./Suit";

export function CardJ({suit}: CardProp) {
  return (<div className="card">
  <div className={`front ${getColorClassName(suit)}`}>
    <div className="index">J<br /><Suit name={suit} /></div>
    <img className="face" src={jack} alt="" width="80" height="130" />
    <div className="spotA1"><Suit name={suit} /></div>
    <div className="spotC5"><Suit name={suit} /></div>
  </div>
</div>
)
}