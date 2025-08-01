# ahvf

Rust bindings to Apple Hypervisor Framework (for Apple Silicon).

The ahvf bindings are based on [marysaka/ahv](https://github.com/marysaka/ahv).
It uses the exact same frontend (API design), but replace the backend with bindgen FFI.

The plan is to gradually replace the APIs for eaiser integration when developing [simpple-vm](https://github.com/whexy/simpple-vm).

Current roadmap:

- [x] Replace FFI with bindgen.
- [ ] Implement a thread-safe Rust structure.
- [ ]Add macOS 15.0 support.
