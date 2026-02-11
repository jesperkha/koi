# Build and run koi compiler and build test project
run:
    cargo run -- build

# Run tests
test:
    cargo test

# Clean up generated and test files
clean:
    rm -rf \_test target bin
    rm -f main koi.toml

todo:
    todo -pTODO -dsrc
