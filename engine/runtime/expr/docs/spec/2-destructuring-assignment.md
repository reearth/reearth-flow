# (DRAFT) Destructuring Assignment

## list destructuring

A bracket-enclosed list of names on the left-hand side of `=` unpacks the right-hand side into those variables:

```
[a, b] = [1, 2]
```

After this, `a` is `1` and `b` is `2`.

The right-hand side is fully evaluated before any binding occurs. This enables the swap idiom:

```
[a, b] = [b, a]
```

## destructuring in for-in

A bracket-enclosed list of names may also appear as the loop variable in `for ... in`, destructuring each element of the iterated value:

```
for [k, v] in pairs {
    ...
}
```