[![progress-banner](https://backend.codecrafters.io/progress/shell/251478ed-74d1-4cf2-aeb6-7f15fad6fac9)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

## Learnings

- Rust is awesome, but we all know that, right?

- Comparing the Rust vs. Go solutions, I have to say – the Rust has much more utility functions built-in. This made solving this problem easier.

  - But, there is a downside to Rust as well. It is not that easy to compose _traits_ together since you have to, sometimes, worry about lifetimes.

- The structure I've created where we `.parse` a string to `Command` enum works fine, but is a bit awkward.

  - Most of the operations happen within each command. That is a good thing.

  - The problem is that I can't inject dependencies there.

    - The solution is to probably use a `struct` somehow.
