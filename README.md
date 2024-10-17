# Pin1Yin1-rustc

Pin1Yin1Yu3Yan2's compiler written in rust.

## Backends

There are two code generation backends: `llvm` and `c`. By default, both are enabled. You can disable all of them by using the `--no-default-features` flag.

e.g. enable c backend only
```shell
cargo build --release --no-default-features --features=backend-c
```

### LLVM Backend

The version of llvm is `llvm@18`, by default, it's preferred to dynamic link to the system's llvm library.

If you want to use a different version of llvm, or you want to statically link to the llvm library, you can order the feature flags of `inkwell`

```shell
cargo build --release --no-default-features --features=backend-llvm,inkwell/llvm18-0-force-static
```

Enable the feature flag `backend-llvm` to not enough to compile, you must also enable one of the `inkwell` feature flags.

# C Backend

C backend is just a code printer, it doesn't need any extra dependencies.

## Feature flags

- `default`: Enable all features follow, and `inkwell/llvm18-0-prefer-dynamic`.
- `backend-llvm`: Enable llvm backend.
- `backend-c`: Enable c backend.
- `parallel-declare`: Enable parallel type declaration of items(like functions).
- 
## Note

* In archlinux, llvm is installed in `/opt/llvm` and only dynamic library is provided. If you want to compile and use llvm backend, you must set the `LD_LIBRARY_PATH` to `/opt/llvm/lib`.