// Independent validation-only executable. Only opens the explicitly supplied
// private synthetic state directory; no runtime, network or credentials.
fn main() {
    let path=std::env::args_os().nth(1).expect("private state path");
    match drive_core::Store::open(std::path::PathBuf::from(path)) {
        Ok(_store)=>println!("OPEN_OK schema={}",drive_core::STATE_SCHEMA_VERSION),
        Err(error)=>{eprintln!("OPEN_REJECTED {error}");std::process::exit(23);}
    }
}
