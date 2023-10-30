#!/usr/bin/env python3
"""Run `cargo bench`, print the output with CPU information to a timestamped
file.

Does not work on Windows (WSL works).
"""


import platform
import subprocess as sp
import sys
import time
from datetime import datetime
from inspect import cleandoc
from pathlib import Path


def decode_sp_out(b: bytes) -> str:
    return b.decode(errors="ignore").strip()


def get_dtime() -> str:
    return datetime.utcnow().strftime(r"%Y-%m-%d_%H%M")


def git_describe() -> str:
    args = ["git", "describe", "--always", "--tags"]
    return decode_sp_out(sp.check_output(args))


def get_fpath(dtime: str, describe: str) -> tuple[str, Path]:
    fname = f"{describe}_{dtime}.bench"
    fpath = Path(__file__).resolve().parents[0] / "results" / fname
    return (fname, fpath)


def rustc_version() -> str:
    return decode_sp_out(sp.check_output(["rustc", "--version"]))


def get_cpu_info() -> str:
    s = ""
    if platform.system() == "Darwin":
        cmd = ["sysctl", "-n", "machdep.cpu.brand_string"]
        s += decode_sp_out(sp.check_output(cmd))
    else:
        tmp = decode_sp_out(sp.check_output("lscpu"))
        for line in tmp.splitlines():
            if (
                "Architecture" in line
                or "Model name" in line
                or "Socket" in line
                or "Thread" in line
                or "CPU(s)" in line
                or "MHz" in line
            ):
                s += line
    return s


def main():
    start_time = time.time()
    dtime = get_dtime()
    describe = git_describe()
    fname, fpath = get_fpath(dtime, describe)
    version = rustc_version()
    cpu_info = get_cpu_info()
    cmd = ["cargo", "bench", "--features", "unstable-bench"]
    cmd += sys.argv[1:]

    header_str = (
        cleandoc(
            f"""
        {fname}

        Benchmark from {dtime} on commit {describe}
        {version}

        CPU Information:
        {cpu_info}

        Running: '{" ".join(cmd)}'
        """
        )
        + "\n\n\n"
    )

    print(header_str)
    output = header_str

    with sp.Popen(cmd, stdout=sp.PIPE, bufsize=1, universal_newlines=True) as p:
        for line in p.stdout:
            print(line, end="")  # process line here
            output += line

    if p.returncode != 0:
        print("\nCommand did not complete successfully")
        exit(p.returncode)

    end_time = time.time()
    elapsed_time = end_time - start_time
    time_str = f"\nTotal execution time: {time.strftime('%H:%M:%S', time.gmtime(elapsed_time))}"
    output += time_str
    print(time_str)
    print("\nWriting file '{fpath}'...", end="")

    with open(fpath, "w") as f:
        f.write(output)

    print(" Done!")


if __name__ == "__main__":
    main()
