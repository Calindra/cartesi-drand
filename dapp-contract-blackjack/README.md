# Blackjack

Example of two players:
```mermaid
stateDiagram-v2
    %% state CompareHits <<choice>>
    %% state DealerHit <<choice>>

    [*] --> Initial
    Initial --> ShuffleDeck : Deal
    ShuffleDeck --> PlayerTurn  : Player's Turn
    PlayerTurn --> PlayerHit : Hit
    PlayerTurn --> PlayerStand : Stand
    PlayerHit --> DealerTurn
    PlayerStand --> DealerTurn
    DealerTurn --> DealerHit : Hit
    DealerHit --> CompareHands: If player stand
    DealerHit --> CompareHits: Both hit
    CompareHits --> PlayerTurn: Both can continue
    CompareHits --> CompareHands: Some cant continue
    DealerTurn --> CompareHands : Stand
    CompareHands --> PlayerWins : Player Wins
    CompareHands --> DealerWins : Dealer Wins
    CompareHands --> Draw : Draw
    PlayerWins --> PlayAgain
    DealerWins --> PlayAgain
    Draw --> PlayAgain
    PlayAgain --> ShuffleDeck : Deal
    PlayAgain --> [*] : Finish
```
