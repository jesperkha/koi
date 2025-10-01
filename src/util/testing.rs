use crate::error::ErrorSet;

pub fn compare_string_lines_or_panic(ina: String, inb: String) {
    let a: Vec<&str> = ina.trim().split('\n').collect();
    let b: Vec<&str> = inb.trim().split('\n').collect();
    assert_eq!(a.len(), b.len(), "number of lines must be equal");

    for (i, line) in a.iter().enumerate() {
        assert_eq!(line.trim(), b.get(i).unwrap().trim());
    }
}

pub fn must<T>(res: Result<T, ErrorSet>) -> T {
    res.unwrap_or_else(|err| panic!("unexpected error: {}", err))
}
