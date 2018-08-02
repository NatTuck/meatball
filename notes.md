

# Simple Version

Three kinds of value:

 - Integer
 - Function
 - Builtin

(fn foo (a b)
  (+ a b))

Locals:
 - slot 0: cont
 - slot 2: a
 - slot 3: b

Compiled function:
 * Allocate frame

Continuation:
 * Parent stack ptr.
 * Parent return addr.

Call sequence:
 * %rbp = malloc(frame)
 * mov (a), 0(%rbp)
 * mov (b), 8(%rbp)
 * %r10 = %rsp
 * %rsp = %rbp 
 * push %r10
 * call name
 * ...
 * ret
 * pop %rsp

