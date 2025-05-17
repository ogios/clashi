# Clashi

> [!CAUTION]
> WIP project

This only works as a dashboard for clash backend:

- Check groups && proxies
- Select proxy
- Update provider

## Showcase

### Group

![showcase-group](https://github.com/user-attachments/assets/09429241-8b4f-4bf3-b81e-f0d8f4230c08)

### Provider

![showcase-provider](https://github.com/user-attachments/assets/34b12dac-7e55-4614-8a49-09cf6cf911b2)



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
