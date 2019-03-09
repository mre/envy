use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "envy", about = "context-based environment variables")]
pub struct Envy {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(name = "export")]
    Export { shell: String },
    #[structopt(name = "hook")]
    Hook { shell: String },
}
