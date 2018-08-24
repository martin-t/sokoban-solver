use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn run_xsb_pushes() {
    let output = r"Solving levels/custom/02-one-way.txt...
Visited new depth: 0
total created / unique visited / reached duplicates:
1               1                0

Visited new depth: 1
total created / unique visited / reached duplicates:
2               2                0

Visited new depth: 2
total created / unique visited / reached duplicates:
3               3                0

Visited new depth: 3
total created / unique visited / reached duplicates:
4               4                0

States created total: 4
Unique visited total: 4
Reached duplicates total: 0
Created but not reached total: 0

Depth          Created        Unique         Duplicates     Unknown (not reached)
0:             1              1              0              0
1:             1              1              0              0
2:             1              1              0              0
3:             1              1              0              0

Found solution:
###
#.#
# #
# #
#$#
#@#
###

###
#.#
# #
#$#
#@#
# #
###

###
#.#
#$#
#@#
# #
# #
###

###
#*#
#@#
# #
# #
# #
###

UUU
Moves: 3
Pushes: 3
";

    Command::main_binary()
        .unwrap()
        .arg("levels/custom/02-one-way.txt")
        .assert()
        .success()
        .stdout(output)
        .stderr("");
}

#[test]
fn run_custom_moves() {
    let output = r"Solving levels/custom/02-one-way-xsb.txt...
Visited new depth: 0
total created / unique visited / reached duplicates:
1               1                0

Visited new depth: 1
total created / unique visited / reached duplicates:
3               2                0

Visited new depth: 2
total created / unique visited / reached duplicates:
5               3                0

Visited new depth: 3
total created / unique visited / reached duplicates:
7               4                0

States created total: 7
Unique visited total: 4
Reached duplicates total: 0
Created but not reached total: 3

Depth          Created        Unique         Duplicates     Unknown (not reached)
0:             1              1              0              0
1:             2              1              0              1
2:             2              1              0              1
3:             2              1              0              1

Found solution:
<><><><><><><><><>
<>  P   B    _  <>
<><><><><><><><><>

<><><><><><><><><>
<>    P B    _  <>
<><><><><><><><><>

<><><><><><><><><>
<>      P B  _  <>
<><><><><><><><><>

<><><><><><><><><>
<>        P B_  <>
<><><><><><><><><>

rRR
Moves: 3
Pushes: 2
";

    Command::main_binary()
        .unwrap()
        .arg("--move-optimal")
        .arg("--custom")
        .arg("levels/custom/02-one-way-xsb.txt")
        .assert()
        .success()
        .stdout(output)
        .stderr("");
}

#[test]
fn run_bad_formatting_args() {
    // doesn't check stderr - it's not deterministic
    // it sometimes complains about --xsb and sometimees about --custom
    // hopefully should be enough to test that it fails and doesn't print to stdout

    Command::main_binary()
        .unwrap()
        .arg("--custom")
        .arg("--xsb")
        .arg("levels/custom/02-one-way-xsb.txt")
        .assert()
        .failure()
        .stdout("");
}
