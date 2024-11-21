import concurrent.futures
from concurrent.futures import ThreadPoolExecutor
import os
import sqlite3

import pandas as pd

from python.metric import complexity, empirical_probability
from python.repository import (
    parameters_by_run_id,
    state_vectors_by_run_id,
    total_runs,
)


def preprocessing(cache_path: str, database_connection, db_path: str) -> pd.DataFrame:
    results_file = cache_path
    if os.path.exists(results_file):
        total_results_df = pd.read_pickle(results_file)
    else:
        results = parallelized_processing(database_connection, db_path)
        total_results_df = pd.concat(results, ignore_index=True)
        total_results_df.to_pickle(results_file)
    return total_results_df


def process_run(run_id: int, runs_count: int, db_path: str) -> pd.DataFrame:
    print(f"Computing results for run {run_id}")
    con = sqlite3.connect(db_path)
    state_vectors = state_vectors_by_run_id(run_id, con)
    df_parameters = parameters_by_run_id(run_id, con)

    probability_mass = state_vectors["count"].map(
        lambda state_count: empirical_probability(state_count, runs_count)
    )
    df_parameters["complexity"] = complexity(probability_mass)
    return df_parameters


def parallelized_processing(con, db_path: str) -> list[pd.DataFrame]:
    runs_count = total_runs(con)
    print(f"Total runs to compute: {runs_count}")

    results = []

    # Use ThreadPoolExecutor for parallel execution
    with ThreadPoolExecutor() as executor:
        futures = [
            executor.submit(process_run, run_id, runs_count, db_path)
            for run_id in range(1, runs_count + 1)
        ]
        for future in concurrent.futures.as_completed(futures):
            # Gather results as each thread completes
            results.append(future.result())

    return results
