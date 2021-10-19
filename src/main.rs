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
use std::fs::{File};
use std::path::{Path};
use getopts::{Options};
use std::str::FromStr;
use std::cell::{RefCell};
use rand::{StdRng,SeedableRng};

use caminos_lib::{get_git_id,directory_main,file_main,Plugs,
	topology::{self},
	config_parser::{self,ConfigurationValue},
	experiments::{Action,ExperimentOptions},
	};
fn special_export(args: &str, plugs:&Plugs)
{
	let topology_cfg = match config_parser::parse(args)
	{
		Ok(x) => match x
		{
			config_parser::Token::Value(value) => value,
			_ => panic!("Not a value"),
		},
		Err(x) => panic!("Error parsing topology to export ({:?})",x),
	};
	let mut topology = None;
	let mut seed = None;
	let mut format = None;
	let mut filename = None;
	if let ConfigurationValue::Object(ref cv_name, ref cv_pairs)=topology_cfg
	{
		if cv_name!="Export"
		{
			panic!("A Export must be created from a `Export` object not `{}`",cv_name);
		}
		for &(ref name,ref value) in cv_pairs
		{
			//match name.as_ref()
			match AsRef::<str>::as_ref(&name)
			{
				"topology" =>
				{
					topology=Some(value);
				},
				"seed" => match value
				{
					&ConfigurationValue::Number(f) => seed=Some(f as usize),
					_ => panic!("bad value for seed"),
				},
				"format" => match value
				{
					&ConfigurationValue::Number(f) => format=Some(f as usize),
					_ => panic!("bad value for format"),
				},
				"filename" => match value
				{
					&ConfigurationValue::Literal(ref s) => filename=Some(s.to_string()),
					_ => panic!("bad value for filename"),
				},
				_ => panic!("Nothing to do with field {} in Export",name),
			}
		}
	}
	else
	{
		panic!("Trying to create a Export from a non-Object");
	}
	let seed=seed.unwrap_or(42);
	let topology_cfg=topology.expect("There were no topology.");
	let format=format.unwrap_or(0);
	let filename=filename.expect("There were no filename.");
	let rng=RefCell::new(StdRng::from_seed(&[seed]));
	let topology = topology::new_topology(topology::TopologyBuilderArgument{cv:&topology_cfg,plugs,rng:&rng});
	let mut topology_file=File::create(&filename).expect("Could not create topology file");
	topology.write_adjacencies_to_file(&mut topology_file,format).expect("Failed writing topology to file");
}


fn main()
{
	let args: Vec<String> = env::args().collect();

	println!("git_id={}",get_git_id());

	//println!("{:?}", args);
	//if args.len()!=2
	//{
	//	println!("Use:\n\t{} configuration_filename",args[0]);
	//	return;
	//}
	
	let mut opts = Options::new();
	//opts.optopt("l","launch","selected launching method (for directory experiment)","METHOD");
	opts.optopt("a","action","selected action to execute (for directory experiment)","METHOD");
	opts.optopt("r","results","file in which to write the simulation results (for file experiment)","FILE");
	opts.optopt("s","start_index","experiment index in which to start processing","INDEX");
	opts.optopt("e","end_index","experiment index in which to end processing","INDEX");
	opts.optopt("x","special","some special execution","SPECIAL_VALUE");
	opts.optopt("","special_args","arguments for special execution","SPECIAL_VALUE");
	opts.optopt("f","source","copy matching results from another path experiment","PATH");
	opts.optopt("w","where","select the subset of indices for which the configuration expression evaluates to true","EXPRESION");
	opts.optopt("m","message","write a message into the journal file","TEXT");
	opts.optflag("h","help","show this help");
	let option_matches= match opts.parse(&args[1..])
	{
		Ok(m) => m,
		Err(f) => panic!("{}",f.to_string()),
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
				special_export(&option_matches.opt_str("special_args").unwrap(),&plugs);
				return;
			},
			_ => panic!("unrecognized special function {}",special_str),
		}
	}

	let path=Path::new(&option_matches.free[0]);
	if path.is_dir()
	{
		let action=if option_matches.opt_present("action")
		{
			Action::from_str(&option_matches.opt_str("action").unwrap()).expect("Illegal action")
		}
		else
		{
			Action::LocalAndOutput
		};
		let mut options= ExperimentOptions::default();
		if option_matches.opt_present("source")
		{
			options.external_source = Some(Path::new(&option_matches.opt_str("source").unwrap()).to_path_buf());
		}
		if option_matches.opt_present("start_index")
		{
			options.start_index = Some(option_matches.opt_str("start_index").unwrap().parse::<usize>().expect("non-usize received from --start_index"));
		}
		if option_matches.opt_present("end_index")
		{
			options.end_index = Some(option_matches.opt_str("end_index").unwrap().parse::<usize>().expect("non-usize received from --end_index"));
		}
		if option_matches.opt_present("where")
		{
			let expr = match config_parser::parse_expression(&option_matches.opt_str("where").unwrap()).expect("error parsing the where clause")
			{
				config_parser::Token::Expression(expr) => expr,
				x => panic!("the where clause is not an expression ({:?}), which it should be.",x),
			};
			options.where_clause = Some(expr);
		}
		if option_matches.opt_present("message")
		{
			options.message = Some(option_matches.opt_str("message").unwrap());
		}
		return directory_main(&path,&args[0],&plugs,action,options);
	}
	else
	{
		//let mut f = File::open(&args[1]).expect("file cannot be opened");
		let mut f = File::open(&path).expect("file cannot be opened");
		let results_file= if option_matches.opt_present("results")
		{
			Some(File::create(option_matches.opt_str("results").unwrap()).expect("Could not create results file"))
		}
		else
		{
			None
		};
		return file_main(&mut f,&plugs,results_file);
	}
}
