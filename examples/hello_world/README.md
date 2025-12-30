# Expected behavior:

Build command: ```plc build HelloWorld.json```

creates ```build/``` folder and emits following files:

- HelloWorld
- hello_world.st.o
- __init___HelloWorld.o

executing resulting program results in
```
$ ./build/HelloWorld
hello, world!
```

in the HelloWorld.json "compile_type" : "Static" results in an executable and "linker": "cc" is picked because unlike lld, it links libc by default, which is needed for puts to work.
