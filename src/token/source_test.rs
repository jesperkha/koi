use super::*;

#[test]
fn test_file_line_offsets() {
    let src = "Doe, a deer\na female deer.\nRay, a drop of golden sun";
    let file = Source::new_from_string(src);
    let expected = vec![0, 12, 27];
    assert_eq!(expected, file.lines);
}

#[test]
fn test_file_line_offsets_2() {
    let src = "\n\n\n\nDoe, \n\na deer\na female deer.\n";
    let file = Source::new_from_string(src);
    let expected = vec![0, 1, 2, 3, 4, 10, 11, 18];
    assert_eq!(expected, file.lines);
}

#[test]
fn test_line_offset_no_newline_or_input() {
    let file1 = Source::new_from_string("Hello");
    assert_eq!(vec![0], file1.lines);

    let file2 = Source::new_from_string("");
    assert_eq!(vec![0], file2.lines);
}
