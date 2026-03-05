fn main() {
    // Линкуем основную библиотеку
    println!("cargo:rustc-link-search=native=PLI/build/lib");
    println!("cargo:rustc-link-lib=static=pli");
}
