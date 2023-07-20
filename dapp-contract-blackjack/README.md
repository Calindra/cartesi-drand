# Blackjack

Example of two players:
```mermaid
stateDiagram-v2
    state CompareHits <<choice>>
    state DealerHit <<choice>>

    [*] --> Initial
    Initial --> ShuffleDeck : Deal
    ShuffleDeck --> PlayerLoop  : Player's Turn

    state PlayerLoop {
        [*] --> PlayerTurn
        PlayerTurn --> PlayerHit : Hit
        PlayerTurn --> PlayerStand : Stand
        PlayerHit --> DealerTurn
        PlayerStand --> DealerTurn
        DealerTurn --> DealerHit : Hit
        DealerHit --> CompareHits: Both hit
        CompareHits --> PlayerTurn: Both can continue

        DealerHit --> [*]: If player stand
        CompareHits --> [*]: Some cant continue
        DealerTurn --> [*] : Stand
    }

    PlayerLoop --> CompareHands

    CompareHands --> PlayerWins : Player Wins
    CompareHands --> DealerWins : Dealer Wins
    CompareHands --> Draw : Draw
    PlayerWins --> PlayAgain
    DealerWins --> PlayAgain
    Draw --> PlayAgain
    PlayAgain --> ShuffleDeck : Deal
    PlayAgain --> [*] : Finish
```
