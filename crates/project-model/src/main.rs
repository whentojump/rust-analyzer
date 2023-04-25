use std::{
    process::Command,
    collections::HashMap
};

fn main() {
    //
    // `Command` basics
    //

    // cd /etc && ls -l -h apt
    let mut cmd = Command::new("ls");
    cmd.current_dir("/etc")
       .arg("-l")
       .args(["-h", "apt"]);
    cmd.spawn().expect("oops");

    // VAR1=123 VAR2=456 VAR3=789 bash -c 'echo $VAR1 $VAR2 $VAR3'
    let mut envs = HashMap::new();
    envs.insert("VAR1".to_string(), "123".to_string());
    envs.insert("VAR2".to_string(), "456".to_string());
    let mut cmd = Command::new("bash");
    cmd.envs(&envs)
       .env("VAR3", "789")
       .args(["-c", "echo $VAR1 $VAR2 $VAR3"]);
    cmd.spawn().expect("oops");
}
