# OOStuBS++ interrupt synchronization model


(I call it OOStuBS++ because this also involves the userspace in the future)

Note that these levels are not necessarily "privilege levels". It's more of a
"priority of execution" level.

## Definition of synchronization levels

L0: the user context, lowest privilege level, can spin, but can not
    disable/enable irq

L1: the preemptive kernel context that can be interrupted anytime and doesn't
    need synchronization

L2: the "synchronized" kernel context, that can be interrupted, but must be
    linearized, in otherwords, non-preemptive. Scheduling can't happen in L2:
    from another perspective, scheduling can only happen at the linearization
    points on L2.


L3: the "hard irq" kernel context, where interrupts are masked. This is critical
    critical section where you must not block. For example, if you block on a
    spinlock here it's hopeless that the lock holder(the lock holder could be
    either your normal context, or another thread) would release the lock.

## prologue/epilogue model: an rough overview
```text
======================A simplified, single-threaded model=======================

     1.
L0---+                                                                    .USER
     |                      7.
L1---+                      +-------------------------------------.KernelNormal
     |  3.       6.         |
L2   |  +<aaaa+  +aaaa><bbb>+                                       .KernelSync
     |  |     |  |
L3   +AA+     +BB+                                                   .KernelIRQ
     2.       4. 5.
```
1. a hardware interrupt happens in either user or normal kernel context. The
   execution is taken to the interrupt handler and the IRQ is temporarily
   masked
2. the execution is taken to the responsible handler prologue (named A).
3. the handler requires an epilogue (bottom half). The epilogue object (named a)
   is enqueued. We need to clear the epilogue queue before returning to the
   normal context, _so we loop and dequeue the epilogue queue until it's empty_,
   and the queue happens to have exactly one entrant a, so we dequeue the
   object, re-enable irq (entering L2) and run the epilogue `<a>`. (actually,
   since we are seeing an empty queue, we could immediately execute the
   epilogue and return)
4. the execution of `<a>` is interrupted by another hardware, like 2. we are
   taken to the prologue of the handler B.
5. like 3. the handler B also requires an epilogue `<b>` so we enqueue it. and
   take it back to L2.
6. the epilogue `<a>` resumes and runs to complete, however the epilogue queue
   is no longer empty, so `<b>` is dequeued to run. This time no further
   interrupts happens until `<b>` completes. We finally return to the normal
   context.


## prologue/epilogue model: a detailed view
```text
======================Let's refine the details!=================================

     1.
L0---+                                                                    .USER
     |                      7.
L1---+                      +-------------------------------------.KernelNormal
     |  3.       6.         |
L2   |  +<aaaa+  +aaaa><bbb>+                                       .KernelSync
     |  |     |  |
L3   +AA+     +BB+                                                   .KernelIRQ
     2.       4. 5.
       ^
       |
    "locks L2"
```

in Step 2, the L3 code sees an empty epilogue queue, so it executes the
epilogue `<a>` immediately without enqueue. Which means that in Step 4 and 5.
the L3 code also sees an empty queue! This also happens if the L2 code from the
_last_ epilogue entrant is interrupted: the interrupting L3 code wants to
execute its epilogue immediately but it shouldn't!

The insights here is that the epilogue queue being ready or not does not
necessarily suggest whether L2 is available. So we need .... some sort of a
lock of a lock!

Here we have another global variable to indicate whether the L2 is available.
Now we have an infinite recursion of need a lock: how to guarantee that the
`L2_AVAILABLE` flags itself is available, that is, no one else is acting on a
`L2_AVAILABLE` assumption and hasn't yet updated that flag?

Luckily the recursion ends if you can guarantee that the critical sectionis
only in L3! We won't have race condition (on the same CPU) when interrupt is
disabled!

## pro/epilogue model + scheduling
```text
|thread 1 runs>->->->->->->->->->->-> |thread 2 runs ->->->->->->->

         1                   7^        9^
L1-------+                   +-+       +***************************.KernelNormal
         |  3.       6.      | |       |
L2       |  +<aaaa+  +aaaa>+ | +<bbb>+ |                             .KernelSync
         |  |     |  |     | |       | |
L3       +AA+     +BB+     +-+       +-+                              .KernelIRQ
         2.       4. 5.    7v.deq    8.deq empty
```

Let's assume that
- A is a keyboard interrupt, where the prologue A reads and parses the
  registers from the keyboard controller and the epilogue push the input into a
  input buffer.
- B is a timer interrupt, which forces a context switch (if possible) to
  another ready task in the run queue.

```text
1.      thread 1. is running in normal kernel context
2~3.    are the same as the previous example.
4.      prologue of the timer interrupt: we dequeue and enqueue the run queue
        here but do not immediately do context swap, because the synchronized
        epilogue <a> is still running. Also the run_queue must be synchronized
        but we don't want to spin on it, so we borrow the prologue where race
        can't happen on single CPU.
5~6     are the same as the previous example.
7v/7^   this is one omitted detail from the previous diagram: in between the
        execution of each queued epilogue, there is a brief switch to L3 (7v)
        because the epilogue queue must be protected. Also there is a
        linearization point (7^) after the irq gets re-enabled and before the
        next epilogue is started, hence the brief jump back to the normal
        context.
<bbb>   the epilogue of the timer interrupt compute time slices and check
        whether the running task should be preempted and tell the deq loop (via
        a flag variable or something) that we are doing a scheduling at the next
        synchronization point (that is, 9). Note that we are doing slightly
        differently than the OOStuBS, where the context swap is directly done
        directly in the timer epilogue.
8.      like 7, the but the epilogue queue is empty this time (well it doesn't
        really mater if the queue is empty because we have synchronization
        anyways)
9.      the dequeue loop hits the next synchronization point and calls
        do_schedule(). The execution is taken to thread 2.
```

## putting everything together

here is how the interrupt handler work:
```text


              LEVEL 1
                 |
         INTERRUPT ENTRY
                 |
                 | (IRQ automatically disabled)
 +---------------|-------------------------------------------+
 |               v                                  LEVEL 3  |
 |       [FIND HANDLER]                                      |
 |               |                                           |
 |               v                                           |
 |   [EXEC. HANDLER.PROLOGUE]                                |
 |               |                                           |
 |               |            (optionally clear the queue)   |
 |        _______V______     N          _______________________
 |       <needs epilogue?>------->     [RESTORE CTX AND .. IRET]------> LEVEL 1
 |        ```````````````               ``^````````````^```````
 |               |                        ^            ^     | IRQ automatically
 |               | Y                      |            |     | enabled upon IRET
 |        _______V________   N            |            |     |              ^^^^
 |       <is L2 available?>---->[ENQUEUE EPILOGUE]     |     |
 |        ````````````````                             |     |
 |               |                                     |     |
 |               |   +----------[DEQUEUE EPILOGUE]     |     |
 |               |   |                    ^            |     |
 |               |   |               N    |            |     |
 |               |   |        ____________|________  Y |     |
 |               |   |       <epilogue queue empty?>---+     |
 |               |   |        ````````````^`````````         |
 |               V   V                    |                  |
 |      [SET L2 OCCUPIED]        [SET L2 FREE]               |
 |               |                        ^                  |
 |               V                        |                  | <- TODO:
 +--------[ENABLE IRQ]-----------[DISABLE IRQ]---------------+    need a limbo
 |               V                        |         LEVEL 2  |    where we do
 |     [EXEC. HANDLER.EPILOGUE]           |                  |    resschedule
 |               |                        |                  |
 |               +------------------------+                  |
 |               check and clear the epilogue queue          |
 |               before leaving!                             |
 |                                                           |
 +-----------------------------------------------------------+
```
