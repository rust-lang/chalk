# Unreleased

# Release 0.52.0

# Release 0.51.0

# Release 0.50.0

# Release 0.49.0

# Release 0.48.0

# Release 0.47.0

# Release 0.46.0

# Release 0.45.0

# Release 0.43.0

# Release 0.41.0

# Release 0.40.0

# Release 0.39.0

# Release 0.37.0

# Release 0.36.0

# Release 0.35.0

# Release 0.34.0

# Release 0.33.0

# Release 0.32.0

# Release 0.31.0

# Release 0.30.0

# Release 0.29.0

# Release 0.28.0

# Release 0.27.0

# Release 0.26.0

# Release 0.25.0

# Release 0.24.0

# Release 0.23.0

# Release 0.22.0

# Release 0.21.0

# Release 0.20.0

# Release 0.19.0

# Release 0.18.0

# Release 0.17.0

# Release 0.16.0

# Release 0.15.0

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

