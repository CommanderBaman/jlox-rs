# Chapter 12

1. We have methods on instances, but there is no way to define “static” methods
   that can be called directly on the class object itself.
   Add support for them.
   Use a class keyword preceding the method to indicate a static method that
   hangs off the class object.

   A:
   Coding Question

   This requires changes to the grammar.
   Leaving it to future Shubham.
   TODO!

2. Most modern languages support “getters” and “setters”—members on a class that
   look like field reads and writes but that actually execute user-defined code.
   Extend Lox to support getter methods.
   These are declared without a parameter list.
   The body of the getter is executed when a property with that name is
   accessed.

   A:
   Coding Question

   This requires changes to the grammar.
   Leaving it to future Shubham.
   TODO!

3. Python and JavaScript allow you to freely access an object’s fields from
   outside of its own methods.
   Ruby and Smalltalk encapsulate instance state.
   Only methods on the class can access the raw fields, and it is up to the
   class to decide which state is exposed.
   Most statically typed languages offer modifiers like private and public to
   control which parts of a class are externally accessible on a per-member
   basis.

   What are the trade-offs between these approaches and why might a language
   prefer one or the other?

   A:
   I believe the trade off is between flexibility and safety.
   Languages like JS and Python, expose variables and one can easily modify
   whatever they want.
   This makes the code easy to write.
   I think scripting languages use this a lot.
   On the hand, exposing variables like this make our functions prone to side
   effects when it runs.
   A function result might change due to a variable change.
   This makes writing production apps really hard to write/debug.
   There are also various performance gains you get by soldifying what a
   variable can contain otherwise you always have to allocate a pointer or the
   max of all types it can contain, which is not good for either speed or
   memory.

   As an avid enjoyer of Rust, I really like when I can control what I can
   expose.
   This makes me want to break down the tasks into really small functions and
   only expose the main ones without worrying the small ones populating the
   interface.
