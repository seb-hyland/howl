# Type system
## Anonymous Types
Values are assigned in their scope via the `=` shorthand.
```typescript-ts
v = 2;
```
Tuples can be constructed using parentheses, and accessed by index.
```typescript-ts
v = (2.0, 3);              w = v.0;
v = (1.0, "Hi!", 5);       w = v.2;
```
Lists are a 'special case' of tuples for an unspecified number of items of homogeneous type.
```typescript-ts
v = (2.0, 3.0, 1.0);       w = v.1;
```
Tuples can also have named fields, which are accessed by name.
```typescript-ts
v = (x = 4.0, y = 10);     w = v.x;
```
All data is represented as a tuple.
```typescript-ts
5 == (5)     // true
```

## Named Types
Types are constructed via the `type` keyword. Fields prefixed by `@` are 'type fields' rather than 'instance fields'.
```typescript-ts
type Point (
    x, y,
    // Field shared between all Point instances
    @color,
)
```
Instances of a type can be constructed by default with named-tuple syntax, in which instance fields are initialized.
```typescript-ts
v = Point new (x = 3.0, y = 5.5);        w = v.x;
```
Type fields belong to the type, and are initialized/accessed as such.
```typescript-ts
Point.color = (255.0, 0.0, 0.0);     w = Point.color;
```
The `opaque` modifier can be used to disallow access of a field outside of the type's message handlers.
```typescript-ts
type Ui (
    opaque widgets,
    opaque @bg_color,
)
// Error: cannot initialize opaque field `buf` directly
ui = Ui (widgets = (my_button, my_text));
// Error: cannot access opaque field `buf`
col = Ui.bg_color;
```

## Block/lambda definitions
Blocks (with form `[ body ]`) are equivalent to the evaluation of `body`
```typescript-ts
v = [ 5 ]               // 5
v = [ 5 + 4 ]           // 9
v = [ [5 + 4] > 7 ]     // true
```
Lambdas (with form `[ arg1, arg2, ... | body ]`) define a lambda that can be evaluated by passing args as a tuple.
```typescript-ts
add = [ num1, num2 | num1 + num2 ]
double = [ num | num * 2 ]
```

## Handlers
Lambdas can be associated with a type and gain special access to the type's fields even if they are opaque. These are called **instance handlers**, and take the special `self` keyword as their first argument.
```typescript-ts
type Point (
    x,
    y,
    opaque origin,
)
Point << relative_x [ self | self.x + origin.x ]
```
_Type_ handlers, on the other hand, are associated lambdas which do not act on a specific instance. As such, they do not take `self`. This distinction is somewhat akin to instance fields vs type fields.
```typescript-ts
Point << @whatami [ | "A point!" ]
```
Evaluating one of these lambdas is referred to as 'message passing'. As such, the lambdas are referred to as 'handlers', since they handle incoming messages. Message passing takes the syntatic form `<TYPE>/<INSTANCE> <MSG> <ARG1> <ARG2> ...`.
```typescript-ts
p = Point new (x = 2, y = 0)
x = p relative_x
v = Point whatami

Point << add_x [ self, x | self.x = self.x + x ]
p add_x 2
```
In fact, `<<` and `new` are handlers too! Point is a `Type` instance in the global namespace, and the `<<` takes an `Ident` (handler name) and `Lambda` (handler lambda), and adds it to the type.


Arguments can be prefixed with a `#` to indicate they should NOT be evaluated, and rather passed as raw syntax
```typescript-ts
if_not = [
    cond,    // Standard argument
    #body    // MACRO argument
    |
    cond if_true body
    // `if_true`/`if_false` take syntax and evaluate it conditionally
    // as such, we pass body as raw syntax
]
setf = [
    #names,                      // a raw data definition of form (val, ..)
    #op: [ self | self == #= ],  // a 'restriction statement'
    body
    |
    names values for_each [ name: Ident | Globals set name body ]
]
// Usage:
setf (v1, v2, v3) = 3
```
## Restriction statements
Must be an ident/tuple (`Type`) OR block OR lambda taking singular `self` argument (representing the value of the argument)
Each statement in the block must evaluate to a `Type`, `Bool`, or `Trait`
- If `Type`, the arg is checked for the property `self typeof == 'Type'`
- If `Bool`, checked for property `true`
- If `Trait`, checked that all `Trait` methods are implemented for `self typeof`
```typescript-ts
[ arg: Int | ] 5;
[ arg: [List; Collection] | ] (1, 2, 3);
[ arg: [
     self
    |
    List;
    self for_each [ item | [item typeof] == Int ]
    ]
|
] (1, 2, 3);
```
On lambda blocks, a return restriction can be placed:
```typescript-ts
[ x: Int, y: Int -> Int | ^[x + y] ]
```
On type definitions, restrictions can be placed which are checked on field access:
```typescript-ts
Point = type (
    x: Numeric,
    y: [self | Numeric; self != 0]
)
```
## Traits
```typescript-ts
trait Collection (all, for_each, first, nth)
```







# Playground
```typescript-ts
// OR fields: a type that can be one of its field variants
// Variants can be modified later at runtime
type Shape (
    Point(p) ||
    Line(start, end) ||
    Bezier(start, mid, end)
)
// AND fields: a type that has ALL of its field variants
// Variants can be modified later at runtime
type Point (
    x: Float,
    y: Float,
    // Variable shared by all elements of the class
    // Changing its value will update all `Point` instances
    ~render_color: (Float, Float, Float),
)

type Response (
    clicked: bool,
    hovered: bool,
)
// Sets the `place` global to the type of this function signature
place = |Shape| -> Response;
type Window (
    buf, // Some opaque reference from Rust
)
Window << place [self, element |
    element
        if_case Shape.Point [point | self.buf draw_point point.p],
        // `if_case` is a function that runs EXPR if an OR type matches VARIANT
        // Commas at EOL signify to run the next thing on the same input
        // here, they run if_case multiple times on `element`
        if_case Shape.Line [line | self.buf draw_line line.start line.end];

    bezier_callback = [bezier | self.buf draw_bezier line.start line.mid line.end];
    if_case Shape.Bezier bezier_callback;
]
window = Window (buf = /* magic initialization */)

// Type restrictions are OPTIONAL on everything, they simply add a runtime guard
// to ensure proper data is passed
draw = || -> Response;
// Implements draw's signature for the `Point` type
Point << draw [ | window place Shape.Point(self) ]

// Construct a Point instance
p = Point (x = 3, y = 5)
p draw window

// Defines a macro
// `#` keeps body from being evaluated as a function argument
// Instead, it is passed as an Expr: LITERAL CODE
// If cond evaluates to true, THEN the body is executed
when = [cond, #body |
    cond if_true body*
]
typeof when // evaluates to |_, _|

// Defines a macro to bind a global variable
// Binds the evaluation of body to `name`
// Name is not evaluated, but passed as a literal identifier
// Argument guards can also be expressions: here, we check that `#sym` is a literal equal sign
var = [#name: Expr.Ident, #sym: [_ == #=], body |
    Globals add #name body
]
var v = 5

Collection = Trait new (#for_each, #do_each, #all, #iter_type) // List of behaviours
// Defines a macro that binds each given input name to body
setf = [
    #names: [ self |
        Collection; /* ensures the `#names` input implements all Collection behaviours */
        self iter_type == Expr.Ident /* ensures each item in the Collection is an identifier */
    ],
    body
    |
    names do_each [name | var name = body]
]
```
