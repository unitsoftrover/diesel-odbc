extern crate dunce;
use std::{env, path::PathBuf, path::Path};

fn main() {
    
    let mut dir = env::var_os("CARGO_MANIFEST_DIR");
    if let Some(d) = dir{
        eprintln!("CARGO_MANIFEST_DIR: {}", d.to_str().unwrap());
        dir = Some(d);
    }else{
        let current_path = std::env::current_dir().unwrap();
        dir = Some(current_path.as_os_str().to_os_string());
        eprintln!("CARGO_MANIFEST_DIR not setup use current dir:{}", current_path.display());
    }

    let root = PathBuf::from(dir.unwrap());

    // let library_name = "cfun";
    // let library_dir = dunce::canonicalize(root.join("src")).unwrap();
    // println!("cargo:rustc-link-lib=static={}", library_name);
    // println!("cargo:rustc-link-search=native={}", env::join_paths(&[library_dir]).unwrap().to_str().unwrap());

    let library_dir_diesel = dunce::canonicalize(root.join("../../../../diesel/target/debug/")).unwrap();
    // let library_dir_diesel_odbc = dunce::canonicalize(root.join("../diesel-odbc/target/debug/")).unwrap();
    // let library_dir_data_model = dunce::canonicalize(root.join("../diesel-odbc/src/data-model/target/debug/")).unwrap();
    println!("cargo:rustc-link-search={}", env::join_paths(&[library_dir_diesel.clone()]).unwrap().to_str().unwrap());
    // panic!("{:?}", library_dir_data_model);
    // println!("cargo:rustc-link-search=d:/rust/lib_test2/target/debug/");
}

