# Zola Encryption Middleware

## Overview

The Zola encryption middleware allows you to encrypt specific pages at build time, protecting content with password-based or key-based encryption. When a user visits an encrypted page, they see a vault page that prompts for authentication before decrypting and displaying the content client-side in their browser.

### 📝 Important Note: `zola serve` vs `zola build`

**Auto-generated `.key` files behavior:**

- **`zola build`**: `.key` files are written to the `public/` directory on disk alongside encrypted HTML files
- **`zola serve`**: `.key` files are generated in-memory and served via HTTP but **NOT written to disk**
  - Access keys at: `http://localhost:1111/path/to/file.html.key`
  - Example: For `http://localhost:1111/secret/index.html`, get the key at `http://localhost:1111/secret/index.html.key`

**Why?** The `serve` command builds everything in-memory for live reloading and doesn't write outputs to the `public/` directory. All generated files (including `.key` files) are served via HTTP from memory.

### Use Cases

- Private blog posts or journal entries
- Internal documentation not ready for public consumption
- Member-only content without a backend
- Draft content shared with specific reviewers
- Personal notes within a public site

### ⚠️ Important Security Notice

**This is client-side encryption, not server-side access control.**

The encrypted content is publicly downloadable in its encrypted form. This means:

- ❌ **NOT suitable for**: Highly sensitive data, PII, passwords, API keys, or anything requiring strong security
- ✅ **Suitable for**: "Soft" protection against casual browsing, unlisted content, draft previews, or content requiring simple access control

**Key Limitations:**
- Unlimited brute-force attempts possible (no rate limiting)
- Once password is shared, access cannot be revoked without rebuilding
- Weak passwords are vulnerable to dictionary attacks
- Encrypted content is not protected from determined attackers
- Requires JavaScript enabled in user's browser

## Configuration

### Basic Configuration

Add encryption rules to your `config.toml`:

```toml
[[encrypt]]
# Glob patterns for output paths to encrypt
paths = ["secret/**", "private/*.html"]

# Password from environment variable (recommended for production)
password_env = "MY_SECRET"

# Optional: Argon2 parameters (defaults shown)
argon2_memory = 65536      # KB (64 MB)
argon2_iterations = 3       # Time cost
argon2_parallelism = 1      # Parallelism
```

### Multiple Encryption Rules

You can define multiple rules with different authentication methods:

```toml
# Rule 1: Password-based encryption for blog posts
[[encrypt]]
paths = ["blog/private/**"]
password_env = "BLOG_PASSWORD"
argon2_memory = 65536
argon2_iterations = 3

# Rule 2: Key-based encryption for admin pages
[[encrypt]]
paths = ["admin/**"]
key_env = "ADMIN_KEY"

# Rule 3: Auto-generated keys for shareable content
[[encrypt]]
paths = ["share/**"]
# No auth method - auto-generates unique key per file

# Rule 4: Development only (NOT RECOMMENDED for production)
[[encrypt]]
paths = ["drafts/**"]
password = "dev-only-password"  # Plaintext password - insecure!
```

### Configuration Options

#### `paths` (required)

Array of glob patterns matching **output paths** (not source content paths).

**Examples:**
```toml
paths = ["secret/index.html"]           # Exact match
paths = ["secret/*"]                    # All direct children
paths = ["secret/**"]                   # All descendants (recursive)
paths = ["*.html"]                      # All HTML files
paths = ["blog/private/**/*.html"]      # Specific file types in subdirectory
```

**Important:** Patterns match against the **generated output path**, not the source markdown path.

For example, if your content is at `content/blog/secret-post.md` and it generates to `public/blog/secret-post/index.html`, the pattern should match `blog/secret-post/index.html` or `blog/secret-post/*`.

#### Authentication Options (optional)

You can specify zero or one authentication method per rule.

##### No authentication (auto-generated key)

If no authentication method is specified, Zola will automatically generate a unique random 256-bit key for each encrypted file and write it to a `.key` file alongside the encrypted HTML.

```toml
[[encrypt]]
paths = ["share/**"]
# No password_env, password, or key_env specified
```

**Use cases:**
- Shareable protected content with separate key distribution
- No password management needed
- Each file is independently encrypted with its own key
- Keys can be distributed via different channels than the HTML

**Behavior:**
- ℹ️ Build outputs: "Encryption rule #N: No auth method specified. A random key will be auto-generated..."
- Each encrypted file gets a unique key
- Key is written to `<filename>.key` (e.g., `index.html.key` for `index.html`)
- User must provide the key from the `.key` file to decrypt

**Example:**
```bash
# After building
ls public/share/
# index.html       (encrypted vault page)
# index.html.key   (base64-encoded decryption key)

cat public/share/index.html.key
# SGVsbG8gV29ybGQhIFRoaXMgaXMgYSBrZXk=
```

**Sharing:**
- Share the HTML file publicly (encrypted)
- Distribute the `.key` file content separately (email, chat, etc.)
- User enters key from `.key` file into vault prompt

##### `password_env` (recommended for user-facing content)

Environment variable name containing the password.

```toml
password_env = "MY_SECRET"
```

At build time, Zola reads from `ZOLA_ENCRYPTION_PASS_MY_SECRET`:

```bash
export ZOLA_ENCRYPTION_PASS_MY_SECRET="correct-horse-battery-staple"
zola build
```

**Best Practices:**
- Use strong, unique passwords (20+ characters)
- Use a password manager
- Never commit passwords to git
- Rotate passwords periodically

##### `password` (development only)

Plaintext password in config file.

```toml
password = "dev-password"
```

**⚠️ WARNING:** This will trigger a build warning. Only use for local development. Never commit plaintext passwords to version control.

##### `key_env` (advanced)

Environment variable containing a raw 256-bit encryption key in hex or base64 format.

```toml
key_env = "ADMIN_KEY"
```

Generate a key:
```bash
# Generate 256-bit key (hex format)
openssl rand -hex 32

# Or base64 format
openssl rand -base64 32
```

Set the environment variable:
```bash
# Hex (64 characters)
export ZOLA_ENCRYPTION_PASS_ADMIN_KEY="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

# Or base64 (44 characters)
export ZOLA_ENCRYPTION_PASS_ADMIN_KEY="abcdefghijklmnopqrstuvwxyz0123456789ABCD=="
```

**When to use different modes:**
- **Auto-generated key**: Separate key distribution, no password management, machine-readable keys
- **Password mode**: User-friendly, easier to remember and share, proper authentication
- **Raw key mode**: Pre-generated keys, maximum control over key management, machine-to-machine access

#### Argon2 Parameters (optional, password mode only)

These parameters control the password hashing strength. Higher values = more secure but slower decryption.

##### `argon2_memory` (default: 65536)

Memory cost in KB. Controls RAM usage during key derivation.

```toml
argon2_memory = 65536  # 64 MB (recommended minimum)
argon2_memory = 131072 # 128 MB (stronger)
```

**Trade-off:** Higher memory = harder to attack, but slower in-browser decryption (~1-2 seconds).

##### `argon2_iterations` (default: 3)

Time cost / number of iterations.

```toml
argon2_iterations = 3  # Standard
argon2_iterations = 5  # More secure
```

##### `argon2_parallelism` (default: 1)

Parallelism factor. Keep at 1 for browser compatibility.

```toml
argon2_parallelism = 1  # Recommended
```

## Usage

### Building

Set required environment variables:

```bash
export ZOLA_ENCRYPTION_PASS_MY_SECRET="your-password-here"
zola build
```

### Deployment

The vault page works entirely client-side, so deploy normally:

```bash
# After building with encryption
zola build
rsync -avz public/ user@server:/var/www/site/
```

**Important:** The vault page includes all decryption logic. No server-side configuration needed.

### Accessing Encrypted Pages

#### Password Mode:
1. User navigates to encrypted page URL
2. Vault page displays with password prompt
3. User enters password
4. JavaScript derives key from password + salt, then decrypts content
5. Original unencrypted HTML is displayed

#### Raw/Auto-generated Key Mode:
1. User navigates to encrypted page URL
2. Vault page displays with key prompt
3. User enters 256-bit key (from `.key` file or pre-shared)
4. JavaScript decrypts content directly with provided key
5. Original unencrypted HTML is displayed

## Encryption Format

### Blob Structure

Zola uses two different blob formats depending on the authentication method:

#### Version 1: Password-based

```
[version:1][salt:16 bytes][nonce:12 bytes][ciphertext+tag:N bytes]
└─ Base64 encoded and embedded in vault.html
```

**Components:**
- **Version (1 byte)**: `0x01` for password-based encryption
- **Salt (16 bytes)**: Random salt for Argon2 key derivation
- **Nonce (12 bytes)**: Random nonce for AES-GCM (unique per page)
- **Ciphertext+tag**: Encrypted HTML content with GCM authentication tag

#### Version 2: Key-based (raw or auto-generated)

```
[version:1][nonce:12 bytes][ciphertext+tag:N bytes]
└─ Base64 encoded and embedded in vault.html
```

**Components:**
- **Version (1 byte)**: `0x02` for key-based encryption
- **Nonce (12 bytes)**: Random nonce for AES-GCM (unique per page)
- **Ciphertext+tag**: Encrypted HTML content with GCM authentication tag
- **Note**: Key is NOT included in blob - user must provide it (from `.key` file or pre-shared)

### Cryptographic Details

**Encryption Algorithm:** AES-256-GCM
- AEAD (Authenticated Encryption with Associated Data)
- 256-bit key
- 96-bit nonce (randomly generated)
- 128-bit authentication tag
- Prevents both unauthorized access AND tampering

**Key Derivation (password mode):** Argon2id
- Memory-hard (resistant to GPU/ASIC attacks)
- Configurable parameters (memory, iterations, parallelism)
- Random salt per encryption
- Version 0x13
- Same algorithm used server-side (build time) and client-side (browser)

**Password Mode Flow (Version 1):**
```
Password → Argon2id(password, salt, params) → 256-bit key → AES-GCM encryption
```

**Key-based Mode Flow (Version 2):**
```
# Auto-generated:
Random 256-bit key → AES-GCM encryption → Write key to .key file

# Raw key (pre-shared):
User-provided raw key (256-bit) → AES-GCM encryption
```

## Theming

The vault page respects Zola's theme system and can be customized.

### Creating a Custom Vault Template

Create `templates/vault.html` in your site or theme:

```html
{% extends "__zola_builtins/vault.html" %}

{% block title %}🔐 Private Content | {{ config.title }}{% endblock %}
```

Or completely override:

```html
<!doctype html>
<html lang="{{ lang }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="robots" content="noindex, nofollow">
    <title>Protected Content</title>
    <!-- Your custom styles -->
</head>
<body>
    <!-- Your custom vault UI -->
    <script id="encrypted-payload" type="application/octet-stream">
        {{ encrypted_data | safe }}
    </script>

    <!-- Include decryption logic -->
    {% if uses_password %}
        <!-- Argon2 WASM library for password mode -->
    {% endif %}

    <!-- AES-GCM decryption script -->
    <!-- Prompts for password or key depending on mode -->
</body>
</html>
```

### CSS Variables

The default vault template provides CSS variables for theming:

```css
:root {
    --vault-bg: #f5f5f5;
    --vault-fg: #333;
    --vault-primary: #4a90e2;
    --vault-primary-hover: #357abd;
    --vault-border: #ddd;
    --vault-error: #d32f2f;
    --vault-success: #388e3c;
    --vault-container-bg: #fff;
    --vault-shadow: rgba(0, 0, 0, 0.1);
}
```

Override in your theme's CSS:

```css
/* themes/mytheme/sass/vault.scss */
:root {
    --vault-primary: #ff6b6b;
    --vault-primary-hover: #ee5a52;
}
```

## SEO Considerations

### Automatic Sitemap Exclusion

Encrypted pages are automatically excluded from `sitemap.xml` to prevent search engine indexing of inaccessible content.

**Behavior:**
- Encrypted pages: ❌ Not in sitemap
- Normal pages: ✅ Included in sitemap
- Section with encrypted pages: ✅ Section included, encrypted children excluded

### Meta Tags

The vault template includes:
```html
<meta name="robots" content="noindex, nofollow">
```

This tells search engines not to index vault pages.

### Recommendations

- **URLs**: Use non-obvious paths for encrypted content (e.g., `/p/a7b3c9d2` instead of `/secret-plans`)
- **Links**: Avoid linking to encrypted pages from public pages
- **Analytics**: Encrypted pages will show in analytics as vault page loads, not final decrypted content

## Troubleshooting

### Build Errors

#### "Environment variable 'ZOLA_ENCRYPTION_PASS_X' is not set"

**Cause:** Missing environment variable for password or key.

**Solution:** Set the environment variable before building:
```bash
export ZOLA_ENCRYPTION_PASS_X="your-password"
zola build
```

#### "Encryption rule #N must specify only ONE of: password_env, password, or key_env"

**Cause:** Multiple authentication methods specified in same rule.

**Solution:** Use only one (or none) per `[[encrypt]]` block:
```toml
# ✅ Good - Password mode
[[encrypt]]
paths = ["secret/**"]
password_env = "SECRET"

# ✅ Good - Auto-generated key mode
[[encrypt]]
paths = ["share/**"]
# No auth method specified

# ❌ Bad - Multiple methods
[[encrypt]]
paths = ["secret/**"]
password_env = "SECRET"
key_env = "KEY"  # Remove this
```

#### "Invalid glob pattern"

**Cause:** Malformed glob pattern in `paths`.

**Solution:** Check glob syntax:
```toml
paths = ["secret/**"]      # ✅ Recursive
paths = ["secret/*"]       # ✅ Direct children
paths = ["secret/**.html"] # ❌ Invalid
```

### Runtime Errors

#### "Decryption failed. Incorrect password or key."

**Causes:**
- Wrong password/key entered
- Page was re-encrypted with different password
- Corrupted encrypted data

**Solution:**
- Verify correct password/key
- Rebuild site if password changed
- Check browser console for detailed errors

#### Vault page shows but decryption script doesn't run

**Causes:**
- JavaScript disabled in browser
- Browser doesn't support Web Crypto API
- CSP (Content Security Policy) blocking inline scripts

**Solutions:**
- Enable JavaScript
- Use modern browser (Chrome 37+, Firefox 34+, Safari 11+)
- Check CSP headers if self-hosting

#### "This browser does not support encryption features"

**Cause:** Very old browser without Web Crypto API.

**Solution:** Use a modern browser or inform users of minimum requirements.

### Performance Issues

#### Slow decryption (5+ seconds)

**Cause:** High Argon2 parameters on slow device.

**Solutions:**
- Reduce `argon2_memory` (e.g., to 32768 for 32 MB)
- Reduce `argon2_iterations` (e.g., to 2)
- Use key mode instead of password mode

**Trade-off:** Lower parameters = faster decryption but easier to brute-force.

## Security Best Practices

### Password Management

✅ **Do:**
- Use long passwords (20+ characters)
- Use unique passwords per encryption rule
- Store passwords in password manager
- Use environment variables in CI/CD
- Rotate passwords periodically
- Consider using passphrases (e.g., "correct-horse-battery-staple")

❌ **Don't:**
- Hardcode passwords in `config.toml`
- Reuse passwords across sites
- Share passwords in plain text
- Commit `.env` files to git
- Use short or dictionary words

## Implementation Details

### Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Zola Build Process                 │
├─────────────────────────────────────────────────────┤
│  1. Parse config.toml                               │
│  2. Validate encryption rules                       │
│  3. Resolve environment variables                   │
│  4. Render pages                                    │
│  5. Middleware Pipeline:                            │
│     ┌─────────────────────────────────────┐        │
│     │ LiveReloadMiddleware (if serve)     │        │
│     │          ↓                           │        │
│     │ MinifyMiddleware (if enabled)       │        │
│     │          ↓                           │        │
│     │ EncryptionMiddleware (if enabled) ←─┼────────┼─ Matches paths
│     │          ↓                           │        │  Encrypts HTML
│     │ CompressionMiddleware (if enabled)  │        │  Renders vault
│     └─────────────────────────────────────┘        │
│  6. Write outputs to disk                          │
│  7. Generate sitemap (excluding encrypted)         │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│              Browser Decryption Flow                │
├─────────────────────────────────────────────────────┤
│  1. User navigates to encrypted page                │
│  2. vault.html loads (with encrypted blob)          │
│  3. Check blob version:                             │
│     ┌─────────────────────────────────────────┐    │
│     │ Version 1 (Password):                   │    │
│     │  └─ Prompt for password                 │    │
│     │  └─ Derive key with Argon2 + salt       │    │
│     ├─────────────────────────────────────────┤    │
│     │ Version 2 (Key-based):                  │    │
│     │  └─ Prompt for key                      │    │
│     │  └─ Parse hex/base64 to raw bytes       │    │
│     └─────────────────────────────────────────┘    │
│  4. Decrypt with AES-GCM using Web Crypto API       │
│  5. Replace document with decrypted HTML            │
│  6. Page functions normally                         │
└─────────────────────────────────────────────────────┘
```

### File Structure

```
components/
├── config/src/config/mod.rs
│   └── EncryptionRule struct, validation, path matching
├── site/src/
│   ├── middleware/encrypt.rs
│   │   └── EncryptionMiddleware (encryption logic)
│   ├── middleware.rs
│   │   └── Public exports
│   ├── lib.rs
│   │   └── Pipeline integration
│   └── sitemap.rs
│       └── Encrypted path exclusion
└── templates/src/
    ├── builtins/vault.html
    │   └── Vault UI + decryption script
    └── lib.rs
        └── Template registration
```

### Dependencies

```toml
# Added to components/site/Cargo.toml
aes-gcm = "0.10"   # AES-256-GCM encryption
argon2 = "0.5"     # Password hashing
base64 = "0.22"    # Base64 encoding
rand = "0.8"       # Random number generation
hex = "0.4"        # Hex encoding/decoding
```

### Browser Requirements

**Minimum versions:**
- Chrome/Edge 37+
- Firefox 34+
- Safari 11+
- Opera 24+

**Required APIs:**
- Web Crypto API (`crypto.subtle`)
- TextEncoder/TextDecoder
- ES6 features (arrow functions, const/let, async/await)
- Base64 decoding (`atob`)
- WebAssembly (for Argon2 in password mode)

**Network Requirements (Password Mode Only):**
- Internet connection to load Argon2 WASM from jsDelivr CDN
- CDN URL: `https://cdn.jsdelivr.net/npm/argon2-browser@1.18.0/dist/argon2-bundled.min.js`
- SRI Hash: `sha384-XOR3aNvHciLPIf6r+2glkrmbBbLmIJ1EChMXjw8eBKBf8gE0rDq1TyUNuRdorOqi`
- Alternative: Use key mode for offline/air-gapped environments

## Known Issues & Limitations

### ⚠️ Current Limitations

1. **External CDN Dependency (Password Mode Only)**
   - Vault template loads Argon2 WASM from jsDelivr CDN
   - Requires internet connection for password-based decryption
   - Key mode has no external dependencies
   - **Workaround:** Use key mode for offline/air-gapped environments

2. **No Server-Side Protection**
   - Encrypted blobs are publicly downloadable
   - No rate limiting on decryption attempts
   - Cannot revoke access without rebuild

3. **Path Matching Limitations**
   - Simplified glob matching for sitemap exclusion
   - May not match all edge cases
   - Complex nested patterns may not work correctly

4. **No Encrypted Asset Support**
   - Only HTML pages are encrypted
   - Images, PDFs, downloads remain unencrypted
   - Workaround: Base64 embed small assets in encrypted pages

5. **Single Page Only**
   - Each encrypted page is independent
   - No shared session across encrypted pages
   - User must enter password for each page

### Future Improvements

**Planned:**
- [x] Proper Argon2 WASM implementation ✅ **COMPLETED**
- [ ] Comprehensive test suite
- [ ] Encrypted asset support
- [ ] Session persistence across pages
- [ ] Time-based expiration
- [ ] Password strength meter in vault UI
- [ ] Audit logging

**Under Consideration:**
- [ ] Shared secrets across multiple pages
- [ ] Encrypted RSS/Atom feeds
- [ ] Two-factor authentication
- [ ] Integration with external auth providers

## FAQ

### Q: Can I change the password for already-encrypted pages?

**A:** Yes, but you must rebuild the site. The password is used during the build process to encrypt the content. To change:

1. Update environment variable with new password
2. Rebuild: `zola build`
3. Deploy updated site

Old passwords will no longer work on new builds.

### Q: Can users bookmark decrypted pages?

**A:** No. Decryption happens client-side and doesn't change the URL. Bookmarks will always point to the vault page.

### Q: Does encryption work with `zola serve`?

**A:** Yes! The encryption middleware runs during `zola serve` just like during `zola build`. Set environment variables before running:

```bash
export ZOLA_ENCRYPTION_PASS_SECRET="password"
zola serve
```

### Q: How do I share encrypted pages?

**A:** It depends on the encryption mode:

**Password Mode:**
Share two things:
1. The page URL (e.g., `https://example.com/secret/`)
2. The password (via secure channel - Signal, password manager, etc.)

⚠️ Never share passwords via email or public channels.

**Raw/Auto-generated Key Mode:**
Share two things:
1. The page URL (e.g., `https://example.com/secret/`)
2. The key content (from `.key` file or pre-shared key)

For auto-generated keys:
```bash
# Share the key file content
cat public/secret/index.html.key
# SGVsbG8gV29ybGQhIFRoaXMgaXMgYSBrZXk=

# Or deploy .key files alongside HTML for programmatic access
```

### Q: Can search engines access encrypted content?

**A:** No. Encrypted pages are:
- Excluded from `sitemap.xml`
- Tagged with `<meta name="robots" content="noindex, nofollow">`
- Not comprehensible to crawlers (encrypted blob)

### Q: What happens if JavaScript is disabled?

**A:** The vault page displays, but decryption cannot occur. The user will see the password prompt but entering credentials will not work. There is no graceful degradation.

### Q: Can I encrypt the 404 page?

**A:** No, this would create a poor user experience. The 404 page should remain accessible to inform users of broken links.

### Q: How large can encrypted pages be?

**A:** No hard limit, but consider:
- Larger pages = longer decryption time
- Entire page loaded into memory
- Recommended max: ~1 MB of HTML

For very large content, consider splitting into multiple pages.

### Q: Can I use multiple passwords for the same page?

**A:** No. Each page can only be encrypted with one password/key. However, you can create multiple encrypted copies with different passwords using different output paths.

## Support

### Reporting Issues

If you encounter bugs or have feature requests:

1. Check existing issues: https://github.com/getzola/zola/issues
2. Provide minimal reproduction case
3. Include Zola version, OS, and config snippet
4. Share error messages (redact sensitive info)

### Contributing

Contributions welcome! Areas needing help:
- Argon2 WASM implementation
- Test coverage
- Documentation improvements
- Performance optimizations
- Browser compatibility testing

### License

This feature is part of Zola and follows the same license (MIT).

---

**Version:** 1.0.0
**Last Updated:** 2025-01-09
**Compatibility:** Zola 0.21.0+
