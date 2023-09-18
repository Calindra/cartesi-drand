import { ReactNode } from "react";

import { NetworkErrorMessage } from "./NetworkErrorMessage";

interface ConnectWalletProps {
  connectWallet?: () => void
  networkError?: string
  dismiss?: () => void
}

export function ConnectWallet({ connectWallet, networkError, dismiss }: ConnectWalletProps): ReactNode {
  return (
    <div className="container">
      <div className="row justify-content-md-center">
        <div className="col-12 text-center">
          {/* Wallet network should be set to Localhost:8545. */}
          {networkError && (
            <NetworkErrorMessage
              message={networkError}
              dismiss={dismiss} />
          )}
        </div>
        <div className="col-6 p-4 text-center">
          <p>Please connect to your wallet.</p>
          <button
            className="btn btn-warning h-10 px-5 m-2  transition-colors duration-150 bg-yellow-400 rounded-lg focus:shadow-outline hover:bg-yellow-800"
            type="button"
            onClick={connectWallet}
          >
            Connect Wallet
          </button>
        </div>
      </div>
    </div>
  );
}