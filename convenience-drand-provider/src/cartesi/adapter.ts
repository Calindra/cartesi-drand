import type { Provider as ProviderOut } from "@ethersproject/providers";
import type { Provider as ProviderIn, Signer } from "ethers";

export function getProvider(
  providerIn: ProviderIn | Signer,
): ProviderOut | Signer {
  return providerIn as unknown as ProviderOut;
}
