# Tunnel

**Tunnel** is a library for easy sending and receiving of data over P2P.

Fundamentally, Tunnel is just a small wrapper over [iroh](https://github.com/n0-computer/iroh), a convenient P2P library.

# Python bindings

While most of the API is the same between the native Rust version and the Python bindings, there are some differences:

- Currently, **all asynchronous code is contained in a blocking API**. This is because async support in PyO3 (the bindings Tunnel uses) is currently experimental.