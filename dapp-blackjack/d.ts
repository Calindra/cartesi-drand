declare module "*.gif" {
  const value: any;
  export = value;
}

type EventType = "accountsChanged" | "chainChanged" | "connect" | "disconnect" | "message";

type EventTuple =
  | [event: "accountsChanged", handler: (accounts: string[]) => void]
  | [event: "chainChanged", handler: (chainId: string) => void]
  | [event: "connect", handler: (info: { chainId: string }) => void]
  | [event: "disconnect", handler: (error: { code: number; message: string }) => void]
  | [event: "message", handler: (message: any, event: MessageEvent) => void];

interface Window {
  /** @link {https://docs.metamask.io/wallet/reference/provider-api/} */
  ethereum?: import("@ethersproject/providers").ExternalProvider & {
    on: (...args: EventTuple) => void;
  };
}
