//! Export functionality for PDF generation

use crate::world::VirtualWorld;

/// Format compilation errors into a single error string
fn format_errors<I, T>(errors: I, prefix: &str) -> String
where
    I: IntoIterator<Item = T>,
    T: std::fmt::Display,
{
    errors
        .into_iter()
        .map(|e| format!("{prefix}: {e}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate PDF bytes from Typst source code
pub fn pdf_bytes_from_source(source: &str) -> Result<Vec<u8>, String> {
    let world = VirtualWorld::new(source);

    let document = typst::compile(&world)
        .output
        .map_err(|errors| format_errors(errors.iter().map(|e| &e.message), "Error"))?;

    typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|errors| format_errors(errors.iter().map(|e| &e.message), "PDF Error"))
}

/// Generate a data URL for the PDF
pub fn pdf_data_url(source: &str) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let bytes = pdf_bytes_from_source(source)?;
    let base64 = STANDARD.encode(&bytes);
    Ok(format!("data:application/pdf;base64,{base64}"))
}
