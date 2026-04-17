# Clean up generated and test files
clean:
    rm -rf \_test target bin ./koi
    rm -f main koi.toml
    rm -f *.koi.h *.a *.o
    rm -f a.out

# Print code todos
todo:
    todo -pTODO -dsrc

# Count lines of code, ignoring test files
cloc:
    cloc src --not-match-f='(_test\.rs$|tests?\.rs$|tests?/|/test_)'

# Build stdlib
std:
    cd lib/std && cargo run build --out=../../koi/lib/std