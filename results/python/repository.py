import pandas as pd


def state_vectors_by_run_id(run_id: int, con) -> pd.DataFrame:
    return pd.read_sql_query(
        f"""
        SELECT sv.px, sv.py, sv.pz, sv.vx, sv.vy, sv.vz, sv.count FROM state_vectors sv
        JOIN particle_parameters pp ON sv.particle_parameters_id = pp.id
        JOIN run_parameters rp ON pp.run_id = rp.run_id
        WHERE rp.run_id == {run_id}
        """,
        con,
    )


def total_runs(con) -> int:
    total_runs_df =  pd.read_sql_query(
        """
        SELECT MAX(run_id)
        FROM run_parameters;
        """,
        con,
    )
    return total_runs_df.iloc[0]["MAX(run_id)"]


def parameters_by_run_id(run_id: int, con) -> pd.DataFrame:
    return pd.read_sql_query(
        f"""
        SELECT * from run_parameters
        WHERE run_id == {run_id}
        """,
        con,
    )