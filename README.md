# Pedalrs

## Building

`memory.x` and `.cargo/config` files are automatically copied over based on the
microcontroller being used. Select the microcontroller by using `features`. By
default, `stm32f0` is selected:

```shell
cargo build --release
```

> **WARNING** The very first time a build is run on a fresh repo, or when
changing between chips it is advisable to build twice as the `memory.x` and
`.cargo/config` files don't appear to be picked up on the first run. It may also
be advisable to `cargo clean` between builds.

To build for STM32F1 chips, run

```shell
cargo build --release --features stm32f1
```

To check the built size

```shell
cargo size --release
```

To deploy on STM32F103C8 (blue pill) target

```shell
cargo flash --chip stm32f103c8 --release
```

To deploy on STM32F042 target

```shell
cargo flash --chip STM32F042F4Px --release
```

## Debugging

Debugging on device requires two terminals. In one terminal:

```shell
openocd -f interface/stlink.cfg -f target/stm32f0x.cfg
```

In a second terminal:

```shell
arm-none-eabi-gdb -q ./target/thumbv6m-none-eabi/release/pedalrs -ex "target remote :3333"
```

Once GDB loads

```gdb
l
b main
c
```

This will stop at the start of the main function. From there you can use normal `gdb` commands to debug.
