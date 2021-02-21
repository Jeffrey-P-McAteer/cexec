
# CExec

CExec is a common execution environment, designed to let users submit WASM applications
(and application sub-components) to remote computers - servers, co-worker PCs,
or a desktop in another room from a cell phone.

The architecture is likely going to be quite volatile for until we reach a `v1.0` status,
so the best place to read about how it works is the Architecture Decision Record directory:
[./adr](./adr).


# Usage

TODO

# Free public servers

TODO

# Testing

```bash
# Run all builds and tests
python -m hygiene
```

# TODOs

 - [ ] Implement limited [WASI](https://github.com/webassembly/wasi) API for client programs
 - [ ] Flush out configuration to let users extend the API with one line + shell script OR `.dll`/`.so` + name of function (using C ABI) OR `.wasm` + name of function (using WASM ABI).
 - [ ] misc. infrastructure efforts like N+1 forwarding for servers, IP rate limiting, etc.






