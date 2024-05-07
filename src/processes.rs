use crate::{
    discord::ClientBundle,
    parser::{CliProcessesAdd, CliProcessesPriority, CliProcessesPriorityOperation},
    prelude::*,
};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

/// Creates a vector of all found target processes. Processes are searched for by process name from `ProcessesConfig`.
#[instrument(skip_all)]
pub fn get_names(config: &ProcessesConfig) -> Vec<String> {
    let sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );

    let mut active_target_processes: Vec<String> = Vec::new();

    for process in &config.processes {
        let mut p = sys.processes_by_exact_name(&process.name);
        if let Some(found_process) = p.next() {
            active_target_processes.push(found_process.name().to_owned());
        }
    }

    trace!("Found target processes \n{active_target_processes:#?}");

    return active_target_processes;
}

/// Returns a tuple with the process text and process icon of the first active process found by `get_names()`.
#[instrument(skip_all)]
pub fn get_active_data(config: &ProcessesConfig, processes: &Vec<String>) -> (String, String) {
    for target_process in &config.processes {
        if processes.first() == Some(&target_process.name) {
            trace!("Process chosen:\n{target_process:#?}");
            return (
                target_process.text.to_owned(),
                target_process.image.to_owned(),
            );
        }
    }
    trace!("No active target processes, using idle data");

    let mut idle_text = config.idle_text.to_owned();
    let mut idle_image = config.idle_image.to_owned();

    if idle_text.is_empty() {
        idle_text = "Idle".to_string();
    }
    if idle_image.is_empty() {
        idle_image = "idle".to_string();
    }

    return (idle_text, idle_image);
}

pub fn print_data_list(config: &ProcessesConfig) -> Result<Option<ClientBundle>, ()> {
    for (index, process) in config.processes.iter().enumerate() {
        println!(
            "Process {}\n\tIcon: \"{}\"\n\tText: \"{}\"\n\tName: \"{}\"",
            index, process.image, process.text, process.name
        );
    }
    return Ok(None);
}

#[instrument(skip_all)]
pub fn add_process(
    config: &mut ProcessesConfig,
    args: CliProcessesAdd,
) -> Result<Option<ClientBundle>, ()> {
    let trace_data: CliProcessesAdd = args.clone();
    let index: usize = config.processes.len();

    config.processes.push(ProcessConfig {
        image: args.image,
        name: args.name,
        text: args.text,
    });

    trace!("Added new process {trace_data:?} to processes list at index {index}");

    return write_config(config);
}

#[instrument(skip_all)]
pub fn change_process_priority(
    config: &mut ProcessesConfig,
    arg: CliProcessesPriority,
) -> Result<Option<ClientBundle>, ()> {
    fn set_index(
        config: &mut ProcessesConfig,
        name: String,
        old_index: usize,
        new_index: usize,
    ) -> Result<Option<ClientBundle>, ()> {
        trace!("Process {name} will be set to index {new_index}");

        let process: ProcessConfig = config.processes.remove(old_index);
        config.processes.insert(new_index, process);

        println!("Set process {name} to priority {new_index}");
        return write_config(config);
    }

    return match arg {
        CliProcessesPriority {
            name,
            operation:
                CliProcessesPriorityOperation {
                    decrease: true,
                    increase: false,
                    set: None,
                },
        } => {
            if let Some(index) = config
                .processes
                .iter()
                .position(|process: &ProcessConfig| process.name == name)
            {
                let new_index: usize =
                    (index as i32 + 1).clamp(0, config.processes.len() as i32 - 1) as usize;
                set_index(config, name, index, new_index)
            } else {
                error!("No process named {name} found");
                Err(())
            }
        }
        CliProcessesPriority {
            name,
            operation:
                CliProcessesPriorityOperation {
                    decrease: false,
                    increase: true,
                    set: None,
                },
        } => {
            if let Some(index) = config
                .processes
                .iter()
                .position(|process: &ProcessConfig| process.name == name)
            {
                let new_index: usize =
                    (index as i32 - 1).clamp(0, config.processes.len() as i32 - 1) as usize;
                set_index(config, name, index, new_index)
            } else {
                error!("No process named {name} found");
                Err(())
            }
        }
        CliProcessesPriority {
            name,
            operation:
                CliProcessesPriorityOperation {
                    decrease: false,
                    increase: false,
                    set: Some(new_index),
                },
        } => {
            if let Some(index) = config
                .processes
                .iter()
                .position(|process: &ProcessConfig| process.name == name)
            {
                let new_index: usize =
                    (new_index as i32).clamp(0, config.processes.len() as i32 - 1) as usize;
                set_index(config, name, index, new_index)
            } else {
                error!("No process named {name} found");
                Err(())
            }
        }
        _ => unreachable!("An operation is required and all are mutually exclusive"),
    };
}

pub fn remove_process(
    config: &mut ProcessesConfig,
    name: String,
) -> Result<Option<ClientBundle>, ()> {
    if let Some(index) = config
        .processes
        .iter()
        .position(|process: &ProcessConfig| process.name == name)
    {
        config.processes.remove(index);

        trace!("Removed process {name}");
        println!("Removed process {name}");

        return write_config(config);
    } else {
        error!("No process named {name} found");
        return Err(());
    }
}