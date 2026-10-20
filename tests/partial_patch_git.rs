//! `partial_patch` output applies with `git apply --cached`.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use shadcn_egui::partial_patch;

fn git(dir: &Path, args: &[&str], stdin: Option<&str>) -> String {
    use std::io::Write as _;
    let mut cmd = Command::new("git");
    cmd.current_dir(dir).args(args).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped());
    let mut child = cmd.spawn().expect("git runs");
    if let Some(input) = stdin {
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    }
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn stage_and_unstage_selected_lines() {
    if Command::new("git").arg("--version").output().is_err() {
        return; // no git on this machine
    }
    let dir = std::env::temp_dir().join(format!("shadcn_egui_patch_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q"], None);
    let before: String = (1..=30).map(|i| format!("line {i}\n")).collect();
    std::fs::write(dir.join("f.txt"), &before).unwrap();
    git(&dir, &["add", "f.txt"], None);
    git(&dir, &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "init"], None);

    // Two hunks: change line 3, add a line after 25.
    let after = before.replace("line 3\n", "line three\n").replace("line 25\n", "line 25\nline 25.5\n");
    std::fs::write(dir.join("f.txt"), &after).unwrap();
    let diff = git(&dir, &["diff"], None);

    // Stage only the added line of the second hunk.
    let added = diff.lines().position(|l| l == "+line 25.5").unwrap();
    let patch = partial_patch(&diff, &BTreeSet::from([added]), false);
    git(&dir, &["apply", "--cached", "-"], Some(&patch));
    let staged = git(&dir, &["diff", "--cached"], None);
    assert!(staged.contains("+line 25.5") && !staged.contains("line three"), "{staged}");

    // Stage the first hunk too, then unstage just its added line.
    let diff = git(&dir, &["diff"], None);
    let sel: BTreeSet<usize> = diff.lines().enumerate().filter(|(_, l)| l.starts_with(['+', '-']) && !l.starts_with("+++") && !l.starts_with("---")).map(|(i, _)| i).collect();
    git(&dir, &["apply", "--cached", "-"], Some(&partial_patch(&diff, &sel, false)));
    let staged = git(&dir, &["diff", "--cached"], None);
    let three = staged.lines().position(|l| l == "+line three").unwrap();
    git(&dir, &["apply", "--cached", "--reverse", "-"], Some(&partial_patch(&staged, &BTreeSet::from([three]), true)));
    let staged = git(&dir, &["diff", "--cached"], None);
    assert!(staged.contains("-line 3\n") && !staged.contains("+line three"), "{staged}");
    assert!(staged.contains("+line 25.5"), "{staged}");
    let _ = std::fs::remove_dir_all(&dir);
}
