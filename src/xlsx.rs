use calamine::{open_workbook, DataType, Reader, Rows, Xlsx};
use chrono::Utc;
use log::warn;
use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};
use thiserror::Error;

use super::models::{Catalog, Category, Item};

const HEADER: &'static str = "Bon de commande N°";
const COMMAND_SHEET_NAME: &'static str = "Commande";
const OTHER_SHEET_NAME: &'static str = "Recap";

const COLUMNS_ORDER: &'static [&'static str] =
    &["Produits", "Unité", "Prix vente TTC", "Quantité", "Total"];

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("opening error")]
    OpeningError(#[from] calamine::XlsxError),
    #[error("unrecognized file element")]
    InvalidFileType,
    #[error("unexpected cell content")]
    UnexpectedCellContent,
}

fn read_string_cell(cell: &DataType) -> Result<&String, ImportError> {
    match cell {
        DataType::String(txt) => Ok(txt),
        _ => Err(ImportError::UnexpectedCellContent),
    }
}

fn read_float_cell(cell: &DataType) -> Result<f64, ImportError> {
    match cell {
        DataType::Float(value) => Ok(*value),
        _ => Err(ImportError::UnexpectedCellContent),
    }
}

fn is_string_cell(cell: &DataType) -> bool {
    match cell {
        DataType::String(_) => true,
        _ => false,
    }
}

fn is_float_cell(cell: &DataType) -> bool {
    match cell {
        DataType::Float(_) => true,
        _ => false,
    }
}

fn is_empty_cell(cell: &DataType) -> bool {
    match cell {
        DataType::Empty => true,
        _ => false,
    }
}

fn is_end_of_items(cells: &[DataType]) -> bool {
    match &cells[3] {
        DataType::String(txt) if txt == "TOTAL" => true,
        _ => false,
    }
}

fn read_category_row(cells: &[DataType]) -> Option<Category> {
    if is_string_cell(&cells[0]) && is_empty_cell(&cells[1]) && is_empty_cell(&cells[2]) {
        let txt = read_string_cell(&cells[0]).unwrap();
        Some(Category::new(txt))
    } else {
        None
    }
}

fn read_item_row(cells: &[DataType]) -> Result<Option<Item>, ImportError> {
    if is_string_cell(&cells[0]) && is_float_cell(&cells[2]) {
        let title = read_string_cell(&cells[0])?.clone();
        let unit = read_string_cell(&cells[1])
            .unwrap_or(&"1".to_owned())
            .clone();
        let price = read_float_cell(&cells[2])?;
        Ok(Some(Item::new(title, unit, price)))
    } else {
        Ok(None)
    }
}

/// Import and decode on xslx file provided by the farm
pub fn import_xlsx<R: Read + Seek>(reader: R) -> Result<Catalog, ImportError> {
    let mut workbook = Xlsx::new(reader)?;
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
            if cells.len() < 5 {
                continue;
            }

            if is_end_of_items(cells) {
                reached_botom = true;
            } else if let Some(category) = read_category_row(cells) {
                categories.push(category);
            } else {
                match read_item_row(cells) {
                    Ok(Some(item)) => {
                        if let Some(last) = categories.last_mut() {
                            last.add_item(item)
                        }
                    }
                    Err(e) => return Err(e),
                    _ => (),
                }
            }
        } else {
            reached_botom = true;
        }
    }

    Ok(Catalog::new(categories))
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
    use super::import_xlsx;
    // use calamine::{open_workbook, DataType, Reader, Xlsx};
    use std::{fs::File, io::BufReader, path::PathBuf};
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // #[test]
    // fn print_elements() {
    //     let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //     d.push("assets/test.xlsx");
    //     println!("Opening: {}", d.display());

    //     let mut workbook: Xlsx<_> = open_workbook(d).unwrap();

    //     if let Some(Ok(sheets)) = workbook.worksheet_range("Commande") {
    //         for row in sheets.rows() {
    //             let string = row
    //                 .iter()
    //                 .map(|e| match e {
    //                     DataType::String(txt) => format!("Text: {}", txt),
    //                     DataType::Float(value) => format!("Float: {}", value),
    //                     DataType::Bool(value) => format!("Bool: {}", value),
    //                     DataType::Int(value) => format!("Int: {}", value),
    //                     DataType::Error(value) => format!("Err: {}", value),
    //                     DataType::Empty => "Empty".to_owned(),
    //                 })
    //                 .collect::<Vec<_>>()
    //                 .join("|");
    //             println!("Rows: {}", string);
    //         }
    //     }
    // }

    #[test]
    fn load_elements() {
        init();
        std::env::set_var("RUST_LOG", "trace");
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("tests_assets/test.xlsx");
        println!("Opening: {}", d.display());

        let file = File::open(d).unwrap();
        let mut reader = BufReader::new(file);
        let week_offer = import_xlsx(reader).expect("Should parse correctly");

        assert_eq!(10, week_offer.categories().len());
    }
}
