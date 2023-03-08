use clap::Parser;
use std::fs;
use std::path::Path;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

   #[arg(short, long)]
   project: String,
}


fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let project_path = args.project;
    let path = Path::new(&project_path);
    
    let dir = fs::read_dir(path)?;

    
    for dir_entry in dir {
        let entry = dir_entry?;
        let file_namne = entry.file_name();
        println!("{:?}", file_namne);
    }

    Ok(())
}
