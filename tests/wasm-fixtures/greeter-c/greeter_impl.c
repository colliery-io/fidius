// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// Non-Rust polyglot guest (FIDIUS-I-0025): a C implementation of the SAME
// `greeter` WIT as the Rust/Python/JS/Go guests. Bindings by wit-bindgen (C);
// compiled to a component with the wasi-sdk clang. No runtime is embedded, so
// the component is tiny.
#include "greeter_plugin.h"
#include <stdlib.h>
#include <string.h>

void exports_fidius_greeter_greeter_greet(greeter_plugin_string_t *name,
                                          greeter_plugin_string_t *ret) {
    static const char pre[] = "Hello, ";
    size_t n = 7 + name->len + 1;
    uint8_t *buf = malloc(n);
    memcpy(buf, pre, 7);
    memcpy(buf + 7, name->ptr, name->len);
    buf[n - 1] = '!';
    ret->ptr = buf;
    ret->len = n;
}

bool exports_fidius_greeter_greeter_add(int64_t a, int64_t b, int64_t *ret,
                                        exports_fidius_greeter_greeter_plugin_error_t *err) {
    (void)err;
    *ret = a + b; // the Ok arm of result<s64, plugin-error>
    return true;
}

void exports_fidius_greeter_greeter_echo_bytes(greeter_plugin_list_u8_t *data,
                                               greeter_plugin_list_u8_t *ret) {
    uint8_t *buf = malloc(data->len);
    for (size_t i = 0; i < data->len; i++) buf[i] = data->ptr[data->len - 1 - i];
    ret->ptr = buf;
    ret->len = data->len;
}

bool exports_fidius_greeter_greeter_probe_env(void) { return false; }

uint64_t exports_fidius_greeter_greeter_fidius_interface_hash(void) {
    return 0x0102030405060708ull; // must equal the other guests' hash
}
