[![progress-banner](https://backend.codecrafters.io/progress/shell/251478ed-74d1-4cf2-aeb6-7f15fad6fac9)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

## Learnings

- Rust is awesome, but we all know that, right?

- Comparing the Rust vs. Go solutions, I have to say – the Rust has much more utility functions built-in. This made solving this problem easier.

  - But, there is a downside to Rust as well. It is not that easy to compose _traits_ together since you have to, sometimes, worry about lifetimes.

- You can encapsulate _a lot_ if you use traits.

  - I especially love the fact that you can create methods on enums.

  - Let us not forget about the ability to provide default implementations!

- Implementing `FromStr` on the `Command` made the "main" code so concise.

- **Embedding `traits` inside `struct`s is quite hard for me**.

  - First, **since the "size" of the `trait` can only be known at runtime, you have to use `dyn` and `Box`**.

    ```rust
      struct Foo {
          prompter: Box<dyn Prompter>,
      }
    ```

    You can "hide" this complexity by adding the `new` method and wrap the trait with `Box` there.

    ```rust
      impl Foo {
          fn new(prompter: impl Prompter) -> Self {
              return Self {
                  prompter: Box::new(prompter),
              };
          }
      }
    ```

    Still, as a Rust novice, I find it hard to wrap my head around these concepts.

  - Second, and this applies to all struct properties, **as soon as you want to have a pointer to something as struct property, you have to deal with lifetimes**.

    - This makes total sense, but it introduces a lot of complexity.

- Writing tests in Rust is a joy.

  - Side-note: I'm happy that we Node finally has a native test runner!

- The _match pattern guard conditions_ in Rust never cease to amaze me. You can do stuff like the following:

  ```rust
  ' ' if some_condition => {}
  ```

  So, you are matching on specific character BUT only if the condition is met.
