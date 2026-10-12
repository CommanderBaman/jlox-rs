# Chapter 11

1. Why is it safe to eagerly define the variable bound to a function’s name when
   other variables must wait until after they are initialized before they can be
   used?

   A:
   Eagerly defining a function name enables us recursion.
   It is safe to do so because a function is not evaluated when we visit the
   function body.
   It will only be evaluated until after the whole function definition which
   makes it safe.
   I thought of the fibonacci as a function test while writing this.
   
2. How do other languages you know handle local variables that refer to the same
   name in their initializer?
   Is it a runtime error?
   Compile error?
   Allowed?
   Do they treat global variables differently?
   Do you agree with their choices?
   Justify your answer.

   A:
   I know about three languages:
   python, js, and rust In python, as long as the variable is defined it is safe
   to refer to it.
   Hence, it works that x = x.
   I think it works by copying the value.
   If the variable is not defined before hand, we can not use it.
   It gives an error.
   In JS, we can not redefine a variable that is defined once.
   Also, using an undefined variable gives an error.
   Hence, it does not work that let x = 5; { let x = x; }.
   Here, it works just as lox says - it gives a reference error when the block
   runs.
   In rust, shadowing is enabled but the assignment consumes the first value,
   making access to the first value afterwards invalid.

   I really like rust's method.
   I think shadowing should be allowed because it lets me write a much better
   code.
   There is less room for mistakes.
   There are a lot of wrappers and I want to unwrap them without changing the
   name of the variable.

3. Extend the resolver to report an error if a local variable is never used 

   A:
   Coding Question.
   We need to have another value denoting that a variable was used.
   I think we can keep it in the scope itself alongside the declaration,
   definition value.
   We need to extend it to declaration -> definition -> used.
   We write a use function alongside declare and define functions.
   This use function will be called during the visit_variable in expression.
   When we pop the scope, we check if there are any variables which are not
   used.
   If not used, we raise an error.

   I used this to learn typestate system, but could not apply it as it is not a
   good fit here.
   I want to use a single map for three different types which is only possible
   if it is an enum.

   There was one mistake above.
   I need to find out which scope the variable is defined in before marking it
   as use.


4. Our resolver calculates which environment the variable is found in, but it’s
   still looked up by name in that map.
   A more efficient environment representation would store local variables in an
   array and look them up by index.

   Extend the resolver to associate a unique index for each local variable
   declared in a scope.
   When resolving a variable access, look up both the scope the variable is in
   and its index and store that.
   In the interpreter, use that to quickly access a variable by its index
   instead of using a map.

   A:
   Coding Question - TODO.
   It would change things too much.
   Skipping.
