// Copyright 2022 Cartesi Pte. Ltd.

import type { Abi } from "viem";

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

export type Contract = {
    address: string;
    abi: Abi;
};

export type Deployment = {
    name: string;
    chainId: string;
    contracts: Record<string, Contract>;
};

export const checkIfIsDeployment = (obj: unknown): obj is Deployment => {
    if (typeof obj !== "object" || obj === null) {
        return false;
    }
    const deployment = obj as Deployment;
    return (
        typeof deployment.name === "string" &&
        typeof deployment.chainId === "string" &&
        typeof deployment.contracts === "object" &&
        deployment.contracts !== null
    );
}