#[macro_use]
extern crate prettytable;

mod tables;
use tables::*;

use clap::{crate_version, Clap};

#[derive(Clap)]
#[clap(version = crate_version!())]
struct Opts {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Clap)]
#[clap(version = crate_version!())]
enum SubCommand {
  Parts(Parts),
  Build(Build),
  Inventory(Inventory),
  Bom(Bom),
}

/// A subcommand for adding/modifying/removing parts
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
}

/// A subcommand for importing a bom from an Eagle .sch file
#[derive(Clap)]
struct ImportBom {
  #[clap(short, long)]
  filename: String,
}

/// A subcommand for showing a bom from pn
#[derive(Clap)]
struct ShowBom {
  /// Part number of the Bom in question
  #[clap(short, long)]
  part_number: String,
  /// Version of the part in question
  #[clap(short, long)]
  version: i32,
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
struct DeleteBuild {}

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
  #[clap(short, long)]
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
  Export(ExportInventory),
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
  #[clap(short, long)]
  filename: String,
}

/// Export inventory and shortages via .csv
#[derive(Clap)]
struct ExportInventory {
  /// Inventory to a .csv file
  #[clap(short, long)]
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

// TODO: maybe reverse the arguments so that it's more of an action show inventory vs inventory show
fn main() {
  let opts: Opts = Opts::parse();

  match opts.subcmd {
    SubCommand::Build(s) => match s.subcmd {
      BuildSubCommand::Create(_) => {
        builds::create();
      }
      BuildSubCommand::Delete(_) => {
        println!("delete!");
      }
      BuildSubCommand::Show(a) => {
        builds::show(a.all);
      }
      BuildSubCommand::Complete(a) => {
        builds::complete(a.build_id);
      }
    },
    SubCommand::Inventory(s) => match s.subcmd {
      InventorySubCommand::Create(_) => {
        inventory::create();
      }
      InventorySubCommand::Import(a) => {
        inventory::create_from_file(&a.filename);
      }
      InventorySubCommand::Export(a) => {
        inventory::export_to_file(&a.filename);
      }
      InventorySubCommand::Delete(_) => {
        println!("delete!");
      }
      InventorySubCommand::Show(a) => {
        if a.show_shortage {
          inventory::show_shortage(a.all_entries);
        } else {
          inventory::show();
        }
      }
    },
    // TODO: Search for a part
    SubCommand::Parts(s) => match s.subcmd {
      PartsSubCommand::Create(a) => match a.filename {
        Some(x) => parts::create_by_csv(&x),
        None => parts::create(),
      },
      PartsSubCommand::Delete(_) => {
        parts::delete();
      }
      PartsSubCommand::Show(_) => {
        parts::show();
      }
      PartsSubCommand::Rename(_) => {
        parts::rename();
      }
    },
    SubCommand::Bom(s) => match s.subcmd {
      BomSubCommand::Import(a) => {
        bom::import(&a.filename);
      }
      BomSubCommand::Show(a) => {
        bom::show(&a.part_number, &a.version);
      }
    },
  }
}
