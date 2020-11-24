caminos
=====

This crate provides the CAMINOS simulator as a binary. Most of the functionality is obtained from the `caminos-lib` crate.

# Usage

To use this simulator first install it.

```bash
$ cargo install caminos
```

Then you should  be able to run it.
```bash
caminos --help
```

To perform a single simulation create a file `my_experiment.cfg` and execute the simulator. See the `caminos-lib` crate for documentation on its format.
```bash
caminos my_experiment.cfg
```
You may set the `--results` flags to write the simulation result into it instead of to stdout.

For more complex experiments it is recommended to make a new directory `/path/to/my/experiment`. This directory should include a file `main.cfg` describing the experiment to perform and a file `main.od` describing the outputs to be generated. It may contain a file `remote` to help to `pull` result files launched remotely. Then, to run all the simulations locally and create the outputs execute the following.
```bash
caminos /path/to/my/experiment
```

# Executing Simulations Using SLURM

If we have access to a machine with a SLURM queue system then a way to proceed is as follows.
* Make a local experiment with its `main.cfg`.
* Create the `/path/to/my/experiment/remote`
```
[
	Remote{
		name: "default",
		host: "the.remote.host",
		username: "myusername",
		root: "/path/in/the/remote/machine/to/my/experiment",
		binary: "/path/in/the/remote/to/caminos",
	},
]
```
* Perform a push to create the files in the remote.
```bash
local$ caminos /path/to/my/experiment --action=push
```
* Login into the remote machine.
* Create the slurm jobs
```bash
the.remote.host$ caminos /path/in/the/remote/machine/to/my/experiment --action=slurm
```
* Close the connection to the remote machine.
* Pull the results. It is fine if only a few have ended, you are indicated how many are yet to be completed.
```bash
local$ caminos /path/to/my/experiment --action=pull
```
* You may now generate your desired outputs if you are so inclined.
```bash
local$ caminos /path/to/my/experiment --action=output
```

