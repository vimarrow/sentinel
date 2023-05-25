# ![Screenshot 2023-05-25 at 08 34 21](https://github.com/vimarrow/sentinel/assets/43005064/58281bfa-af12-43bd-b1f8-5330afaf64c1)

#### Sentinel is an experimtent!

It's a hybrid web serderer that combines the power of SPA and SSR strategies in order to achieve the following thinigs in this order:
1. **Security**: Uses Rust as it's memory safe, and keeps all important logic on Server side, only exposing reusable components to the client.
2. **Performance**: Low latency and low resource usage.
3. **Versatility**: From simple to complex enterprise projects, this PoC should prove that this arhitecture offers much flexibility.

## Structure
There are 2 main components:
- `/satellite` - the Backend code, written in Rust, that will run on the Server side
- `/station` - It's the Frontend part, using Web Components with Lit.js

## WIP
I'll add more to this Readme soon.
