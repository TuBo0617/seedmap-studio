# Local patches

This directory vendors [cubiomes](https://github.com/xpple/cubiomes) at commit
`a49d5bc5b709e6e28b5a74daddb3efb382fba7bb` (MIT). One source-level patch is
applied on top, because otherwise the tree needs a GNU-only `-include` flag at
build time:

## `loot/loot_functions.h`

```diff
-static inline int roll_count_constant(RandomSource*, const int min, int unused_)
+static inline int roll_count_constant(RandomSource *unused_rnd, const int min, int unused_)
```

Upstream uses a C23-style unnamed parameter. GCC and clang accept it, MSVC does
not, and working around it with a forced include breaks on Windows because the
header resolves `#include "../rng.h"` relative to its own location. Naming the
parameter is behaviour-neutral and keeps the header self-contained on every
compiler.

To regenerate from upstream:

```bash
git clone https://github.com/xpple/cubiomes
cd cubiomes && git checkout a49d5bc5b709e6e28b5a74daddb3efb382fba7bb
sed -i 's/RandomSource\*, const int min/RandomSource *unused_rnd, const int min/' loot/loot_functions.h
```
