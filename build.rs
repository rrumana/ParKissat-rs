use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.cpp");
    
    // Copy wrapper.h to output directory so stub.cpp can find it
    std::fs::copy("wrapper.h", out_path.join("wrapper.h"))
        .expect("Failed to copy wrapper.h");
    
    // For now, let's create a simple stub implementation to test the FFI structure
    // We'll build the actual ParKissat-RS integration later
    
    // Create a stub implementation
    let stub_cpp = r#"
#include "wrapper.h"
#include <cstdlib>
#include <cstring>
#include <vector>
#include <map>
#include <set>

extern "C" {

struct ParkissatSolver {
    std::vector<std::vector<int>> clauses;
    std::map<int, bool> model;
    ParkissatResult last_result;
    int num_variables;
    bool configured;
    
    ParkissatSolver() : last_result(PARKISSAT_UNKNOWN), num_variables(0), configured(false) {}
};

ParkissatSolver* parkissat_new(void) {
    try {
        return new ParkissatSolver();
    } catch (...) {
        return nullptr;
    }
}

void parkissat_delete(ParkissatSolver* solver) {
    if (solver) {
        delete solver;
    }
}

void parkissat_configure(ParkissatSolver* solver, const ParkissatConfig* config) {
    if (solver && config) {
        solver->configured = true;
    }
}

bool parkissat_load_dimacs(ParkissatSolver* solver, const char* filename) {
    return solver && filename && solver->configured;
}

void parkissat_add_clause(ParkissatSolver* solver, const int* literals, int size) {
    if (!solver || !literals || size <= 0) return;
    
    std::vector<int> clause(literals, literals + size);
    solver->clauses.push_back(clause);
    
    // Update variable count
    for (int lit : clause) {
        int var = abs(lit);
        if (var > solver->num_variables) {
            solver->num_variables = var;
        }
    }
}

void parkissat_set_variable_count(ParkissatSolver* solver, int num_vars) {
    if (solver && num_vars > 0) {
        solver->num_variables = num_vars;
    }
}

ParkissatResult parkissat_solve(ParkissatSolver* solver) {
    if (!solver || !solver->configured) {
        return PARKISSAT_UNKNOWN;
    }
    
    // Simple stub: check for obvious contradictions
    if (!solver->clauses.empty()) {
        // Check for unit clauses that contradict each other
        std::set<int> unit_clauses;
        for (const auto& clause : solver->clauses) {
            if (clause.size() == 1) {
                int lit = clause[0];
                if (unit_clauses.count(-lit)) {
                    // Found contradiction: both x and ¬x as unit clauses
                    solver->last_result = PARKISSAT_UNSAT;
                    return solver->last_result;
                }
                unit_clauses.insert(lit);
            }
        }
        
        // No obvious contradiction found, assume SAT
        solver->last_result = PARKISSAT_SAT;
        
        // Create a simple model (all variables true)
        solver->model.clear();
        for (int i = 1; i <= solver->num_variables; i++) {
            solver->model[i] = true;
        }
    } else {
        solver->last_result = PARKISSAT_UNSAT;
    }
    
    return solver->last_result;
}

ParkissatResult parkissat_solve_with_assumptions(ParkissatSolver* solver, const int* assumptions, int num_assumptions) {
    // For now, just call regular solve
    return parkissat_solve(solver);
}

bool parkissat_get_model_value(ParkissatSolver* solver, int variable) {
    if (!solver || variable <= 0) return false;
    
    auto it = solver->model.find(variable);
    return it != solver->model.end() ? it->second : false;
}

int parkissat_get_model_size(ParkissatSolver* solver) {
    return solver ? static_cast<int>(solver->model.size()) : 0;
}

void parkissat_get_model(ParkissatSolver* solver, int* model, int size) {
    if (!solver || !model || size <= 0) return;
    
    int i = 0;
    for (const auto& pair : solver->model) {
        if (i >= size) break;
        model[i++] = pair.second ? pair.first : -pair.first;
    }
}

ParkissatStatistics parkissat_get_statistics(ParkissatSolver* solver) {
    ParkissatStatistics stats = {0, 0, 0, 0, 0.0};
    if (solver) {
        stats.decisions = 1;  // Stub values
        stats.propagations = 1;
    }
    return stats;
}

void parkissat_interrupt(ParkissatSolver* solver) {
    // Stub implementation
}

void parkissat_clear_interrupt(ParkissatSolver* solver) {
    // Stub implementation
}

} // extern "C"
"#;

    // Write the stub to a temporary file
    let stub_path = PathBuf::from(&out_dir).join("stub.cpp");
    std::fs::write(&stub_path, stub_cpp).expect("Failed to write stub file");
    
    // Build the stub
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file(&stub_path)
        .include(&out_path)  // Add include path for wrapper.h
        .flag("-std=c++17")
        .flag("-fPIC")
        .compile("parkissat_stub");
    
    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("parkissat_.*")
        .allowlist_type("Parkissat.*")
        .allowlist_var("PARKISSAT_.*")
        .generate()
        .expect("Unable to generate bindings");
    
    let out_path = PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}