fn main()
{
    println!("cargo:rustc-link-search=native=../build");
    println!("cargo:rustc-link-lib=static=feo_core");
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");
}