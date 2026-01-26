# Object Table
```rust
struct Value(u64);
struct ObjTable(Vec<ObjPtr>);
struct ObjPtr {
    ptr: u64,
}

struct List {
    elem_ty: u64,
    start: ObjPtr,
    len: u64,
    cap: u64,
}
```

Global -> ObjTable -> Heap(List) -> ObjTable -> Heap(ListElement)


# Tuple
## Stack
Ptr->
## Heap
| TypeID, HashMap<id, Value> |

# List
## Stack
Ptr->
## Heap
|
