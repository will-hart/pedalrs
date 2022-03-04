# Pedalrs

## Building

```shell
cargo build --release
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
