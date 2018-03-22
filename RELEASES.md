The `chalk-engine` and `chalk-macros` crates are published to
crates.io periodically for use by the compiler. The rest of chalk is
not yet published, though it might be nice to publish the interpreter
at some point.

# Release 0.1.0

**Tag:** `chalk-engine-v0.1.0`

Initial release.

# Procedure to cut a release

Should make a script or something, but:

```
> // update version numbers
> cd chalk-macros
> cargo publish
> cd ../chalk-ngine
> cargo publish
> git tag chalk-engine-vXXX
```

