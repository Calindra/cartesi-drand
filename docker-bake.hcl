group "default" {
    description = "Default group"
    targets = ["dapp"]
}

target "dapp" {
    description = "Build image"
    dockerfile = "Dockerfile"
    tags = ["blackjack:latest"]
    args = {
        FOLDER_MIDDLEWARE = "convenience-middleware"
        MACHINE_EMULATOR_TOOLS_VERSION = "0.17.0"
    }
}