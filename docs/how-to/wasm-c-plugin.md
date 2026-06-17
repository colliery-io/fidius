<!--
Copyright 2026 Colliery, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

# A WASM Plugin in C (wasi-sdk)

The **same `greeter` interface** as the [Rust](../tutorials/your-first-wasm-plugin.md),
[Python](wasm-python-plugin.md), [JavaScript](wasm-javascript-plugin.md), and
[Go](wasm-go-plugin.md) guests, now in **C**. This is the leanest of all: with no
runtime to embed, the component is **~18 KB**.

The worked example is the committed fixture `tests/wasm-fixtures/greeter-c/`,
verified by `polyglot_c_guest_behaves_identically` in `crates/fidius-host`.

## Prerequisites

- **`wit-bindgen`**: `cargo install wit-bindgen-cli`.
- **wasi-sdk** (its bundled clang + `wasm32-wasip2` sysroot). Point `WASI_SDK` at
  the install (MacPorts: `/opt/local/libexec/wasi-sdk`).

## 1. Generate bindings + implement

`wit-bindgen c` emits a header declaring the functions you must define, named
`exports_<pkg>_<iface>_<func>`. WIT type mapping: strings/lists are
`{ uint8_t *ptr; size_t len; }`; `result<T, _>` is a `bool` return (`true` = Ok)
with out-params:

```bash
wit-bindgen c --world greeter-plugin path/to/greeter/wit --out-dir gen
```

```c
// greeter_impl.c
#include "greeter_plugin.h"
#include <stdlib.h>
#include <string.h>

void exports_fidius_greeter_greeter_greet(greeter_plugin_string_t *name,
                                          greeter_plugin_string_t *ret) {
    size_t n = 7 + name->len + 1;            // "Hello, " + name + "!"
    uint8_t *buf = malloc(n);
    memcpy(buf, "Hello, ", 7);
    memcpy(buf + 7, name->ptr, name->len);
    buf[n - 1] = '!';
    ret->ptr = buf; ret->len = n;
}

bool exports_fidius_greeter_greeter_add(int64_t a, int64_t b, int64_t *ret,
                                        exports_fidius_greeter_greeter_plugin_error_t *err) {
    *ret = a + b; return true;               // the Ok arm of result<s64, plugin-error>
}

void exports_fidius_greeter_greeter_echo_bytes(greeter_plugin_list_u8_t *data,
                                               greeter_plugin_list_u8_t *ret) {
    uint8_t *buf = malloc(data->len);
    for (size_t i = 0; i < data->len; i++) buf[i] = data->ptr[data->len - 1 - i];
    ret->ptr = buf; ret->len = data->len;
}

bool exports_fidius_greeter_greeter_probe_env(void) { return false; }

uint64_t exports_fidius_greeter_greeter_fidius_interface_hash(void) {
    return 0x0102030405060708ull;            // must equal the other guests' hash
}
```

Returned buffers are `malloc`'d; the canonical ABI's `cabi_realloc` (libc
`malloc`/`free`) owns them after the call. The host instantiates a fresh sandbox
per call, so there is nothing long-lived to leak.

## 2. Build the component

The wasi-sdk `wasm32-wasip2` target links **straight to a component** (via
`wasm-component-ld`) — no preview1→preview2 adapter needed:

```bash
"$WASI_SDK/bin/clang" --target=wasm32-wasip2 -mexec-model=reactor \
  --sysroot="$WASI_SDK/share/wasi-sysroot" \
  -I gen gen/greeter_plugin.c greeter_impl.c gen/greeter_plugin_component_type.o \
  -o greeter_c.wasm

wasm-tools validate --features component-model greeter_c.wasm
```

!!! warning "Don't pass `-O2` if `wasm-opt` is on `PATH`"
    With an optimization flag, the wasi-sdk clang runs `wasm-opt` (binaryen) as a
    post-link pass — but binaryen can't parse a Component Model binary and fails
    with *"surprising value (at 0:8)"*. Omit `-O` (the guest is tiny), or ensure
    `wasm-opt` isn't on `PATH` for the link.

The `greeter_plugin_component_type.o` (emitted by wit-bindgen) carries the
component's type information; `-mexec-model=reactor` builds a library (no `main`).

## 3. Package, sign, and load

Identical to any fidius package (`[wasm].component = "greeter_c.wasm"`). The host
loads it through the same `load_wasm` + descriptor as every other guest:

```rust
let handle = host.load_wasm("greeter-c-pkg", &Greeter_WASM_DESCRIPTOR)?;
let greeting: String = handle.call_method(0, &("Ada".to_string(),))?;
assert_eq!(greeting, "Hello, Ada!");
```

## See also

- [A WASM Plugin in Go](wasm-go-plugin.md)
- [A WASM Plugin in JavaScript](wasm-javascript-plugin.md)
- [A WASM Plugin in Python](wasm-python-plugin.md)
- [Your First WASM Plugin (Rust)](../tutorials/your-first-wasm-plugin.md)
