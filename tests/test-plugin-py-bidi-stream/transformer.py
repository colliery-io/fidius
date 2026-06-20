# Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
#
# BD.4 fixture: a BIDIRECTIONAL Python plugin. `transform` receives a host-fed
# iterator (`rows`) AND returns a generator — it yields each input doubled, lazily
# (each `yield` pulls one input via the host-fed iterator).
__interface_hash__ = __HASH_PLACEHOLDER__

from fidius import method


@method
def transform(rows):
    for r in rows:
        yield r * 2
