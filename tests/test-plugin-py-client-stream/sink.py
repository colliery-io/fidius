# Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
#
# CS2.4 fixture: a CLIENT-streaming Python plugin. `load` receives a host-fed
# iterator (`rows`) the host produces, iterates it (`sum`), and returns the total.
__interface_hash__ = __HASH_PLACEHOLDER__

from fidius import method


@method
def load(rows):
    return sum(rows)
