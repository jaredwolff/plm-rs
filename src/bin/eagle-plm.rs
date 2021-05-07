use clap::{crate_version, Clap};
use eagle_plm::{config, establish_connection, prompt, tables::*, Application};
use std::io;

#[derive(Clap)]
#[clap(version = crate_version!())]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
    /// Optional path to config
    #[clap(short, long)]
    config_path: Option<String>,
}

#[derive(Clap)]
#[clap(version = crate_version!())]
enum SubCommand {
    Install(Install),
    Parts(Parts),
    Build(Build),
    Inventory(Inventory),
    Bom(Bom),
}

/// A subcommand for installing configuration to $HOME
#[derive(Clap)]
struct Install {}

/// A subcommand for importing and showing BOM by part number
#[derive(Clap)]
struct Bom {
    #[clap(subcommand)]
    subcmd: BomSubCommand,
}

#[derive(Clap)]
#[clap(version = crate_version!())]
enum BomSubCommand {
    Import(ImportBom),
    Show(ShowBom),
    Export(ExportBom),
}

/// A subcommand for importing a bom from an Eagle .sch file
#[derive(Clap)]
struct ExportBom {
    /// Name of BOM to export
    name: String,
    /// Version of the part in question
    #[clap(short, long)]
    version: Option<i32>,
}

/// A subcommand for importing a bom from an Eagle .sch file
#[derive(Clap)]
struct ImportBom {
    /// Path of .sch file to be imported
    filename: String,
}

/// A subcommand for showing a bom from pn
#[derive(Clap)]
struct ShowBom {
    /// Part number of the Bom in question
    part_number: String,
    /// Version of the part in question
    #[clap(short, long)]
    version: Option<i32>,
}

/// A subcommand for adding/modifying/removing parts
#[derive(Clap)]
struct Parts {
    #[clap(subcommand)]
    subcmd: PartsSubCommand,
}

#[derive(Clap)]
#[clap(version = crate_version!())]
enum PartsSubCommand {
    Create(CreateParts),
    Delete(DeleteParts),
    Show(ShowParts),
    Rename(RenamePart),
}

/// Create parts manually
#[derive(Clap)]
struct CreateParts {
    /// Create part from a .csv file
    #[clap(short, long)]
    filename: Option<String>,
}

/// Delete parts manually
#[derive(Clap)]
struct DeleteParts {}

/// Show all parts
#[derive(Clap)]
struct ShowParts {}

/// Rename a part
#[derive(Clap)]
struct RenamePart {}

#[derive(Clap)]
#[clap(version = crate_version!())]
enum BuildSubCommand {
    Create(CreateBuild),
    Delete(DeleteBuild),
    Show(ShowBuilds),
    Complete(CompleteBuild),
}

/// Create build manually
#[derive(Clap)]
struct CreateBuild {}

/// Delete build manually
#[derive(Clap)]
struct DeleteBuild {
    /// ID for the build. Get an id from builds show
    build_id: i32,
}

/// Show all builds
#[derive(Clap)]
struct ShowBuilds {
    /// Show all builds. (Completed are hidden by default)
    #[clap(short, long)]
    all: bool,
}

/// Complete a build by id
#[derive(Clap)]
struct CompleteBuild {
    /// ID for the build. Get an id from builds show
    build_id: i32,
}

/// A subcommand for adding/modifying/removing/completing builds
#[derive(Clap)]
struct Build {
    #[clap(subcommand)]
    subcmd: BuildSubCommand,
}

/// A subcommand for adding/modifying/removing inventory
#[derive(Clap)]
struct Inventory {
    #[clap(subcommand)]
    subcmd: InventorySubCommand,
}

#[derive(Clap)]
#[clap(version = crate_version!())]
enum InventorySubCommand {
    Create(CreateInventory),
    Import(ImportInventory),
    Update(UpdateInventory),
    Export(ExportInventory),
    Shortages(ExportInventoryShortages),
    Delete(DeleteInventory),
    Show(ShowInventory),
}

/// Create inventory manually
#[derive(Clap)]
struct CreateInventory {}

/// Import inventory via .csv
#[derive(Clap)]
struct ImportInventory {
    /// Inventory from a .csv file
    filename: String,
}

/// Import inventory via .csv
#[derive(Clap)]
struct UpdateInventory {
    /// Inventory from a .csv file
    filename: String,
}

/// Export inventory and shortages via .csv
#[derive(Clap)]
struct ExportInventory {
    /// Flag to export all inventory (event 0 qty entries)
    #[clap(short, long)]
    export_all: bool,
    /// Inventory to a .csv file
    filename: String,
}

/// Export inventory and shortages via .csv
#[derive(Clap)]
struct ExportInventoryShortages {
    /// Inventory to a .csv file
    filename: String,
}

/// Delete inventory manually
#[derive(Clap)]
struct DeleteInventory {}

/// Show all inventory
#[derive(Clap)]
struct ShowInventory {
    /// Show inventory shortages
    #[clap(short, long)]
    show_shortage: bool,
    /// Show show non short entries
    #[clap(short, long)]
    all_entries: bool,
}

fn main() {
    let opts: Opts = Opts::parse();

    // Config path
    let config_path = match config::get_config_path(&opts.config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Unable to get a valid config path. Error: {}", e);
            std::process::exit(1);
        }
    };

    let config: config::Config;

    // First check if the config is valid
    match &opts.subcmd {
        SubCommand::Install(_s) => {
            let db_name = "database.db".to_string();

            config = config::Config {
                database_name: db_name.clone(),
                library_name: "your-library".to_string(),
                attrition_config: config::AttritionConfig {
                    entries: Vec::new(),
                },
                part_number_ignore_list: Vec::new(),
            };

            // Install the config
            if config::save_config(&config, &config_path).is_err() {
                eprintln!("Unable to install database and config");
                std::process::exit(1);
            };

            println!("Config installed to {}", config_path.to_string_lossy());
            std::process::exit(0);
        }
        _ => {
            // Set the config
            config = match config::load_config(&config_path) {
                Ok(c) => c,
                Err(e) => {
                    if e.to_string().contains("No such file or directory") {
                        eprintln!("Unable to get config. Run `eagle-plm install` first.");
                    } else {
                        eprintln!("Error loading configuration: {}", e);
                    }

                    std::process::exit(1);
                }
            };
        }
    };

    // Establish connection!
    let conn = establish_connection(&config.database_name);

    // For prompts
    let stdio = io::stdin();
    let input = stdio.lock();
    let output = io::stdout();

    let prompt = prompt::Prompt {
        reader: input,
        writer: output,
    };

    // Settings for application
    let mut app = Application {
        config_path,
        config,
        prompt,
        conn,
    };

    // Then run the command.
    match opts.subcmd {
        SubCommand::Build(s) => match s.subcmd {
            BuildSubCommand::Create(_) => {
                builds::create(&mut app);
            }
            // TODO: take the next argument after delete instead of needing a flag...
            BuildSubCommand::Delete(a) => {
                builds::delete(&mut app, a.build_id);
            }
            BuildSubCommand::Show(a) => {
                builds::show(&mut app, a.all);
            }
            BuildSubCommand::Complete(a) => {
                builds::complete(&mut app, a.build_id);
            }
        },
        SubCommand::Inventory(s) => match s.subcmd {
            InventorySubCommand::Create(_) => {
                inventory::create(&mut app);
            }
            InventorySubCommand::Import(a) => {
                inventory::create_from_file(&mut app, &a.filename);
            }
            InventorySubCommand::Update(a) => {
                inventory::update_from_file(&mut app, &a.filename);
            }
            InventorySubCommand::Export(a) => {
                inventory::export_to_file(&mut app, &a.filename, a.export_all);
            }
            InventorySubCommand::Shortages(a) => {
                inventory::export_shortages_to_file(&mut app, &a.filename);
            }
            InventorySubCommand::Delete(_) => {
                println!("Not implemented!");
            }
            InventorySubCommand::Show(a) => {
                if a.show_shortage {
                    inventory::show_shortage(&mut app, a.all_entries);
                } else {
                    inventory::show(&mut app, a.all_entries);
                }
            }
        },
        // TODO: Search for a part
        SubCommand::Parts(s) => match s.subcmd {
            PartsSubCommand::Create(a) => match a.filename {
                Some(x) => parts::create_by_csv(&mut app, &x),
                None => parts::create(&mut app),
            },
            PartsSubCommand::Delete(_) => {
                parts::delete(&mut app);
            }
            PartsSubCommand::Show(_) => {
                parts::show(&mut app);
            }
            PartsSubCommand::Rename(_) => {
                parts::rename(&mut app);
            }
        },
        SubCommand::Bom(s) => match s.subcmd {
            BomSubCommand::Import(a) => {
                bom::import(&mut app, &a.filename);
            }
            BomSubCommand::Export(a) => {
                bom::export(&mut app, &a.name, &a.version);
            }
            BomSubCommand::Show(a) => {
                // Note: version is borrowed as an Option
                // not required for this command to work
                bom::show(&mut app, &a.part_number, &a.version);
            }
        },
        _ => {}
    }
}
