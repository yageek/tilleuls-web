use calamine::{open_workbook, DataType, Reader, Rows, Xlsx};
use chrono::Utc;
use log::warn;
use std::path::Path;
use thiserror::Error;

use tilleuls_domain::{Category, Item, WeeklyBasketOffer};

const HEADER: &'static str = "Bon de commande N°";
const COMMAND_SHEET_NAME: &'static str = "Commande";
const OTHER_SHEET_NAME: &'static str = "Recap";

const COLUMNS_ORDER: &'static [&'static str] =
    &["Produits", "Unité", "Prix vente TTC", "Quantité", "Total"];

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("opeming error")]
    OpeningError(#[from] calamine::XlsxError),
    #[error("unrecognized file element")]
    InvalidFileType,
}

// R: Reader<RS = BufReader<File>>,
pub fn import_xslx<P: AsRef<Path>>(path: P) -> Result<WeeklyBasketOffer, ImportError> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    // We validate the known shape of the current formular.
    // For now, we assume one worksheet exists with two elements

    let sheets = workbook.sheet_names();
    if !sheets.contains(&COMMAND_SHEET_NAME.to_owned())
        || !sheets.contains(&OTHER_SHEET_NAME.to_owned())
    {
        warn!("Missing known columns");
        return Err(ImportError::InvalidFileType);
    }

    let range = match workbook.worksheet_range(COMMAND_SHEET_NAME) {
        Some(Ok(range)) => range,
        _ => return Err(ImportError::InvalidFileType),
    };

    // Retrieve iterator
    let rows = &mut range.rows();

    // Find the header
    if !has_header(rows) || !has_product_columns(rows) {
        warn!("Did not find the header");
        return Err(ImportError::InvalidFileType);
    }

    // Here the iterator is correct
    let mut categories: Vec<Category> = vec![];
    let mut reached_botom = false;
    while !reached_botom {
        if let Some(cells) = rows.next() {
            // Retrieve cells as reference string
            let string_cells: Vec<&String> = cells
                .iter()
                .flat_map(|c| match c {
                    DataType::String(s) => Some(s),
                    _ => None,
                })
                .collect();

            // If we have a category | or end
            if string_cells.len() == 1 {
                if string_cells[0] == "TOTAL" {
                    reached_botom = true;
                } else {
                    let category = Category::new(string_cells[0]);
                    categories.push(category);
                }
            } else if string_cells.len() == 2 {
                if let Some(last) = categories.last_mut() {
                    let item = Item::new(string_cells[0], string_cells[1]);
                    last.add_item(item);
                }
            }
        } else {
            reached_botom = true;
        }
    }

    Ok(WeeklyBasketOffer::new(Utc::today(), categories))
}

fn has_header(rows: &mut Rows<DataType>) -> bool {
    rows.find(|c| match c.first() {
        Some(DataType::String(txt)) if txt == HEADER => true,
        _ => false,
    })
    .is_some()
}

fn has_product_columns(rows: &mut Rows<DataType>) -> bool {
    rows.find(|cells| {
        if cells.len() < COLUMNS_ORDER.len() {
            return false;
        }
        for i in 0..COLUMNS_ORDER.len() {
            if let DataType::String(txt) = &cells[i] {
                if txt == COLUMNS_ORDER[i] {
                    continue;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        return true;
    })
    .is_some()
}

#[cfg(test)]
mod tests {
    use crate::import_xslx;
    use calamine::{open_workbook, DataType, Reader, Xlsx};
    use std::path::PathBuf;
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn print_elements() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("assets/test.xlsx");
        println!("Opening: {}", d.display());

        let mut workbook: Xlsx<_> = open_workbook(d).unwrap();

        if let Some(Ok(sheets)) = workbook.worksheet_range("Commande") {
            for row in sheets.rows() {
                let string = row
                    .iter()
                    .flat_map(|e| {
                        if let DataType::String(txt) = e {
                            Some(txt.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                println!("Rows: {}", string);
            }
        }
    }

    #[test]
    fn load_elements() {
        init();
        std::env::set_var("RUST_LOG", "trace");
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("assets/test.xlsx");
        println!("Opening: {}", d.display());

        let importer = import_xslx(d).expect("Should parse correctly");
    }
}
