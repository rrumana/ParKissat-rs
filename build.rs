use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.cpp");
    println!("cargo:rerun-if-changed=ParKissat-RS");
    
    // Get the current directory for ParKissat-RS
    let parkissat_dir = PathBuf::from("ParKissat-RS");
    let kissat_dir = parkissat_dir.join("kissat_mab");
    let painless_dir = parkissat_dir.join("painless-src");
    
    // Step 1: Build kissat_mab
    println!("cargo:warning=Building kissat_mab...");
    
    // Make configure script executable
    let configure_path = kissat_dir.join("configure");
    Command::new("chmod")
        .args(&["+x", configure_path.to_str().unwrap()])
        .status()
        .expect("Failed to make configure executable");
    
    // Run configure script
    let configure_status = Command::new("./configure")
        .arg("--compact")
        .current_dir(&kissat_dir)
        .status()
        .expect("Failed to run kissat configure");
    
    if !configure_status.success() {
        panic!("kissat configure failed");
    }
    
    // Build kissat
    let make_status = Command::new("make")
        .current_dir(&kissat_dir)
        .status()
        .expect("Failed to run make for kissat");
    
    if !make_status.success() {
        panic!("kissat make failed");
    }
    
    // Step 2: Build painless-src
    println!("cargo:warning=Building painless-src...");
    
    let painless_make_status = Command::new("make")
        .current_dir(&painless_dir)
        .status()
        .expect("Failed to run make for painless-src");
    
    if !painless_make_status.success() {
        panic!("painless-src make failed");
    }
    
    // Step 3: Copy wrapper.h to output directory
    std::fs::copy("wrapper.h", out_path.join("wrapper.h"))
        .expect("Failed to copy wrapper.h");
    
    // Step 4: Compile wrapper.cpp with proper includes and linking
    println!("cargo:warning=Compiling wrapper.cpp...");
    
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("wrapper.cpp")
        .include(&out_path)  // For wrapper.h
        .include(&parkissat_dir)  // For ParKissat-RS headers
        .include(&kissat_dir)  // For kissat headers
        .include(&painless_dir)  // For painless headers
        .flag("-std=c++17")
        .flag("-O3")
        .flag("-DNDEBUG")
        .flag("-fopenmp")  // Enable OpenMP
        .flag("-fPIC");
    
    // Add painless-src object files to the build first
    let painless_objects = [
        "clauses/ClauseBuffer.o",
        "clauses/ClauseDatabase.o",
        "sharing/HordeSatSharing.o",
        "sharing/Sharer.o",
        "simplify/parse.o",
        "simplify/simplify.o",
        "solvers/KissatBonus.o",
        "solvers/SolverFactory.o",
        "utils/Logger.o",
        "utils/Parameters.o",
        "utils/SatUtils.o",
        "utils/System.o",
        "working/Portfolio.o",
        "working/SequentialWorker.o",
    ];
    
    for obj in &painless_objects {
        let obj_path = painless_dir.join(obj);
        build.object(&obj_path);
    }
    
    // Extract and add all object files from kissat library
    let kissat_build_dir = kissat_dir.join("build");
    let kissat_objects = [
        "allocate.o", "analyze.o", "ands.o", "application.o", "arena.o", "assign.o",
        "autarky.o", "averages.o", "backtrack.o", "backward.o", "build.o", "bump.o",
        "ccnr.o", "check.o", "clause.o", "clueue.o", "collect.o", "colors.o",
        "compact.o", "config.o", "cvec.o", "decide.o", "deduce.o", "dense.o",
        "dominate.o", "dump.o", "eliminate.o", "equivalences.o", "error.o", "extend.o",
        "failed.o", "file.o", "flags.o", "format.o", "forward.o", "frames.o",
        "gates.o", "handle.o", "heap.o", "ifthenelse.o", "import.o", "internal.o",
        "learn.o", "limits.o", "logging.o", "ls.o", "minimize.o", "mode.o",
        "options.o", "parse.o", "phases.o", "print.o", "probe.o", "profile.o",
        "promote.o", "proof.o", "propdense.o", "prophyper.o", "proprobe.o", "propsearch.o",
        "queue.o", "reduce.o", "reluctant.o", "rephase.o", "report.o", "resize.o",
        "resolve.o", "resources.o", "restart.o", "search.o", "smooth.o", "sort.o",
        "stack.o", "statistics.o", "strengthen.o", "substitute.o", "terminate.o",
        "ternary.o", "trail.o", "transitive.o", "utilities.o", "vector.o", "vivify.o",
        "walk.o", "watch.o", "weaken.o", "witness.o", "xors.o"
    ];
    
    for obj in &kissat_objects {
        let obj_path = kissat_build_dir.join(obj);
        if obj_path.exists() {
            build.object(&obj_path);
        }
    }
    
    // Add all the required library paths
    println!("cargo:rustc-link-search=native={}", kissat_dir.join("build").display());
    println!("cargo:rustc-link-search=native={}", painless_dir.display());
    
    // Link required system libraries
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=gomp");  // OpenMP library
    println!("cargo:rustc-link-lib=stdc++");  // C++ standard library
    
    // Compile the wrapper
    build.compile("parkissat_wrapper");
    
    // Step 5: Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("parkissat_.*")
        .allowlist_type("Parkissat.*")
        .allowlist_var("PARKISSAT_.*")
        .generate()
        .expect("Unable to generate bindings");
    
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    
    println!("cargo:warning=ParKissat-RS build completed successfully");
}