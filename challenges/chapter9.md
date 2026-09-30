# Chapter 9

1. A few chapters from now, when Lox supports first-class functions and dynamic
   dispatch, we technically won’t need branching statements built into the
   language.
   Show how conditional execution can be implemented in terms of those.
   Name a language that uses this technique for its control flow.

   A:
   Conditional execution can be implemented via first class functions and
   dynamic dispatch.
   By having first class function, we can encode in them what kind of behavior
   we want from the expressions we attach them to.
   Example, for a falsy value we will execute a different block than for a
   truthy block.
   ```py
   evaluate(expression).if_else(true_statements, false_statements)
   ```
   Basically, dynamic dispatch handles the if else part and first class
   functions carry our code that is to be run.

   Javascript is a language that uses both of these.
   It treats functions as objects, so it supports first class functions.
   Function to run is chosen at runtime, so it uses dynamic dispatch.
   Other is python.
   I think lambda functions prove this.


2. Likewise, looping can be implemented using those same tools, provided our
   interpreter supports an important optimization.
   What is it, and why is it necessary?
   Name a language that uses this technique for iteration.

   A:
   Looping can be done too.
   The problem is that we have to call the same function again and again.
   This might lead to a stack overflow when we call the same function again.

   I read something about it in too-many-lists tutorial.
   We implemented the drop function for the list so that it becomes tail
   recursive.
   Solution is that we know the function is just a repeat with the arguments
   changed, so we can use the same piece of machine code.
   This won't blow up the stack.

   After some research, this technique is called Tail Call Optimization

   I believed python had it because it contained generators, but no.
   That isn't it.
   My language knowledge is not broad enough.


3. Unlike Lox, most other C-style languages also support break and continue
   statements inside loops.
   Add support for break statements.

   The syntax is a break keyword followed by a semicolon.
   It should be a syntax error to have a break statement appear outside of any
   enclosing loop.
   At runtime, a break statement causes execution to jump to the end of the
   nearest enclosing loop and proceeds from there.
   Note that the break may be nested inside other blocks and if statements that
   also need to be exited.

   A:
   I can't really find the answer here.
   What I am not able to do is, divide the work between the parser and grammar.
   If the grammar is too loose, the runtime suffers.
   If the parser is too strong, grammar suffers.
   
   Context:
   Let me decide on the grammar. 
   
   ```
   breakStmt -> "break" ";"
   blockStmt -> "{" (declaration | breakStmt)* "}"
   stmtWithBreak -> statement | breakStmt
   whileStmt -> "while" "(" expression ")" stmtWithBreak
   forStmt -> "for" "(" varDecl | exprStmt | ";" )
               expression? ";" expression? ")" stmtWithBreak
   ifStmt -> "if" "(" expression ")" stmtWithBreak ( "else" statement )?
   ```
   
   I believe the problem starts with break inside if statements of loop.
   I can't really include it in the grammar as is, because the grammar does not
   support conditionals.
   I have to think of some other way.
   
   I will leave the implementation for future Shubham 🫡.
   I have to add the token which is easy but the statement implementation is a
   problem.
   Maybe, if I make a variable in the interpreter and check for it at end of
   every while loop.
   
   It is hard but lets have a basic thing running first.
