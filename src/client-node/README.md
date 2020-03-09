# Fluvio Client for Node.js

Node.js binding for Fluvio streaming platform.

## Installation on NPM

Fluvio client is native module.  It is written using rust.  

### Install Rust tooling

First, install rust tooling by following [rustup](https://rustup.rs).
```
<follow instruction on rustup>
```

Then, you must enable nightly toolchain:
```
rustup toolchain install nightly
```

And enable globally
```
rustup default nightly
```

### Install NPM package

```
npm install @fluvio/client
```

### Example usage

```
var flvClient = require('@fluvio/client');

```

Please see test.js in the node_modules for example.


# Development Build and Test

To compile rust library:

```
make
```

To test development module:

```
node test.js
```

# Build and Link

Compile and publish for local node project (npm link):

```
make build-local
```

## License

This project is licensed under the [Apache license](LICENSE-APACHE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Fluvio by you, shall be licensed as Apache, without any additional
terms or conditions.
