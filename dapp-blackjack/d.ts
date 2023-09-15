declare module "*.gif" {
  const value: any;
  export = value;
}

interface Window {
  /** @link {https://docs.metamask.io/wallet/reference/provider-api/} */
  ethereum?: // import("@ethersproject/providers").ExternalProvider &
  import("ethers").Eip1193Provider & import("ethers").AbstractProvider;
}
