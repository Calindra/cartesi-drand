import type { CardProp } from "./CardProp";
import { Suit, getColorClassName } from "./Suit";
import jack from "../../assets/jack.gif";
import king from "../../assets/king.gif";
import queen from "../../assets/queen.gif";

type Letter = "A" | "B" | "C";
type PosNum = 1 | 2 | 3 | 4 | 5;
type RankJQK = "J" | "Q" | "K";
type Position = `${Letter}${PosNum}` | "ace" | `face`;
export type Rank = "A" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | RankJQK;

interface CardWrapper extends CardProp {
  rank: Rank;
  isFaceDown?: boolean;
}

const glyphsMap: Readonly<Record<Rank, Position[]>> = {
  A: ["ace"],
  "2": ["B1", "B5"],
  "3": ["B1", "B3", "B5"],
  "4": ["A1", "A5", "C1", "C5"],
  "5": ["A1", "A5", "B3", "C1", "C5"],
  "6": ["A1", "A3", "A5", "C1", "C3", "C5"],
  "7": ["A1", "A3", "A5", "B2", "C1", "C3", "C5"],
  "8": ["A1", "A3", "A5", "B2", "B4", "C1", "C3", "C5"],
  "9": ["A1", "A2", "A4", "A5", "B3", "C1", "C2", "C4", "C5"],
  "10": ["A1", "A2", "A4", "A5", "B2", "B4", "C1", "C2", "C4", "C5"],
  J: ["face", "A1", "C5"],
  Q: ["face", "A1", "C5"],
  K: ["face", "A1", "C5"],
};

const faceImages: Readonly<Record<RankJQK, string>> = {
  J: jack,
  Q: queen,
  K: king,
};

export function CardM({ suit, rank, isFaceDown = false }: CardWrapper): JSX.Element {
  const glyphs = glyphsMap[rank];

  return (
    <div className="card">
      {!isFaceDown && (
        <div className={`front ${getColorClassName(suit)}`}>
          <div className="index">
            {rank}
            <br />
            <Suit name={suit} />
          </div>
          {glyphs.map((glyph) => {
            if (glyph === "face") {
              return <img key={glyph} className={glyph} src={faceImages[rank]} alt="" width="80" height="130" />;
            }
            return (
              <div key={glyph} className={glyph === "ace" ? glyph : `spot${glyph}`}>
                <Suit name={suit} />
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
