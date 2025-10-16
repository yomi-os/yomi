# Contributing to Yomi

Thank you for your interest in contributing to Yomi! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow:

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Prioritize the project's goals and community health

## Getting Started

### Prerequisites

- Rust nightly toolchain
- QEMU for testing
- Basic knowledge of operating system concepts
- Familiarity with Rust's ownership and borrowing

### Setup Your Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/yomi.git
cd yomi

# Run the setup script
./scripts/setup-dev.sh

# Build the project
cargo xtask build

# Run tests
cargo xtask test
```

## Development Workflow

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a feature branch** from `master`
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Make your changes** following our coding standards
5. **Test your changes** thoroughly
6. **Commit your changes** with clear commit messages
7. **Push to your fork** and submit a pull request

## Coding Standards

### Rust Code Style

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `rustfmt` for automatic formatting
- Use `clippy` for linting

```bash
# Format your code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

### Code Organization

- Keep modules focused and cohesive
- Use meaningful names for functions, variables, and types
- Add documentation comments for public APIs
- Prefer small, composable functions over large monolithic ones

### Comments and Documentation

- Write comments in **English** (except for `docs/ja/` directory)
- Use `///` for documentation comments on public items
- Use `//` for implementation comments
- Explain *why*, not *what* (the code shows what)

Example:
```rust
/// Initializes the kernel's memory management system.
///
/// This must be called early in the boot process, before any
/// dynamic memory allocation occurs.
///
/// # Safety
///
/// This function is unsafe because it directly manipulates hardware
/// memory management registers.
pub unsafe fn init_memory() {
    // Initialize page tables before enabling paging
    // to avoid triple faults during the transition
    setup_page_tables();
    enable_paging();
}
```

### Safety and Security

- Minimize use of `unsafe` code
- Document all `unsafe` blocks with safety invariants
- Prefer type safety over runtime checks
- Follow the principle of least privilege

## Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(kernel): add basic interrupt handling

Implement IDT setup and basic interrupt handlers for
exceptions and hardware interrupts.

Closes #123
```

```
fix(vga): prevent buffer overflow in write_string

Add bounds checking to prevent writing beyond the VGA
buffer boundaries.
```

## Pull Request Process

1. **Update documentation** if you're changing APIs or behavior
2. **Add tests** for new functionality
3. **Ensure all tests pass** before submitting
4. **Update the README.md** if needed
5. **Reference related issues** in your PR description

### PR Title Format

Use the same format as commit messages:
```
feat(kernel): add support for virtual memory
```

### PR Description Template

```markdown
## Description
Brief description of what this PR does

## Motivation
Why is this change necessary?

## Changes
- List of changes made
- Another change

## Testing
How was this tested?

## Related Issues
Closes #123
Relates to #456
```

## Testing

### Running Tests

```bash
# Run all tests
cargo xtask test

# Run kernel tests
cd kernel && cargo test

# Run tests for a specific module
cargo test --package yomi-kernel --lib kernel::memory
```

### Writing Tests

- Add unit tests in the same file as the code
- Add integration tests in the `tests/` directory
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_allocation_succeeds() {
        // Test implementation
    }

    #[test]
    #[should_panic(expected = "out of memory")]
    fn test_allocation_fails_when_out_of_memory() {
        // Test implementation
    }
}
```

## Documentation

### Code Documentation

- All public APIs must have documentation comments
- Include examples in documentation where helpful
- Document panics, errors, and safety requirements

### Project Documentation

- Documentation in `docs/ja/` should be in **Japanese**
- All other documentation should be in **English**
- Update relevant documentation when making changes

## Questions?

If you have questions about contributing:

- Open an issue with the `question` label
- Check existing issues and documentation
- Join our community discussions

## Recognition

Contributors will be recognized in:
- The project's README
- Release notes for their contributions
- The project's contributor list

Thank you for contributing to Yomi! 🦀
