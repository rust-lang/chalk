# Unreleased

# Release 0.14.0

# Release 0.13.0

# Release 0.10.0

- Too many changes to list

# Release 0.9.0

- Added the variance parameter

# Releases 0.6.0 .. 0.8.1

Forgot to write release notes =)

# Release 0.5.0

**Tag:** `chalk-engine-v0.5.0`

Pare down to very few dependencies, and make even those optional.

# Release 0.4.0

**Tag:** `chalk-engine-v0.4.0`

Tweak various things about the traits to aid in rustc integration.

# Release 0.2.0

**Tag:** `chalk-engine-v0.2.0`

Remove some pointless traits from Chalk engine context.

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

