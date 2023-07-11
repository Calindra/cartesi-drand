
export interface DrandConfig {
    chainHash: string
    publicKey: string
    secondsToWait: number
}

export interface InputSenderConfig {
    dappAddress: string
    mnemonic: string
    rpc: string
    accountIndex: number
}

export interface CartesiConfig {
    inspectEndpoint: string
}
