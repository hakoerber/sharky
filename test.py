#!/usr/bin/env python3

import os
import random
import subprocess
import sys
import tempfile


def run(*args):
    subprocess.run(["cargo", "run", "--quiet", "--", *args], check=True)


tmpdir = tempfile.TemporaryDirectory()

secret_path = f"{tmpdir.name}/secret"
# between 1 an 2 MB
secret = os.urandom(int(1e6 * (1 + random.random())))

open(secret_path, "wb").write(secret)

run("share", secret_path, *[f"{tmpdir.name}/{i}.share" for i in range(1, 7)])

for i in range(4, 8):
    secret_path_recovered = f"{tmpdir.name}/secret.recovered_from_{i}"
    run(
        "recover",
        secret_path_recovered,
        *[f"{tmpdir.name}/{i}.share" for i in range(1, i)],
    )

    recovered_data = open(secret_path_recovered, "rb").read()

    if secret != recovered_data:
        print("failed")
        sys.exit(1)

print("all passed")
