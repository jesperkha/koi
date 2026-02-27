# Build and run koi compiler and build test project
run:
    cargo run -- build

# Run tests
test:
    cargo test

# Clean up generated and test files
clean:
    rm -rf \_test target bin ./koi
    rm -f main koi.toml
    rm -f *.koi.h *.a

todo:
    todo -pTODO -dsrc
