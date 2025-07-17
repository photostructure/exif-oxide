# Security Policy

## Supported Versions

We provide security updates only for the latest release of exif-oxide. Users are encouraged to keep their installations up to date to receive security fixes and improvements.

## Reporting Security Vulnerabilities

We take security vulnerabilities seriously. If you discover a security vulnerability within exif-oxide, please follow these steps:

### 1. Do Not Disclose Publicly

Please do not disclose the vulnerability publicly until we have had a chance to address it.

### 2. Report Via GitHub Security Advisory

The preferred method for reporting vulnerabilities is through GitHub's private security vulnerability reporting feature:

1. Go to the [Security tab](https://github.com/photostructure/exif-oxide/security) of the exif-oxide repository
2. Click "Report a vulnerability"
3. Fill out the form with as much detail as possible

### 3. Alternative: Email Report

If you are unable to use GitHub's security reporting feature, you can email security reports to:
- Primary: [maintainer email - to be configured]
- Please use PGP encryption if possible (key ID: [to be configured])

### 4. What to Include

Please provide:
- A clear description of the vulnerability
- Steps to reproduce the issue
- Affected versions
- Potential impact
- Any suggested fixes or mitigations

## Response Process

### Timeline

- **Initial Response**: Within 48 hours of receiving the report, we will acknowledge receipt
- **Initial Assessment**: Within 7 days, we will provide an initial assessment of the severity and scope
- **Patch Development**: We will work on developing a fix as quickly as possible, typically within 30 days for critical issues
- **Coordinated Disclosure**: We will coordinate with you on the disclosure timeline

### What We Will Do

1. **Confirm the vulnerability** for the current released version
2. **Develop a fix** and prepare a security patch
3. **Prepare security advisory** documentation
4. **Release patch** for the current released version
5. **Submit advisory** to [RustSec Advisory Database](https://github.com/RustSec/advisory-db)
6. **Update documentation** to reference the advisory

## Security Best Practices for Users

### Dependencies

- Regularly run `cargo audit` to check for known vulnerabilities
- Keep dependencies up to date with `cargo update`
- Consider using `cargo-deny` for more comprehensive dependency management

### Usage Guidelines

- **Input Validation**: Always validate and sanitize EXIF data before using it in security-sensitive contexts
- **Memory Safety**: While exif-oxide is written in Rust, be aware of potential panics when processing malformed files
- **File Sources**: Only process EXIF data from trusted sources when possible
- **Error Handling**: Always handle errors appropriately rather than unwrapping Results

## Security Features

exif-oxide includes several security-focused design decisions:

- **Memory Safety**: Written in Rust with safe code by default
- **Bounds Checking**: All offset calculations are bounds-checked
- **Input Validation**: Validates EXIF structure before processing
- **Limited Recursion**: Prevents stack overflow from maliciously crafted files
- **No Unsafe Code**: Minimizes use of unsafe blocks (currently zero unsafe code)

## Scope

The following are considered security vulnerabilities:

- Memory corruption issues (buffer overflows, use-after-free, etc.)
- Denial of service through resource exhaustion
- Arbitrary code execution
- Information disclosure of sensitive data
- Crashes or panics when processing valid EXIF data

The following are generally NOT considered security vulnerabilities:

- Bugs that only affect incorrect EXIF parsing without security impact
- Performance issues that don't lead to DoS
- Issues in development dependencies
- Missing best practice features (unless they lead to vulnerabilities)

## Recognition

We appreciate security researchers who follow responsible disclosure practices. With your permission, we will acknowledge your contribution in:
- The security advisory
- The project's release notes
- A SECURITY_ACKNOWLEDGMENTS.md file (for significant contributions)

## References

- [Rust Security Working Group](https://www.rust-lang.org/governance/wgs/wg-security-response)
- [RustSec Advisory Database](https://rustsec.org/)
- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)