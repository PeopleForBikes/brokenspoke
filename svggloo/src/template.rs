use clap::ValueEnum;
use color_eyre::{eyre::Report, Result};
use csv::Reader;
use minijinja::Environment;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

type Record = HashMap<String, String>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Exporter {
    Inkscape,
    CairoSVG,
    SVG2PDF,
}

/// Render an SVG template.
///
/// Merges the data from the CSV file into the SVG template to create a new SVG
/// file and render it to PDF.
///
/// The `field_based_name` argument can be used to specify one or several fields
/// from the CSV file that must be used to name the output files. If the fields
/// don't exist, this function will panic. Once all the fields are being
/// collected, they are transformed to lowercase and concatenated together using
/// the `separator`, in the order they were specified.
///
/// If `field_based_name` is not specified, it defaults to the first field of a
/// CSV record.
///
/// If `separator` is not specified, it defaults to dash (`-`).
///
/// ```no_run
/// # use color_eyre::{eyre::Report, Result};
/// use std::path::Path;
/// use svggloo::template::{render, Exporter};
///
/// # fn main() -> Result<(), Report> {
/// let svg_template = Path::new("SVG_TEMPLATE_FILENAME");
/// let output_dir = Path::new("OUTPUT_DIR");
/// let fields = vec![
///     String::from("country"),
///     String::from("state"),
///     String::from("city"),
/// ];
/// let _ = render(
///     &svg_template.canonicalize()?,
///     output_dir,
///     Some(Exporter::CairoSVG),
///     Some(fields),
///     None,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn render(
    svg_template: &Path,
    output_dir: &Path,
    exporter: Option<Exporter>,
    field_based_name: Option<Vec<String>>,
    separator: Option<&str>,
) -> Result<(), Report> {
    // Locate the template file data and the prepare the output directory.
    let template_data = svg_template.with_extension("csv");
    fs::create_dir_all(output_dir)?;

    // Load the template.
    let source = fs::read_to_string(svg_template)?;
    let name = svg_template
        .file_name()
        .expect("Invalid template name.")
        .to_str()
        .unwrap();
    let mut env = Environment::new();
    env.add_template(name, &source)?;
    let tmpl = env.get_template(name).unwrap();

    // Set the separator.
    let sep = separator.unwrap_or("-");

    // Read the CSV.
    let mut csv_reader = Reader::from_path(template_data)?;
    let mut files: Vec<PathBuf> = Vec::new();
    for result in csv_reader.deserialize() {
        let record: Record = result?;

        // Construct the name of the output file.
        let item_name = match field_based_name.clone() {
            Some(fields) => {
                let v = fields
                    .clone()
                    .iter()
                    .map(|f| record[f].clone())
                    .map(|f| f.replace(' ', "_"))
                    .collect::<Vec<String>>();
                v.join(sep).to_lowercase()
            }
            None => record.values().next().unwrap().to_owned().to_lowercase(),
        };
        let mut item = item_name.clone();
        item.push_str(".svg");

        // Render the template to file for this specific record.
        let rendered = tmpl.render(&record)?;
        let output_file = output_dir.join(&item);
        fs::write(&output_file, rendered)?;
        files.push(output_file);
    }

    // Convert it to pdf.
    if let Some(exporter) = exporter {
        match exporter {
            Exporter::Inkscape => export_with_inkscape(&files),
            Exporter::CairoSVG => export_with_cairosvg(&files),
            Exporter::SVG2PDF => export_with_svg2pdf(&files),
        }
    }
    Ok(())
}

/// Render the template using a record from the CSV file.
///
/// ```no_run
/// # use color_eyre::{eyre::Report, Result};
/// use svggloo::template::render_record;
/// use std::collections::HashMap;
///
/// # fn main() -> Result<(), Report> {
/// let template = "This is {{city}}.";
/// let record = HashMap::from([("city", "Austin")]);
/// let rendered = render_record(template, record)?;
/// assert_eq!(rendered, String::from("This is Austin"));
/// # Ok(())
/// # }
/// ```
pub fn render_record<S: Serialize>(template: &str, record: S) -> Result<String, Report> {
    let name = "template";
    let mut env = Environment::new();
    env.add_template(name, template)?;
    let tmpl = env.get_template(name).unwrap();

    // Render the template to file for this specific record.
    Ok(tmpl.render(&record)?)
}

/// Render a template file using a record from the CSV file.
pub fn render_record_from_file<S: Serialize>(
    svg_template: &Path,
    record: S,
) -> Result<String, Report> {
    // Load the template.
    let template = fs::read_to_string(svg_template)?;

    // Render it.
    render_record(&template, record)
}

/// Exports an SVG file to a PDF with Inkscape.
///
/// Exports an SVG `src` file as a PDF with the same name.
///
/// The export is done using Inkspace. If Inkscape is not found, this function
/// will panic.
pub fn export_with_inkscape(srcs: &[PathBuf]) {
    // Set the name of the Inkscape binary.
    let program = "inkscape";

    // Prepare the Inkscape arguments.
    let export_filenames = srcs
        .iter()
        .map(|s| s.clone().into_os_string())
        .filter_map(|src| src.into_string().ok())
        .collect::<Vec<String>>();
    let mut args = vec![
        "--export-area-drawing".to_owned(),
        "--batch-process".to_owned(),
        "--export-type=pdf".to_owned(),
    ];
    args.extend(export_filenames);

    export_with(program, &args);
}

/// Export with a specific program and arguments.
fn export_with(program: &str, args: &[String]) {
    // Prepare the error message.
    let error_msg = format!(
        "Failed to execute command `{} {}`",
        program,
        &args.join(" ")
    );
    // Execute the export command.
    let _output = Command::new(program).args(args).output().expect(&error_msg);
}

/// Exports an SVG file to a PDF with CairoSVG.
///
/// Exports an SVG `src` file as a PDF with the same name.
///
/// The export is done using CairoSVG. If CairoSVG is not found, this function
/// will panic.
pub fn export_with_cairosvg(srcs: &[PathBuf]) {
    for src in srcs {
        // Prepare the input/output values from the src argument.
        let (in_svg, out_pdf) = get_in_out_file(src);

        // Prepare the command.
        let program = "cairosvg";
        let args = vec![
            "-f".to_owned(),
            "pdf".to_owned(),
            "-o".to_owned(),
            out_pdf,
            in_svg,
        ];

        export_with(program, &args);
    }
}

pub fn export_with_svg2pdf(srcs: &[PathBuf]) {
    for src in srcs {
        // Prepare the input/output values from the src argument.
        let (in_svg, _out_pdf) = get_in_out_file(src);

        // Prepare the command.
        let program = "svg2pdf";
        let args = vec![in_svg];

        export_with(program, &args);
    }
}

/// Get the input and output string representations of the provided file.
fn get_in_out_file<P>(src: P) -> (String, String)
where
    P: AsRef<Path>,
{
    let in_svg = src
        .as_ref()
        .to_str()
        .expect("The src file path is not valid UTF-8.");
    let dest = src.as_ref().with_extension("pdf");
    let out_pdf = dest
        .to_str()
        .expect("The dest file path is not valid UTF-8");

    (in_svg.into(), out_pdf.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_in_out_file() {
        let src = PathBuf::from("brochure.svg");
        let (in_svg, out_pdf) = get_in_out_file(src);
        assert_eq!(in_svg, String::from("brochure.svg"));
        assert_eq!(out_pdf, String::from("brochure.pdf"));
    }
}
