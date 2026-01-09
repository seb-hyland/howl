# Type system
## Anonymous Types
```typescript-ts
// Tuple
[var v #[2.0; 3]]            [var w v.0]    [typecheck v #[Float; Int]]
[var v #[x = 4.0; y = 10]    [var w v.x]    [typecheck v #[x = Float; y = Int]]
```
## Named Types
```typescript-ts
[type Point
    #[
        x = Float;
        y = Float;
    ]
    #[color = [Float, Float, Float]]
]
[var v [Point #[x = 3.0; y = 5.5]]]
[var w v.x]
```
