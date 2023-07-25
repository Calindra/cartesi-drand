# Blackjack

Example of two players:
```mermaid
stateDiagram-v2
    [*] --> Initial
    Initial --> ShuffleDeck : Deal
    ShuffleDeck --> PlayerLoop  : Game start

    state PlayerLoop {
        state WhoIsAvailable <<choice>>
        state PhaseInitial <<fork>>
        [*] --> ForkAvailable
        ForkAvailable --> PhaseInitial

        PhaseInitial --> Player1
        Player1 --> Player1Hit : Hit
        Player1 --> Player1Stand : Stand


        PhaseInitial --> Player2
        Player2 --> Player2Hit : Hit
        Player2 --> Player2Stand : Stand


        %% PhaseInitial --> Player3
        %% Player3 --> Player3Hit : Hit
        %% Player3 --> Player3Stand : Stand

        state PhaseWait <<join>>

        Player1Hit --> PhaseWait
        Player2Hit --> PhaseWait
        %% Player3Hit --> PhaseWait

        Player1Stand --> PhaseWait
        Player2Stand --> PhaseWait
        %% Player3Stand --> PhaseWait

        PhaseWait --> WhoIsAvailable
        WhoIsAvailable --> ForkAvailable: Some can continue
        WhoIsAvailable --> [*]: Nobody can continue
    }

    PlayerLoop --> CompareHands

    CompareHands --> PlayerWins : Some Player Wins
    CompareHands --> Draw : Draw
    PlayerWins --> PlayAgain
    Draw --> PlayAgain
    PlayAgain --> ShuffleDeck : Deal
    PlayAgain --> [*] : Finish
```

```mermaid
sequenceDiagram
    participant DApp as DApp
    participant Middleware as Middleware
    participant Rollups as Rollups

    DApp->>Middleware: Chama /finish com input
    Middleware->>Rollups: Chama /finish com input
    Rollups->>Middleware: Retorna resultado do processamento
    Middleware->>DApp: Retorna resultado do processamento
    DApp->>Middleware: Chama /random
    Middleware->>Rollups: Chama /finish para congelar a Cartesi Machine
    Note over Middleware: Aguarda resposta do /finish pelo /random
    Rollups->>Middleware: Retorna resultado do /finish
    alt Resultado é um beacon
        Middleware->>DApp: Devolve o beacon para o DApp
    else Resultado não é um beacon
        Middleware->>DApp: Retorna erro 404 para o DApp
        DApp->>Middleware: Chama /finish novamente
        Middleware->>Rollups: Chama /finish novamente para obter o input
        Rollups->>Middleware: Retorna resultado do /finish novamente
        Note over Middleware: Processamento adicional do input
        Middleware->>DApp: Retorna resultado do /finish novamente
    end
```