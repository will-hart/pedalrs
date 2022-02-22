# Pedalrs

To build

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
