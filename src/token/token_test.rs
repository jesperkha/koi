use super::*;

#[test]
fn test_file_line_offsets() {
    let src = "Doe, a deer\na female deer.\nRay, a drop of golden sun";
    let file = File::new_test_file(src);
    let expected = vec![0, 12, 27];
    assert_eq!(expected, file.lines);
}

#[test]
fn test_line_offset_no_newline_or_input() {
    let file1 = File::new_test_file("Hello");
    assert_eq!(vec![0], file1.lines);

    let file2 = File::new_test_file("");
    assert_eq!(vec![0], file2.lines);
}
