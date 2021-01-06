# Async

Everything is sync by default. If you want to run something asynchrounous you have to use the `async` keyword.

```nog
async || {
  // I run async
};
```

It is important to know that code running in async is not truly async. It can still block the main thread.

```nog
async || {
  loop {
    // I block the main thread => really slows down nog
  }
};
```

To avoid blocking the main thread you can use the `sleep` keyword, which takes a number that represents the amount of milliseconds to sleep for.

```nog
async || {
  loop {
    // I no longer block the main thread
    sleep 100;
  }
};
```
