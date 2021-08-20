# SnakeOS

This is the game snake, bootable on a x86_64 CPU written completely in rust.

[snake](./screenshots/screenshot.png)

## Features

* Play snake on a x86_64 CPU
    * Let's have fun!
* Dynamic memory management
    * The snake can grow!
* Interrupt handling
    * We can read the keyboard!
* Async/Await support
    * We can update the world and read the keyboard at the same time!
* Only 212kB kernel size
    * You can put it on the smalles USB stick you have around!

## Build Commands

Use the Makefile to build the game. 
The only dependencies are `podman` and `buildah` which are used to setup the build enviroment.

```
make snakeos.img
```

This will first setup a build container with the necessary dependencies and then build the game.

To run the game, you can use the following command:

```
make run
```

which will actually run `qemu-system-x86_64 --enable-kvm -drive format=raw,file=snakeos.img` for you

