// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import type { Chain as TypedChain } from "viem";
import {
  anvil,
  goerli,
  bscTestnet,
  avalancheFuji,
  polygonMumbai,
  optimismGoerli,
  arbitrumGoerli,
  gnosisChiado,
  sepolia,
} from "viem/chains";

export interface Chain {
  name: string;
  chain: TypedChain;
}

// compatible networks
export const networks = new Map<number, Chain>([
  [31337, { name: "localhost", chain: anvil }],
  [5, { name: "goerli", chain: goerli }],
  [97, { name: "bsc_testnet", chain: bscTestnet }],
  [43113, { name: "avax_fuji", chain: avalancheFuji }],
  [80001, { name: "polygon_mumbai", chain: polygonMumbai }],
  [420, { name: "optimism_goerli", chain: optimismGoerli }],
  [421613, { name: "arbitrum_goerli", chain: arbitrumGoerli }],
  [10200, { name: "chiado", chain: gnosisChiado }],
  [11155111, { name: "sepolia", chain: sepolia }],
]);