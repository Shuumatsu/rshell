use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn main() {
    loop {
        print!("> ");
        // need to explicitly flush this to ensure it prints before read_line
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // must be peekable so we know when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_proc = None;

        while let Some(command) = commands.next() {
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_proc = None;
                }
                "exit" => return,
                command => {
                    let stdin = previous_proc.map_or(Stdio::inherit(), |proc: Child| {
                        Stdio::from(proc.stdout.unwrap())
                    });

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let proc = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match proc {
                        Ok(output) => {
                            previous_proc = Some(output);
                        }
                        Err(e) => {
                            previous_proc = None;
                            eprintln!("{}", e);
                        }
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_proc {
            // block until the final command has finished
            final_command.wait().unwrap();
        }
    }
}
