# Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
#
# FIDIUS-A-0006 / CI.4: a CONFIGURED Python plugin. Instead of module-level
# @method functions (the singleton model), the module exports
# `__fidius_configure__(config) -> instance`; the host binds methods on the
# returned instance, so the config is bound once and N differently-configured
# instances coexist. Implements the BytePipe interface (reverse + name); `name`
# returns the configured display name.

# Replaced at stage time with BytePipe_PYTHON_DESCRIPTOR.interface_hash.
__interface_hash__ = __HASH_PLACEHOLDER__


class ConfiguredPipe:
    def __init__(self, config):
        self._name = config["display_name"]

    def reverse(self, data):
        return bytes(reversed(data))

    def name(self):
        return self._name


def __fidius_configure__(config):
    """Bind the config once and return the configured instance."""
    return ConfiguredPipe(config)
