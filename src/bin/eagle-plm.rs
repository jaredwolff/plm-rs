use clap::{crate_version, Clap};

use eagle_plm::{config, tables::*};

#[derive(Clap)]
#[clap(version = crate_version!())]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
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

    let config;

    // First check if the config is valid
    match &opts.subcmd {
        SubCommand::Install(_s) => {
            let db_name = "database.db".to_string();

            // Get default config
            let config = config::Config {
                database_name: db_name.clone(),
                library_name: "your-library".to_string(),
            };

            // Install the config
            if config::set_config(&config).is_err() {
                eprintln!("Unable to install database and config");
                std::process::exit(1);
            }

            let mut path = config::get_config_path().unwrap();
            path.push("config.toml");

            println!("Config installed to {}", path.to_string_lossy());
            std::process::exit(0);
        }
        _ => {
            // Set the config
            config = match config::get_config() {
                Ok(c) => c,
                Err(_e) => {
                    eprintln!("Unable to get config. Run `eagle-plm install` first.");
                    std::process::exit(1);
                }
            };
        }
    };

    // Then run the command.
    match opts.subcmd {
        SubCommand::Build(s) => match s.subcmd {
            BuildSubCommand::Create(_) => {
                builds::create(&config);
            }
            // TODO: take the next argument after delete instead of needing a flag...
            BuildSubCommand::Delete(a) => {
                builds::delete(&config, a.build_id);
            }
            BuildSubCommand::Show(a) => {
                builds::show(&config, a.all);
            }
            BuildSubCommand::Complete(a) => {
                builds::complete(&config, a.build_id);
            }
        },
        SubCommand::Inventory(s) => match s.subcmd {
            InventorySubCommand::Create(_) => {
                inventory::create(&config);
            }
            InventorySubCommand::Import(a) => {
                inventory::create_from_file(&config, &a.filename);
            }
            InventorySubCommand::Update(a) => {
                inventory::update_from_file(&config, &a.filename);
            }
            InventorySubCommand::Export(a) => {
                inventory::export_to_file(&config, &a.filename, a.export_all);
            }
            InventorySubCommand::Shortages(a) => {
                inventory::export_shortages_to_file(&config, &a.filename);
            }
            InventorySubCommand::Delete(_) => {
                println!("Not implemented!");
            }
            InventorySubCommand::Show(a) => {
                if a.show_shortage {
                    inventory::show_shortage(&config, a.all_entries);
                } else {
                    inventory::show(&config, a.all_entries);
                }
            }
        },
        // TODO: Search for a part
        SubCommand::Parts(s) => match s.subcmd {
            PartsSubCommand::Create(a) => match a.filename {
                Some(x) => parts::create_by_csv(&config, &x),
                None => parts::create(&config),
            },
            PartsSubCommand::Delete(_) => {
                parts::delete(&config);
            }
            PartsSubCommand::Show(_) => {
                parts::show(&config);
            }
            PartsSubCommand::Rename(_) => {
                parts::rename(&config);
            }
        },
        SubCommand::Bom(s) => match s.subcmd {
            BomSubCommand::Import(a) => {
                bom::import(&config, &a.filename);
            }
            BomSubCommand::Export(a) => {
                bom::export(&config, &a.name, &a.version);
            }
            BomSubCommand::Show(a) => {
                // Note: version is borrowed as an Option
                // not required for this command to work
                bom::show(&config, &a.part_number, &a.version);
            }
        },
        _ => {}
    }
}
