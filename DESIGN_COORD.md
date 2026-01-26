- Connected nodes networked together
- "Compute-sharing" AND "data-sharing" system
- 'Spawn' blocks carry with them a copy of state at start -> sets state at end (atomic block) OR at 'yield' points
- Maybe also have their own message queue? 
- Maybe can register handlers themselves? or just pattern match on incoming messages?

- Only one thread can access global state, and has a ringbuffer
- Other threads can drop messages into the ringbuffer (end of block OR yield)
- The main thread checks the ringbuffer often to update state
- At system startup the interpreter spawns N threads for N cores
- Scheduler schedules 'spawn' blocks on threads or remote systems on the network
- What if the return value is a set of messages? Rather than just state
- Maybe way to tag public resources on a node -> allow others to 'see'

- Blocks are run in a context with zero state -> attempted access of globals just doesn't work
- Maybe you can select a block to run on with a resource? Like get a Processor object and spawn on that system? Not sure how immutability is preserved though...

- If you have a public resource -> all subresources are public
- PublicObjectList or something -> can copy to current device
    - Can copy and sync? Think syncing UI
    - Maybe would require TeaTime or something
    - Imagine could take another system's Ui state and have it in a window
    - Handler on button click -> waiting on incoming message (empty mailbox)? (Need a yield mechanism... maybe for IO as well)
    - How does it deal with button state get/set?
        - Standard fine-grained locking (BAD)
        - TeaTime?
        - Single-actor?
            - Send message to owning thread queue
            - Wait for response?
        - Atomic transaction (STM)


- What if each Object is actually a TransientObject that evolves over time (each 'state' is immutable)
- Changes to Objects can thus be rolled back -> each change is timestamped and checked to 

Both systems hold a UI object
System 2 wants to change Button
It does an atomic update to Button (STM)

State needs to exist inside an isolation container which is separated from rest of system -> kind of like a thread!
We can synchronize isolation containers accross systems for replicated determinism

# Island
- SHARED MUTABLE STATE
- An object with internal state
- Isolated from rest of system
- No pointers, etc -> 'functional object'
- Exists on the network -> other peers can 'join' the island
    - Joined clients contain a replicated copy of the island
    - TeaTime synchronization

# Glacier (Frozen Island)
- SHARED IMMUTABLE STATE
- Entirely immutable -> 'frozen'
- One copy per physical device (multicore can access without locking)
- No replicated copying necessary
- If any processes are acting on the glacier (hold a ref), it cannot be thawed

# Atomic transactions
- SMALL INSTANTANEOUS TRANSACTIONS
- Uses STM system on some captured context
    - Transitions each context object to a wrapper STMObject?
