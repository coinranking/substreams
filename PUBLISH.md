# Publishing DEX Packages to Substreams Registry

This guide explains how to build, test, and publish DEX packages to the [Substreams.dev](https://substreams.dev) registry.

## Prerequisites

### Required Tools
- **Rust toolchain** with `wasm32-unknown-unknown` target
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- **Substreams CLI** v1.1.0 or higher
  ```bash
  # macOS
  brew install streamingfast/tap/substreams
  
  # Linux/Other
  # Download from https://github.com/streamingfast/substreams/releases
  ```

### Authentication
Before publishing, you need to authenticate with the Substreams registry:
```bash
substreams auth
```
This will open your browser for GitHub authentication. The token is stored locally for future use.

## Building a DEX Package

### 1. Navigate to the DEX Package
```bash
cd dexes/uniswap-v3-forks  # or your specific DEX implementation
```

### 2. Update Version Number
Edit `substreams.yaml` to bump the version:
```yaml
package:
  name: coinranking_uniswap_v3_forks
  version: v0.2.0  # Increment according to semver
```

**Version Guidelines:**
- **Patch** (v0.1.0 → v0.1.1): Bug fixes, documentation updates
- **Minor** (v0.1.0 → v0.2.0): New features, backwards-compatible changes
- **Major** (v0.1.0 → v1.0.0): Breaking changes to output format

### 3. Build the WASM Module
```bash
cargo build --target wasm32-unknown-unknown --release
```

This creates the WASM binary at:
```
../../target/wasm32-unknown-unknown/release/coinranking_uniswap_v3_forks.wasm
```

## Testing Before Publishing

### 1. Run Local Tests
```bash
# Use the test script (if available)
./test.sh

# Or test manually with a small block range
substreams run substreams.yaml map_v3_ticker_output \
  --start-block 12369621 \
  --stop-block +10
```

### 2. Verify Output Format
Ensure the output matches the expected `TickerOutput` structure:
```json
{
  "tickers": [
    {
      "poolAddress": "0x...",
      "blockVolumeToken0": "1000000",
      "blockVolumeToken1": "2000000",
      "swapCount": 5,
      "closePrice": "1.234",
      "blockNumber": 12369621,
      "timestamp": 1620000000
    }
  ]
}
```

### 3. Test on Different Networks (if applicable)
For universal packages like uniswap-v3-forks:
```bash
# Test on Polygon
ENDPOINT=polygon.streamingfast.io:443 ./test.sh -s 22757547 -e +10

# Test on Arbitrum
ENDPOINT=arb-one.streamingfast.io:443 ./test.sh -s 1107 -e +10
```

## Creating the Package

### 1. Pack the Substreams
```bash
substreams pack
```

This creates a `.spkg` file containing:
- Compiled WASM module
- Protobuf definitions
- Substreams manifest
- All dependencies

The output file will be named:
```
coinranking-uniswap-v3-forks-v0.2.0.spkg
```

### 2. Verify the Package
```bash
# Inspect package contents
substreams info coinranking-uniswap-v3-forks-v0.2.0.spkg

# Test the package locally
substreams run coinranking-uniswap-v3-forks-v0.2.0.spkg \
  map_v3_ticker_output \
  --start-block 12369621 \
  --stop-block +10
```

## Publishing to the Registry

### 1. Publish the Package
```bash
substreams publish coinranking-uniswap-v3-forks-v0.2.0.spkg
```

Or publish directly without creating a `.spkg` file first:
```bash
substreams publish substreams.yaml
```

### 2. Publishing from GitHub Release (Alternative)
You can also publish from a GitHub release URL:
```bash
substreams publish https://github.com/coinranking/substreams/releases/download/v0.2.0/coinranking-uniswap-v3-forks-v0.2.0.spkg
```

### 3. Verify Publication
After publishing, verify your package on the registry:
- Visit https://substreams.dev
- Search for your package name
- Check that the version and metadata are correct

## Post-Publication

### 1. Tag the Release
Create a git tag for the published version:
```bash
git tag dexes/uniswap-v3-forks/v0.2.0
git push origin dexes/uniswap-v3-forks/v0.2.0
```

### 2. Update Documentation
- Update README with the new version number
- Add release notes if significant changes were made
- Update any integration examples

### 3. Announce the Release (Optional)
- Create a GitHub release with notes
- Notify users of breaking changes (if any)

## Best Practices

### DO:
- ✅ Test thoroughly before publishing
- ✅ Use semantic versioning consistently
- ✅ Include clear descriptions in `substreams.yaml`
- ✅ Test on multiple networks (for universal packages)
- ✅ Keep the output format consistent with `TickerOutput`
- ✅ Document any limitations or requirements

### DON'T:
- ❌ Publish untested changes
- ❌ Break the output format without a major version bump
- ❌ Forget to update the version number
- ❌ Publish packages with compilation warnings
- ❌ Include sensitive information in the package

## Troubleshooting

### Authentication Issues
```bash
# Re-authenticate if token expires
substreams auth

# Check current authentication status
substreams registry list  # Should work if authenticated
```

### Build Failures
```bash
# Clean build
cargo clean
cargo build --target wasm32-unknown-unknown --release

# Check for missing dependencies
cargo check
```

### Package Too Large
If your `.spkg` exceeds size limits:
1. Ensure you're building in release mode
2. Check for unnecessary dependencies
3. Remove debug symbols if present

## Example: Publishing uniswap-v3-forks

Complete workflow for publishing the uniswap-v3-forks package:

```bash
# 1. Navigate to package
cd dexes/uniswap-v3-forks

# 2. Update version in substreams.yaml
# Edit: version: v0.2.0

# 3. Build
cargo build --target wasm32-unknown-unknown --release

# 4. Test
./test.sh -s 12369621 -e +100 -f

# 5. Pack
substreams pack

# 6. Publish
substreams publish coinranking-uniswap-v3-forks-v0.2.0.spkg

# 7. Tag
git tag dexes/uniswap-v3-forks/v0.2.0
git push origin dexes/uniswap-v3-forks/v0.2.0
```

## Registry Notes

- Packages are **immutable** once published - you cannot overwrite a version
- The registry is public - all published packages are visible to everyone
- Package names should follow the format: `coinranking_[dex_name]`
- Descriptions should clearly state supported networks and DEX types

## Support

For issues with:
- **Building/Testing**: Check the README in the specific DEX folder
- **Publishing**: See [Substreams documentation](https://substreams.streamingfast.io/)
- **Registry**: Visit [substreams.dev](https://substreams.dev) or open an issue