# gm_cpreprocessor

Another cursed Garry's Mod module. This time, it adds the C preprocessor to Lua scripts.

It works by detouring `RunStringEx` and overriding the executed Lua source code with MSVC compiler, gnu compiler or clang preprocess-only mode output.

# Demo

`garrysmod/lua/cpreprocessor_test.lua`

```lua
#define MACRO(NAME) function Print ## NAME () print("hello world") end

MACRO(HelloWorld)

PrintHelloWorld()
```

```lua
lua_run require("cpreprocessor")
> require("cpreprocessor")...
lua_openscript cpreprocessor_test.lua
Running script cpreprocessor_test.lua...
hello world
```

# Requirements

MSVC compiler, gnu compiler, or clang installed to the system