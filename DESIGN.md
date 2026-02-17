# Foreward
> The computer revolution hasn't happened yet.
> 
> _Alan Kay_

Howl's vision is to be a simple yet powerfully expressive programming environment to bring the computing revolution to everyone. In order to bridge this gap, Howl's syntax attempts to borrow from natural language when possible while remaining readable, clear, and concise.

Howl is _not_ designed to be a high-performance systems language (at least for the low architectures of today). However, it hopes in the long term to enable reliable, dynamic, self-healing, and infinitly scaleable systems.



# Objects and Messages
All values in Howl are **objects**. Objects can be sent **messages**, and may return **responses** to the messages.

A few basic syntaxes are provided for object construction:
```
5                                         // An Integer
```
```
3.14                                      // A Float
```
```
"Hello, world!"                           // A String
```
```
#x                                        // A Symbol
```
```
{ 5, 3, 4 }                               // A List
```
```
{ #x := 5, "Jonathan" := #johnSmith  }    // A Dictionary
```

Messages can then be sent to these objects. For instance, Integer objects understand the '+' message, which responds with a new Integer based on the sum of two input Integers.
```
5 + 3    // Response: an Integer object with value 8
```

`:=` binds Objects to **variables**.
```
x := 5.        // x is now bound to an Integer object with value 5
y := x + 3.    // y is now bound to the response of 'x + 3', which is an Integer object with value 8
```



# Messaging syntax
Message sends can take any of the following forms:

### Unary
These messages simply tell an object to execute an action:
```
Point new.
Log open.
Rectangle new.
```

### Binary
Binary messages allow mathematical notation:
```
1 + 2.
value = 5.    // Either a True or a False object.
```
Binary messages can be chained, and will be evaluated left-to-right.
```
2 + 4 * 3.    // Evaluates as (2 + 4) * 3
```

### Named
```
myList append: 5.
myList at: 4 insert: 33.
```

## Parentheses
A message send can be parenthesised to prioritise its execution. The response returned then replaces the group.

For instance, we can send an unary constructor message, then send a binary message to the response.
```
(Integer random) + 3.    // The (Integer random) message returns a random integer as its response
```
Or, we can send a named message, then another named message to the response of the first:
```
(1 to: 10) map: [ /* ... */ ]    // (1 to: 10) responds with the sequence 1, 2, .., 10
```


## Call Pipelines
A **sentence** is the basic unit of Howl code. It can be formed from one or more messages followed by a terminal period. The following forms are accepted:

### Pipelines
Multiple message sends can be chained together, with each message being sent to the response returned by the previous. These messages are split by arrows.
```
myDictionary at: "John Smith" ->    // This message responds with the Person at the key "John Smith"
    age ->                          // This message requests individual's age from the Person response above
    > 18.                           // This message checks if the age response is greater than 18
    
2 + 4 * 3 ->                        // This computes the mathematical expression
    isEven ->                       // This checks if the response of the computation above is even
    ifTrue: [ /* ... */ ].          // This runs the block (covered below!) if the response above is True
```

### Cascades
Multiple messages can also be sent in sequence to the same object. These messages are split by commas.
```
myDictionary 
    put: johnSmith at: "John Smith",        // Puts a name-object pair into myDictionary
    put: graceHopper at: "Grace Hopper".    // Puts another name-object pair into myDictionary
  
(Button new)                           // A new Button object
    label: "Click me!",                // Sets the label of the new Button
    colour: (Colour hex: 0x40E0D0),    // Sets the colour of the new Button
    whenClicked: [ /* ... */ ],        // Sets the behaviour of the new Button when it is clicked
    display.                           // Displays the new Button on the current Canvas
```




# Types
As we have seen, every object in Howl has a **type**. Types define behaviour (all `Button` instances can be acted on the same way!) and structure (all `Button` instances contain some data that define them). Technically, type objects (such as Integer, Button, Dictionary) are instances of the `Type` type!

New types can be constructed by sending a `named:withInstanceFields:` or `named:withInstanceFields:andTypeFields:` message to `Type`.
```
Type 
    named: #Customer
    withInstanceFields: #(named, age, address, email, preferredLanguage)
    andTypeFields: #(allCustomers)
```
Instance fields vary per instance; each customer has a different first and last name, age, address, etc. Type fields, on the other hand, are common to all instances of the type. Here, we store a registry of all customers as a type field!



# Blocks and Handlers
Blocks are Objects that contain some code.
```
[ 5 + 3 ]    // A block that contains a single sentence
```
The block does not immediately evaluate; the `value` message must be passed.
```
[ 5 + 3 ] value.
```
If the last sentence in the block does not terminate with a period, its value is the block's response.
```
out := [ 5 + 3 ] value.    // out is bound to an Integer object of value 8.
```
Alternatively, a sentence can be prepended with `^`. This escapes all inner blocks, and forces the outermost block to respond with the sentence's value.
```
[
    x := 2.
    x isEven -> ifTrue: [ ^ x + 5 ].    // This makes the outer block exit with a response of 7
    ^ x    // This will never run, but if it did it would exit with the value of x as the response
]
```
Blocks evaluate in an insolated **context**; as such, they do not have access to variable bindings in outer contexts. However, the block can begin with a capture clause that binds variables from the outer context into the inner context.
```
x := 5.
y := 10.

myBlock := [ | x, y | x + y ].    // This block has x and y bound in scope

x := 3.
out := myBlock value.             // out is bound to an Integer of value 13; the current x and y values are used
```
Blocks can also accept **arguments**, or values that specify its behavior. Arguments are specified before captures.
```
z := 3.
myBlock := [ x, y | z | x + y + z ].
```
Blocks which handle arguments are referred to as **handlers**. Handlers can be executed with the `call:` message, and a List or Dictionary (with Symbol keys) containing corresponding values.
```
myBlock call: { 1, 2 }.
myBlock call: { #x := 1, #y := 2 }.
```

## Object Handlers
Continuing the example from above, let us add **message handlers** to the `Customer` type.

On creation, types already have some default message handlers. Three of these are `instanceMessage:handler:`, `typeMessage:handler:`, and `typeConstructor:handler:`, which enable the addition of new handlers. These handlers are then called in response to messages sent to type instances or the type itself, and all instance/type fields are bound into scope during handler execution.
```
Customer 
    typeConstructor: #named:
    handler: [ 
        self, aName
        |
        name := aName.                       // Sets instance field
        preferredLanguage := English.        // Sets instance field
        allCustomers at: aName put: self.    // Sets type field
        self
    ].
Customer
    instanceMessage: #verifyEmail!:
    handler: [
        self, anEmailAddress
        |
        anEmailAddress contains: "@" -> 
            ifFalse: [ MalformedInputError new ]            // Responds with a MalformedInputError
            ifTrue: [ | email | email := anEmailAddress. ]    // Sets the email field and responds with nil
    ]

jane := Customer named: "Jane Doe".
jane verifyEmail!: "janedoe@gmail.com" -> ifError: [ Log append: "Invalid email address". ].
```
