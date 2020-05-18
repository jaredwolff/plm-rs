#[macro_use]
extern crate prettytable;

mod parts;

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
  Builds(Builds),
  Inventory(Inventory),
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
}

/// Create parts manually
#[derive(Clap)]
struct CreateParts {}

/// Delete parts manually
#[derive(Clap)]
struct DeleteParts {}

/// Show all parts
#[derive(Clap)]
struct ShowParts {}

#[derive(Clap)]
#[clap(version = crate_version!())]
enum BuildsSubCommand {
  Create(CreateBuild),
  Delete(DeleteBuild),
  Show(ShowBuilds),
}

/// Create build manually
#[derive(Clap)]
struct CreateBuild {}

/// Delete build manually
#[derive(Clap)]
struct DeleteBuild {}

/// Show all builds
#[derive(Clap)]
struct ShowBuilds {}

/// A subcommand for adding/modifying/removing/completing builds
#[derive(Clap)]
struct Builds {
  #[clap(subcommand)]
  subcmd: BuildsSubCommand,
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
  Delete(DeleteInventory),
  Show(ShowInventory),
}

/// Create inventory manually
#[derive(Clap)]
struct CreateInventory {}

/// Delete inventory manually
#[derive(Clap)]
struct DeleteInventory {}

/// Show all inventory
#[derive(Clap)]
struct ShowInventory {}

fn main() {
  let opts: Opts = Opts::parse();

  match opts.subcmd {
    SubCommand::Builds(s) => match s.subcmd {
      BuildsSubCommand::Create(_) => {
        println!("create!");
      }
      BuildsSubCommand::Delete(_) => {
        println!("delete!");
      }
      BuildsSubCommand::Show(_) => {
        println!("show!");
      }
    },
    SubCommand::Inventory(s) => match s.subcmd {
      InventorySubCommand::Create(_) => {
        println!("create!");
      }
      InventorySubCommand::Delete(_) => {
        println!("delete!");
      }
      InventorySubCommand::Show(_) => {
        println!("show!");
      }
    },
    SubCommand::Parts(s) => match s.subcmd {
      PartsSubCommand::Create(_) => {
        parts::create();
      }
      PartsSubCommand::Delete(_) => {
        parts::delete();
      }
      PartsSubCommand::Show(_) => {
        parts::show();
      }
    },
  }
}
