# Clashi

> [!CAUTION]
> WIP project

![showcase](https://github.com/user-attachments/assets/ec04356a-c0e1-4264-b1c8-40bb574e3883)

This only works as a dashboard for clash backend:

- Check groups && proxies
- Select proxy
- Update provider

For Clash configuration operations, please checkout [clashtui](https://github.com/JohanChane/clashtui).

## Keybinds

Does not support keybind customization yet.

### Tabs

```
tab/shift+tab: swith page between group and provider
```

### Group Page

```
hjkl/←↓↑→: select group
space/enter: enter proxy page of the current selected group
r: latency test for current selected group
```

### Provider Page

```
hjkl/←↓↑→: select provider
space/enter: enter proxy page of the current selected provider
f: update provider subscription
```

### Proxy Page

```
jk/↓↑: select proxy
r: latency test for current selected proxy
R: latency test for current group
```

## TODO

- secret
- Search/Filter groups&&proxies
- Keybind info
- Integrate with calloop or other eventloop to save resources.
