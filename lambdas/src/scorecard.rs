use aws_lambda_events::event::sqs::SqsEvent;
use aws_sdk_s3::primitives::ByteStream;
use bnacore::combine::combine_mem;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use minijinja::Environment;
use serde_json::Value;
use std::fs;
use std::io::BufWriter;

const BUCKET_NAME: &str = "remy-is-testing";

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    // Load the parameters.
    let v: Value = serde_json::from_str(
        event.payload.records[0]
            .body
            .clone()
            .unwrap_or("{}".to_string())
            .as_str(),
    )?;

    // Compute the bucket key.
    let year = &v["year"];
    let country = &v["country"];
    let city = &v["city"];
    let region = v.get("region").unwrap_or(city);
    let key = format!("{}/{}{}{}", year, country, region, city);

    // Load the template.
    let source_page_1 = include_str!("../../assets/visuals/template-scorecard-pg1-v23.2.svg");
    let mut env = Environment::new();
    env.add_template("scorecard", &source_page_1).unwrap();

    // Render the template to file for this specific record.
    let template = env.get_template("scorecard").unwrap();
    let rendered = template.render(&v)?;

    // Convert it to pdf.
    let pdf_page_1 = svg2pdf::convert_str(&rendered, svg2pdf::Options::default())?;

    // Load the second page and convert it to pdf.
    let source_page_2 = include_str!("../../assets/visuals/template-scorecard-pg1-v23.2.svg");
    let pdf_page_2 = svg2pdf::convert_str(&source_page_2, svg2pdf::Options::default())?;

    // Combine the 2 pages.
    let full_pdf = vec![pdf_page_1.as_slice(), pdf_page_2.as_slice()];
    let mut scorecard = combine_mem(full_pdf.as_slice())?;
    let buffer: Vec<u8> = Vec::new();
    let mut writer = BufWriter::new(buffer);
    scorecard.save_to(writer.get_mut())?;

    // Upload to S3.
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);
    let body = ByteStream::from(writer.buffer().to_vec());
    client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(key)
        .body(body)
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
