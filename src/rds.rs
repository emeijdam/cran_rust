use chrono::{NaiveDate, TimeZone, Utc};
use csv::{QuoteStyle, WriterBuilder};
use rds2rust::{Attributes, Logical, RObject, read_rds};
use std::{error::Error, fmt, fs::File, path::Path, sync::Arc};

// --- Custom Error Type ---

#[derive(Debug)]
pub enum RdsError {
    Io(std::io::Error),
    RdsRead(rds2rust::Error),
    Csv(csv::Error),
    UnsupportedType(String),
    InvalidStructure(String),
    MissingAttribute(String),
}

impl fmt::Display for RdsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RdsError::Io(err) => write!(f, "IO Error: {}", err),
            RdsError::RdsRead(err) => write!(f, "RDS Read Error: {}", err),
            RdsError::Csv(err) => write!(f, "CSV Error: {}", err),
            RdsError::UnsupportedType(t) => write!(f, "Unsupported RObject type: {}", t),
            RdsError::InvalidStructure(msg) => write!(f, "Invalid RDS structure: {}", msg),
            RdsError::MissingAttribute(attr) => write!(f, "Missing expected attribute: {}", attr),
        }
    }
}

impl Error for RdsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RdsError::Io(err) => Some(err),
            RdsError::RdsRead(err) => Some(err),
            RdsError::Csv(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RdsError {
    fn from(err: std::io::Error) -> Self {
        RdsError::Io(err)
    }
}

impl From<rds2rust::Error> for RdsError {
    fn from(err: rds2rust::Error) -> Self {
        RdsError::RdsRead(err)
    }
}

impl From<csv::Error> for RdsError {
    fn from(err: csv::Error) -> Self {
        RdsError::Csv(err)
    }
}

// --- Main Converter ---

pub fn parse_rds_file(input_path: &Path, output_path: &Path) -> Result<(), RdsError> {
    let data = std::fs::read(input_path)?;
    let result = read_rds(&data)?;
    RdsConverter::new(output_path).convert(&result.object)
}

struct RdsConverter<'a> {
    output_path: &'a Path,
}

impl<'a> RdsConverter<'a> {
    fn new(output_path: &'a Path) -> Self {
        Self { output_path }
    }

    fn convert(&self, r_object: &RObject) -> Result<(), RdsError> {
        match r_object {
            RObject::WithAttributes { object, attributes } => {
                self.handle_with_attributes(object, attributes)
            }
            // Other top-level types could be handled here
            _ => Err(RdsError::UnsupportedType(format!(
                "Top-level RObject must be WithAttributes, found {:?}",
                r_object.variant_name()
            ))),
        }
    }

    /// Handles the main `WithAttributes` RObject, dispatching to more specific handlers.
    fn handle_with_attributes(
        &self,
        object: &RObject,
        attributes: &Attributes,
    ) -> Result<(), RdsError> {
        let attr_map: std::collections::HashMap<_, _> =
            attributes.iter().map(|(k, v)| (k.as_ref(), v)).collect();

        // Case 1: Matrix-like structure (e.g., packages.rds)
        if attr_map.contains_key("dim") && attr_map.contains_key("dimnames") {
            self.handle_matrix(object, &attr_map)
        }
        // Case 2: List of DataFrames (e.g., archive.rds)
        else if attr_map.contains_key("names") {
            if let RObject::List(items) = object {
                self.handle_dataframe_list(items, &attr_map)
            } else {
                Err(RdsError::InvalidStructure(
                    "Expected a List object when 'names' attribute is present".to_string(),
                ))
            }
        }
        // If neither, it's an unsupported structure
        else {
            Err(RdsError::InvalidStructure(
                "RObject with attributes does not match known structures (matrix or named list)"
                    .to_string(),
            ))
        }
    }

    /// Handles RDS files structured like a matrix with dimensions and dimension names.
    /// This is typical for files like the main `packages.rds`.
    fn handle_matrix(
        &self,
        object: &RObject,
        attr_map: &std::collections::HashMap<&str, &RObject>,
    ) -> Result<(), RdsError> {
        let (num_rows, _num_cols) = match attr_map.get("dim") {
            Some(RObject::Integer(dims)) if dims.len() == 2 => (dims[0], dims[1]),
            _ => {
                return Err(RdsError::InvalidStructure(
                    "Expected 'dim' attribute with two integers".to_string(),
                ));
            }
        };

        let columns: Vec<String> = match attr_map.get("dimnames") {
            Some(RObject::List(dimnames)) if dimnames.len() > 1 => match &dimnames[1] {
                RObject::Character(cols) => cols.iter().map(|s| s.to_string()).collect(),
                _ => {
                    return Err(RdsError::InvalidStructure(
                        "Second element of 'dimnames' must be a character vector".to_string(),
                    ));
                }
            },
            _ => {
                return Err(RdsError::InvalidStructure(
                    "Expected 'dimnames' attribute as a list".to_string(),
                ));
            }
        };

        let mut wtr = WriterBuilder::new()
            .quote_style(QuoteStyle::NonNumeric)
            .delimiter(b';')
            .from_path(self.output_path)?;

        wtr.write_record(&columns)?;

        let data_vec = match object {
            RObject::Character(v) => v,
            _ => {
                return Err(RdsError::UnsupportedType(
                    "Matrix data must be a character vector".to_string(),
                ));
            }
        };

        // Data is column-major, so we iterate through rows and pick the correct cell.
        for row_idx in 0..num_rows as usize {
            let mut row: Vec<String> = Vec::with_capacity(columns.len());
            for col_idx in 0..columns.len() {
                let data_idx = (col_idx as i32 * num_rows) as usize + row_idx;
                row.push(
                    data_vec
                        .get(data_idx)
                        .cloned()
                        .unwrap_or_default()
                        .to_string(),
                );
            }
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Handles RDS files structured as a named list of data frames.
    /// This is typical for files like `archive.rds`.
    fn handle_dataframe_list(
        &self,
        items: &[RObject],
        attr_map: &std::collections::HashMap<&str, &RObject>,
    ) -> Result<(), RdsError> {
        let package_names: Vec<String> = match attr_map.get("names") {
            Some(RObject::Character(names)) => names.iter().map(|s| s.to_string()).collect(),
            _ => {
                return Err(RdsError::InvalidStructure(
                    "Expected 'names' attribute to be a character vector".to_string(),
                ));
            }
        };

        if items.is_empty() {
            return Ok(()); // Nothing to write
        }

        // Determine header from the first data frame
        let mut header: Vec<String> = match items.get(0) {
            Some(RObject::DataFrame(df)) => df
                .columns
                .iter()
                .map(|(name, _)| name.to_string())
                .collect(),
            _ => {
                return Err(RdsError::InvalidStructure(
                    "Expected first item in list to be a DataFrame".to_string(),
                ));
            }
        };

        let mut wtr = WriterBuilder::new()
            .quote_style(QuoteStyle::NonNumeric)
            .delimiter(b';')
            .from_path(self.output_path)?;

        // Prepend custom columns
        header.insert(0, "file_name".to_string());
        header.insert(0, "package_name".to_string());
        wtr.write_record(&header)?;

        for (index, item) in items.iter().enumerate() {
            let package_name = package_names.get(index).map_or("", |s| s.as_str());

            if let RObject::DataFrame(_df) = item {
                // _df to silence unused variable warning
                self.write_dataframe_rows(&mut wtr, item, package_name)?;
            }
        }

        wtr.flush()?;
        Ok(())
    }

    fn write_dataframe_rows(
        &self,
        wtr: &mut csv::Writer<File>,
        df_robject: &RObject, // Renamed parameter to avoid conflict
        package_name: &str,
    ) -> Result<(), RdsError> {
        let df_struct = if let RObject::DataFrame(df_inner) = df_robject {
            df_inner
        } else {
            return Err(RdsError::InvalidStructure(
                "Expected RObject::DataFrame in write_dataframe_rows".to_string(),
            ));
        };

        let num_rows = df_struct.row_names.len();
        if num_rows == 0 {
            return Ok(());
        }

        // 1. Convert all columns to string representations first
        let mut string_cols: Vec<Vec<String>> = Vec::new();
        for (_, col_data) in &df_struct.columns {
            let col_as_strings = convert_robject_to_strings(col_data, num_rows)?;
            string_cols.push(col_as_strings);
        }

        // 2. Write row by row
        for row_idx in 0..num_rows {
            let mut record: Vec<&str> = Vec::with_capacity(df_struct.columns.len() + 2);
            record.push(package_name);
            record.push(
                df_struct
                    .row_names
                    .get(row_idx)
                    .map_or("", |s: &Arc<str>| s.as_ref()),
            );

            for col in &string_cols {
                record.push(col.get(row_idx).map_or("", |s| s.as_str()));
            }

            wtr.write_record(&record)?;
        }

        Ok(())
    }
}

// --- Helper Functions ---

/// Converts a single RObject vector into a Vec of Strings.
fn convert_robject_to_strings(robject: &RObject, num_rows: usize) -> Result<Vec<String>, RdsError> {
    match robject {
        RObject::Integer(v) => Ok(v.iter().map(|x| x.to_string()).collect()),
        RObject::Real(v) => Ok(v.iter().map(|x| x.to_string()).collect()),
        RObject::Character(v) => Ok(v.iter().map(|s| s.to_string()).collect()),
        RObject::Logical(v) => Ok(v
            .iter()
            .map(|&x| (if x == Logical::True { "TRUE" } else { "FALSE" }).to_string())
            .collect()),
        RObject::S3Object(s3) => {
            let classes: Vec<&str> = s3.class.iter().map(AsRef::as_ref).collect();
            if classes.contains(&"octmode") {
                if let RObject::Integer(v) = &*s3.base {
                    return Ok(v.iter().map(|x| format!("{:o}", x)).collect());
                }
            } else if classes.contains(&"Date") {
                if let RObject::Real(v) = &*s3.base {
                    return Ok(v
                        .iter()
                        .map(|&days| {
                            NaiveDate::from_ymd_opt(1970, 1, 1)
                                .and_then(|epoch: chrono::NaiveDate| {
                                    epoch.checked_add_signed(chrono::Duration::days(days as i64))
                                })
                                .map_or_else(
                                    || "Invalid Date".to_string(),
                                    |d: chrono::NaiveDate| d.to_string(),
                                )
                        })
                        .collect());
                }
            } else if classes.contains(&"POSIXct") {
                if let RObject::Real(v) = &*s3.base {
                    return Ok(v
                        .iter()
                        .map(|&secs| {
                            Utc.timestamp_opt(secs as i64, 0).single().map_or_else(
                                || "Invalid Time".to_string(),
                                |dt: chrono::DateTime<Utc>| {
                                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                                },
                            )
                        })
                        .collect());
                }
            }
            // Fallback for unknown S3 types
            Ok(vec!["<S3>".to_string(); num_rows])
        }
        // Fallback for other RObject types
        _ => Ok(vec!["".to_string(); num_rows]),
    }
}
