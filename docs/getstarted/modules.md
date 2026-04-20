# Modules

Koi has a very simple module system. Subdirectories of your source directory (`src`) are submodules of your program. All files in the same module are part of the same compilation unit and share symbols and declarations. 

```
/src
    /math
        add.koi
    main.koi
```

```go
// add.koi

// Marking symbols with `pub` exports them
// and lets you import them from other modules.
pub func add(a int, b int) int {
    return a + b
}
```

```go
// main.koi

import math

func f() {
    sum := math.add(1, 2)
}
```

## Importing

You can import symbols by name to omit the prefixed module name when using them:

```
import math { pow }

import io {
    println,
    write,
    close,
}
```

You can also alias imported namespaces using the `as` keyword. This is useful when multiple submodules share the same name:

```
import routes.user as userRoutes
import models.user as userModels
```

Combining named imports and aliasing is not allowed.
