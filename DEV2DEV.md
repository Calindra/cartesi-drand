Install RISC-V

```sh
docker run --rm --privileged tonistiigi/binfmt --install riscv64
```

Build dapp and middleware:

```sh
npx cartesi build
```

Run dapp, middleware and others:

```sh
npx cartesi start
npx cartesi rollups logs -f
```