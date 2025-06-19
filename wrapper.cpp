#include "wrapper.h"
#include "painless-src/painless.h"
#include "painless-src/solvers/SolverInterface.h"
#include "painless-src/solvers/SolverFactory.h"
#include "painless-src/clauses/ClauseExchange.h"
#include "painless-src/utils/Parameters.h"
#include "painless-src/working/SequentialWorker.h"
#include "painless-src/working/Portfolio.h"

#include <vector>
#include <memory>
#include <cstring>
#include <atomic>
#include <thread>
#include <mutex>

extern "C" {

struct ParkissatSolver {
    std::vector<SolverInterface*> solvers;
    std::vector<ClauseExchange*> clauses;
    std::vector<int> model;
    ParkissatResult last_result;
    int num_variables;
    bool interrupted;
    ParkissatConfig config;
    
    ParkissatSolver() : last_result(PARKISSAT_UNKNOWN), num_variables(0), interrupted(false) {
        // Initialize default config
        config.num_threads = 1;
        config.timeout_seconds = 0;
        config.random_seed = 0;
        config.enable_preprocessing = false;
        config.verbosity = 0;
    }
    
    ~ParkissatSolver() {
        // Clean up clauses
        for (auto* clause : clauses) {
            delete[] clause;
        }
        
        // Clean up solvers
        for (auto* solver : solvers) {
            solver->release();
        }
    }
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
    if (!solver || !config) return;
    
    solver->config = *config;
    
    // Initialize solvers based on configuration
    solver->solvers.clear();
    
    int num_solvers = config->num_threads > 0 ? config->num_threads : 1;
    
    for (int i = 0; i < num_solvers; i++) {
        // Create a solver instance (using KissatBonus as default)
        SolverInterface* s = SolverFactory::createKissatBonusSolver();
        if (s) {
            solver->solvers.push_back(s);
            
            // Configure solver parameters
            parameter p;
            p.tier1 = 2;
            p.chrono = 1;
            p.stable = 1;
            p.walkinitially = 0;
            p.target = 1;
            p.phase = 1;
            p.heuristic = 1;
            p.margin = 0;
            p.ccanr = 1;
            p.targetinc = 1;
            
            s->setParameter(p);
            
            if (config->random_seed != 0) {
                s->diversify(i + config->random_seed);
            } else {
                s->diversify(i);
            }
        }
    }
}

bool parkissat_load_dimacs(ParkissatSolver* solver, const char* filename) {
    if (!solver || !filename) return false;
    
    try {
        if (!solver->solvers.empty()) {
            return solver->solvers[0]->loadFormula(filename);
        }
        return false;
    } catch (...) {
        return false;
    }
}

void parkissat_add_clause(ParkissatSolver* solver, const int* literals, int size) {
    if (!solver || !literals || size <= 0) return;
    
    try {
        // Create ClauseExchange structure
        ClauseExchange* clause = (ClauseExchange*)malloc(sizeof(ClauseExchange) + size * sizeof(int));
        if (!clause) return;
        
        clause->nbRefs.store(1);
        clause->lbd = 2; // Default LBD value
        clause->from = 0;
        clause->size = size;
        
        // Copy literals
        for (int i = 0; i < size; i++) {
            clause->lits[i] = literals[i];
            
            // Update variable count
            int var = abs(literals[i]);
            if (var > solver->num_variables) {
                solver->num_variables = var;
            }
        }
        
        solver->clauses.push_back(clause);
        
        // Add clause to all solvers
        for (auto* s : solver->solvers) {
            s->addClause(clause);
        }
    } catch (...) {
        // Handle exception
    }
}

void parkissat_set_variable_count(ParkissatSolver* solver, int num_vars) {
    if (solver && num_vars > 0) {
        solver->num_variables = num_vars;
    }
}

ParkissatResult parkissat_solve(ParkissatSolver* solver) {
    if (!solver || solver->solvers.empty()) {
        return PARKISSAT_UNKNOWN;
    }
    
    try {
        solver->interrupted = false;
        
        std::vector<int> empty_cube;
        SatResult result;
        
        if (solver->solvers.size() == 1) {
            // Single-threaded solving
            SolverInterface* s = solver->solvers[0];
            result = s->solve(empty_cube);
            if (result == SAT) {
                solver->model = s->getModel();
            }
        } else {
            // Multi-threaded solving using threads
            std::vector<std::thread> threads;
            std::atomic<bool> solved(false);
            std::atomic<SatResult> final_result(UNKNOWN);
            std::mutex model_mutex;
            
            for (size_t i = 0; i < solver->solvers.size(); i++) {
                threads.emplace_back([&, i]() {
                    if (solved.load()) return;
                    
                    SolverInterface* s = solver->solvers[i];
                    SatResult local_result = s->solve(empty_cube);
                    
                    if (local_result == SAT || local_result == UNSAT) {
                        bool expected = false;
                        if (solved.compare_exchange_strong(expected, true)) {
                            // This thread found the result first
                            final_result.store(local_result);
                            if (local_result == SAT) {
                                std::lock_guard<std::mutex> lock(model_mutex);
                                solver->model = s->getModel();
                            }
                            
                            // Interrupt other solvers
                            for (auto* other_solver : solver->solvers) {
                                if (other_solver != s) {
                                    other_solver->setSolverInterrupt();
                                }
                            }
                        }
                    }
                });
            }
            
            // Wait for all threads to complete
            for (auto& thread : threads) {
                thread.join();
            }
            
            result = final_result.load();
        }
        
        switch (result) {
            case SAT:
                solver->last_result = PARKISSAT_SAT;
                break;
            case UNSAT:
                solver->last_result = PARKISSAT_UNSAT;
                solver->model.clear();
                break;
            default:
                solver->last_result = PARKISSAT_UNKNOWN;
                solver->model.clear();
                break;
        }
        
        return solver->last_result;
    } catch (...) {
        return PARKISSAT_UNKNOWN;
    }
}

ParkissatResult parkissat_solve_with_assumptions(ParkissatSolver* solver, const int* assumptions, int num_assumptions) {
    if (!solver || solver->solvers.empty()) {
        return PARKISSAT_UNKNOWN;
    }
    
    try {
        solver->interrupted = false;
        
        // Convert assumptions to vector
        std::vector<int> cube;
        if (assumptions && num_assumptions > 0) {
            cube.assign(assumptions, assumptions + num_assumptions);
        }
        
        SatResult result;
        
        if (solver->solvers.size() == 1) {
            // Single-threaded solving
            SolverInterface* s = solver->solvers[0];
            result = s->solve(cube);
            if (result == SAT) {
                solver->model = s->getModel();
            }
        } else {
            // Multi-threaded solving using threads
            std::vector<std::thread> threads;
            std::atomic<bool> solved(false);
            std::atomic<SatResult> final_result(UNKNOWN);
            std::mutex model_mutex;
            
            for (size_t i = 0; i < solver->solvers.size(); i++) {
                threads.emplace_back([&, i]() {
                    if (solved.load()) return;
                    
                    SolverInterface* s = solver->solvers[i];
                    SatResult local_result = s->solve(cube);
                    
                    if (local_result == SAT || local_result == UNSAT) {
                        bool expected = false;
                        if (solved.compare_exchange_strong(expected, true)) {
                            // This thread found the result first
                            final_result.store(local_result);
                            if (local_result == SAT) {
                                std::lock_guard<std::mutex> lock(model_mutex);
                                solver->model = s->getModel();
                            }
                            
                            // Interrupt other solvers
                            for (auto* other_solver : solver->solvers) {
                                if (other_solver != s) {
                                    other_solver->setSolverInterrupt();
                                }
                            }
                        }
                    }
                });
            }
            
            // Wait for all threads to complete
            for (auto& thread : threads) {
                thread.join();
            }
            
            result = final_result.load();
        }
        
        switch (result) {
            case SAT:
                solver->last_result = PARKISSAT_SAT;
                break;
            case UNSAT:
                solver->last_result = PARKISSAT_UNSAT;
                solver->model.clear();
                break;
            default:
                solver->last_result = PARKISSAT_UNKNOWN;
                solver->model.clear();
                break;
        }
        
        return solver->last_result;
    } catch (...) {
        return PARKISSAT_UNKNOWN;
    }
}

bool parkissat_get_model_value(ParkissatSolver* solver, int variable) {
    if (!solver || variable <= 0 || variable > solver->num_variables) {
        return false;
    }
    
    if (solver->last_result != PARKISSAT_SAT || solver->model.empty()) {
        return false;
    }
    
    // Model is 1-indexed, find the variable
    for (size_t i = 0; i < solver->model.size(); i++) {
        if (abs(solver->model[i]) == variable) {
            return solver->model[i] > 0;
        }
    }
    
    return false; // Variable not found in model
}

int parkissat_get_model_size(ParkissatSolver* solver) {
    if (!solver) return 0;
    return static_cast<int>(solver->model.size());
}

void parkissat_get_model(ParkissatSolver* solver, int* model, int size) {
    if (!solver || !model || size <= 0) return;
    
    int copy_size = std::min(size, static_cast<int>(solver->model.size()));
    for (int i = 0; i < copy_size; i++) {
        model[i] = solver->model[i];
    }
}

ParkissatStatistics parkissat_get_statistics(ParkissatSolver* solver) {
    ParkissatStatistics stats = {0, 0, 0, 0, 0.0};
    
    if (!solver || solver->solvers.empty()) {
        return stats;
    }
    
    try {
        // Aggregate statistics from all solvers
        for (auto* s : solver->solvers) {
            SolvingStatistics s_stats = s->getStatistics();
            stats.propagations += s_stats.propagations;
            stats.decisions += s_stats.decisions;
            stats.conflicts += s_stats.conflicts;
            stats.restarts += s_stats.restarts;
            if (s_stats.memPeak > stats.mem_peak) {
                stats.mem_peak = s_stats.memPeak;
            }
        }
    } catch (...) {
        // Return zero stats on error
    }
    
    return stats;
}

void parkissat_interrupt(ParkissatSolver* solver) {
    if (!solver) return;
    
    solver->interrupted = true;
    for (auto* s : solver->solvers) {
        s->setSolverInterrupt();
    }
}

void parkissat_clear_interrupt(ParkissatSolver* solver) {
    if (!solver) return;
    
    solver->interrupted = false;
    for (auto* s : solver->solvers) {
        s->unsetSolverInterrupt();
    }
}

} // extern "C"