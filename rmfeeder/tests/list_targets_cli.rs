use std::process::Command;

#[test]
fn list_targets_outputs_stable_csv() {
    let output = Command::new(env!("CARGO_BIN_EXE_rmfeeder"))
        .arg("--list-targets")
        .output()
        .expect("run rmfeeder --list-targets");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");

    assert!(stdout.starts_with("flag,width,height,description\n"));
    assert!(stdout.contains("letter,2550,3300,US Letter\n"));
    assert!(stdout.contains("rm1,1404,1872,reMarkable 1\n"));
    assert!(stdout.contains("rmpp-move,1620,2160,reMarkable Paper Pro Move\n"));
    assert!(stdout.contains("supernote-a5x,1920,2560,Supernote A5X\n"));
    assert!(stdout.contains("boox-noteair4c-color,930,1240,Boox Note Air4 C Color Layer\n"));
    assert!(stdout.contains("ipad13,2064,2752,iPad Pro 13-inch\n"));

    let line_count = stdout.lines().count();
    assert_eq!(line_count, 21, "header + 20 targets expected");
}
