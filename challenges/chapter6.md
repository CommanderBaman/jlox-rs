# Chapter 6

1. In C, a block is a statement form that allows you to pack a series of
   statements where a single one is expected.
   The comma operator is an analogous syntax for expressions.
   A comma-separated series of expressions can be given where a single
   expression is expected (except inside a function call’s argument list).
   At runtime, the comma operator evaluates the left operand and discards the
   result.
   Then it evaluates and returns the right operand.

   A:
   I thought of three solutions
   * changing primary to primary → ...
     | "nil" | "(" expression ")" | "," expression but this allows expressions
     like ", 1 + 2"
   * changing equality to equality → comparison ( ( "!=" | "==" | "," )
     comparison )* but this causes problems with cases like "x, y == z" not to
     mention we need to add , as a binary operator which might cause problems
     with evaluation later on - just an intuition
   * changing expression to expression -> equality ( "," equality ) but in order
     to capture this we need to change the return values here I don't want to
     disturb the current type system so.
     otherwise, we have to return a vector everywhere instead of just expression

2. Likewise, add support for the C-style conditional or “ternary” operator ?:.
   What precedence level is allowed between the ?
   and :?
   Is the whole operator left-associative or right-associative?

   A:
   ternary operator is right associative.
   ex, 
   ```js
   check1 ? true1 : check2 ? true2 : false2 
   ```

   boils down to

   ```js
   check1 ? true1 : (check2 ? true2 : false2)
   ```

   as for precedence, it comes between equality and comparison


3. Add error productions to handle each binary operator appearing without a
   left-hand operand.
   In other words, detect a binary operator appearing at the beginning of an
   expression.
   Report that as an error, but also parse and discard a right-hand operand with
   the appropriate precedence.

   A:
   adding error productions for binary operator is easy, just add them at the
   equality step accepting the operators problem is that with this typing, we
   can't form statements with them that is why i didn't add error production for
   + at unary for such cases i think having a very verbose parse error helps
