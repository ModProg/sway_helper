# sway_helper


# Layout Algorithm

Checks what windows are available and reduces the layout accordingly.

Example (window F, B and D don't exist):

```
A|D|F    A|E
-|D|F    A|E
B|-|F => -|E
-|E|F    C|E
C|E|F    C|E
-----    ---
GGGGG    GGG
```
