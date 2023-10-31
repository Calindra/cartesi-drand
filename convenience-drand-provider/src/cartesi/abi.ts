// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// export type Contract<T = unknown> = {
//     address: string;
//     abi: T; // XXX: type it more? or any an existing package, like 'abitype'
// };

export class Contract<T = unknown> {
  constructor(public address: string, public abi: T) {}
  static fromObj(obj: Record<string, unknown>): Contract {
    const address = obj.address;

    if (typeof address !== "string") {
      throw TypeError(`address is not a string: ${address}`);
    }

    return new Contract(address, obj.abi);
  }
}

export class Deployment {
  constructor(
    public name: string,
    public chainId: string,
    public contracts: Record<string, Contract>
  ) { }

    static fromObj(obj: Record<string, unknown>): Deployment {
        const name = obj.name;
        const chainId = obj.chainId;
        const contracts = obj.contracts;

        if (typeof name !== "string") {
            throw TypeError(`name is not a string: ${name}`);
        }

        if (typeof chainId !== "string") {
            throw TypeError(`chainId is not a string: ${chainId}`);
        }

        if (typeof contracts !== "object" || contracts === null) {
            throw TypeError(`contracts is not an object valid: ${contracts}`);
        }

        return new Deployment(name, chainId, contracts as Record<string, Contract>);
    }
}
