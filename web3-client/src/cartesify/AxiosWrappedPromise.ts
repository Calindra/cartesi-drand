export class AxiosWrappedPromise {
    promise: Promise<AxiosResponse>
    reject?: (reason?: any) => void;
    resolve?: (value: AxiosResponse | PromiseLike<AxiosResponse>) => void;
    constructor() {
        this.promise = new Promise((resolve, reject) => {
            this.resolve = resolve;
            this.reject = reject;
        })
    }
}

interface AxiosResponse {
    data: any
    headers: Record<string, string>
    status: number
}
