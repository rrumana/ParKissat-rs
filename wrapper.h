#ifndef PARKISSAT_WRAPPER_H
#define PARKISSAT_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

// Forward declarations
typedef struct ParkissatSolver ParkissatSolver;

// Result codes
typedef enum {
    PARKISSAT_SAT = 10,
    PARKISSAT_UNSAT = 20,
    PARKISSAT_UNKNOWN = 0
} ParkissatResult;

// Solver statistics
typedef struct {
    uint64_t propagations;
    uint64_t decisions;
    uint64_t conflicts;
    uint64_t restarts;
    double mem_peak;
} ParkissatStatistics;

// Configuration parameters
typedef struct {
    int num_threads;
    int timeout_seconds;
    int random_seed;
    bool enable_preprocessing;
    int verbosity;
} ParkissatConfig;

// Core solver functions
ParkissatSolver* parkissat_new(void);
void parkissat_delete(ParkissatSolver* solver);

// Configuration
void parkissat_configure(ParkissatSolver* solver, const ParkissatConfig* config);

// Problem setup
bool parkissat_load_dimacs(ParkissatSolver* solver, const char* filename);
void parkissat_add_clause(ParkissatSolver* solver, const int* literals, int size);
void parkissat_set_variable_count(ParkissatSolver* solver, int num_vars);

// Solving
ParkissatResult parkissat_solve(ParkissatSolver* solver);
ParkissatResult parkissat_solve_with_assumptions(ParkissatSolver* solver, const int* assumptions, int num_assumptions);

// Results
bool parkissat_get_model_value(ParkissatSolver* solver, int variable);
int parkissat_get_model_size(ParkissatSolver* solver);
void parkissat_get_model(ParkissatSolver* solver, int* model, int size);

// Statistics
ParkissatStatistics parkissat_get_statistics(ParkissatSolver* solver);

// Control
void parkissat_interrupt(ParkissatSolver* solver);
void parkissat_clear_interrupt(ParkissatSolver* solver);

#ifdef __cplusplus
}
#endif

#endif // PARKISSAT_WRAPPER_H