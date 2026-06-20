# Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
#
# FIDIUS-I-0029 / CI: a CONFIGURED *streaming* Python plugin. `base` is bound once
# via __fidius_configure__; tick is a generator that reads it.
__interface_hash__ = __HASH_PLACEHOLDER__


class ConfiguredTicker:
    def __init__(self, config):
        self._base = config["base"]

    def tick(self, count):
        for i in range(count):
            yield self._base + i


def __fidius_configure__(config):
    return ConfiguredTicker(config)
