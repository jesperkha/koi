use crate::util::new_source;

#[test]
fn test_file_line_offsets() {
    let src = "Doe, a deer\na female deer.\nRay, a drop of golden sun";
    let file = new_source(src);
    let expected = vec![0, 12, 27];
    assert_eq!(expected, file.lines);
}

#[test]
fn test_file_line_offsets_2() {
    let src = "\n\n\n\nDoe, \n\na deer\na female deer.\n";
    let file = new_source(src);
    let expected = vec![0, 1, 2, 3, 4, 10, 11, 18];
    assert_eq!(expected, file.lines);
}

#[test]
fn test_line_offset_no_newline_or_input() {
    let file1 = new_source("Hello");
    assert_eq!(vec![0], file1.lines);

    let file2 = new_source("");
    assert_eq!(vec![0], file2.lines);
}
