/*!
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
$ caminos --help
```

To perform a single simulation create a file `my_experiment.cfg` and execute the simulator. See the `caminos-lib` crate for documentation on its format.
```bash
$ caminos my_experiment.cfg
```
You may set the `--results` flags to write the simulation result into it instead of to stdout.

For more complex experiments it is recommended to make a new directory `/path/to/my/experiment`. This directory should include a file `main.cfg` describing the experiment to perform and a file `main.od` describing the outputs to be generated. It may contain a file `remote` to help to `pull` result files launched remotely. Then, to run all the simulations locally and create the outputs execute the following.
```bash
$ caminos /path/to/my/experiment
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

# Special modes

The `special` flag enables extra modes other than simulating scenarios. Currently the only such availale mode is exporting a topology. By setting `--special=export` and `--special_args='Export{...}'` it will create a topology file. An example command is the following.

```bash
$ caminos --special=export --special_args='Export{topology:RandomRegularGraph{routers:500,degree:20,servers_per_router:1},seed:5,filename:"the_topology_file"}'
```
*/

use std::env;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use caminos_lib::{get_git_id,get_version_number,Plugs};


fn main()
{
	unsafe { backtrace_on_stack_overflow::enable(||println!("installed backtrace on stack overflow")); }
	let args: Vec<String> = env::args().collect();

	println!("git_id={} version_number={}",get_git_id(),get_version_number());

	let opts = caminos_lib::terminal_default_options();
	let option_matches= match opts.parse(&args[1..])
	{
		Ok(m) => m,
		Err(e) =>
		{
			eprintln!("Error when parsing options: {e}\n");
			std::process::exit(-1);
		}
	};

	if (option_matches.free.is_empty() && !option_matches.opt_present("special")) || option_matches.opt_present("help")
	{
		let brief = format!("Use:\n\t{binary} configuration_filename [-h] [--special=<method>] [--results=<path>]\n\t{binary} experiment_folder [--action=<method>] [--start_index=<index>] [--end_index=<index>] [--source=<path>]",binary=args[0]);
		print!("{}", opts.usage(&brief));
		return;
	}

	let plugs = Plugs::default();
	if option_matches.opt_present("special")
	{
		let special_str = option_matches.opt_str("special").expect("no special string");
		match special_str.as_ref()
		{
			"export" =>
			{
				caminos_lib::special_export(&option_matches.opt_str("special_args").unwrap(),&plugs);
				return;
			},
			_ => panic!("unrecognized special function {}",special_str),
		}
	}

	if let Err(error) = caminos_lib::terminal_main_normal_opts(&args,&plugs,option_matches)
	{
		eprintln!("Got error {error}");
	}
}

