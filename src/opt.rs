use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "envy", about = "context-based environment variables")]
pub struct Envy {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    /// Export environment variables based on the current directory
    #[structopt(name = "export")]
    Export { shell: String },
    /// Print the hook to activate envy for your shell
    #[structopt(name = "hook")]
    Hook { shell: String },
}
